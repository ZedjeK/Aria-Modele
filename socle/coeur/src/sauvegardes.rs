//! Sauvegardes : une copie datée de chaque fichier de l'utilisateur, faite avant de le modifier, dans
//! `sauvegardes\` du dossier de données. Rien n'est jamais supprimé ni écrasé ici ; restaurer, c'est recopier.
//!
//! `sauvegardes\index.json` note d'où vient chaque copie : c'est ce qui permet de la restaurer. Une restauration
//! commence par sauvegarder l'état actuel, puis remet l'ancien : elle ne supprime rien.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::Serialize;

use time::OffsetDateTime;
use time::format_description::BorrowedFormatItem;
use time::macros::format_description;

use crate::{Dossier, Erreur, donnees, journal};

/// Le registre des copies : nom de la copie → chemin d'origine.
pub const INDEX: &str = "index.json";

/// Deux sauvegardes en même temps n'écrivent pas le registre l'une sur l'autre.
static VERROU: Mutex<()> = Mutex::new(());

/// L'horodatage en tête du nom d'une sauvegarde : `2026-11-21_19-05-00`.
const FORME_HORODATAGE: &[BorrowedFormatItem<'static>] =
    format_description!("[year]-[month]-[day]_[hour]-[minute]-[second]");

/// Copie le fichier avant sa modification et renvoie le chemin de la copie.
///
/// # Errors
/// Une erreur si la copie échoue : sans sauvegarde, la modification ne doit pas avoir lieu.
pub fn sauvegarder(dossier: &Dossier, fichier: &Path) -> Result<PathBuf, Erreur> {
    let copie = sauvegarder_a(&dossier.sauvegardes(), fichier, crate::maintenant())?;
    noter_origine(&dossier.sauvegardes(), &copie, fichier);
    Ok(copie)
}

/// Le registre des copies ; vide s'il manque ou ne se lit pas.
fn lire_index(dossier: &Path) -> BTreeMap<String, PathBuf> {
    fs::read_to_string(dossier.join(INDEX))
        .ok()
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_default()
}

/// Note d'où vient une copie. Un registre qui ne s'écrit pas ne fait pas échouer la sauvegarde : la copie existe,
/// elle ne pourra seulement pas se restaurer d'un clic.
fn noter_origine(dossier: &Path, copie: &Path, origine: &Path) {
    let _verrou = VERROU.lock().unwrap_or_else(PoisonError::into_inner);
    let Some(nom) = copie.file_name().map(|n| n.to_string_lossy().into_owned()) else {
        return;
    };
    let complet = fs::canonicalize(origine).unwrap_or_else(|_| origine.to_path_buf());
    let mut index = lire_index(dossier);
    index.insert(nom, complet);
    if let Ok(texte) = serde_json::to_string_pretty(&index) {
        let _ = donnees::ecrire_entier(&dossier.join(INDEX), texte.as_bytes());
    }
}

/// Une copie, telle que l'interface la montre.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Sauvegarde {
    /// Le nom de la copie dans `sauvegardes\`.
    pub nom: String,
    /// Le fichier d'où elle vient ; `None` si le registre ne le sait pas (copie d'une ancienne version).
    pub origine: Option<String>,
    pub taille: u64,
}

/// Les copies, la plus récente d'abord (leur nom commence par la date). Les versions gardées du programme
/// (`sauvegardes\versions\`) et les dossiers ne sont pas listés.
pub fn lister(dossier: &Dossier) -> Vec<Sauvegarde> {
    let racine = dossier.sauvegardes();
    let index = lire_index(&racine);
    let Ok(entrees) = fs::read_dir(&racine) else {
        return Vec::new();
    };
    let mut copies: Vec<Sauvegarde> = entrees
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_file()))
        .map(|e| e.file_name().to_string_lossy().into_owned())
        .filter(|nom| nom != INDEX)
        .map(|nom| Sauvegarde {
            taille: fs::metadata(racine.join(&nom)).map(|m| m.len()).unwrap_or(0),
            origine: index.get(&nom).map(|o| o.display().to_string()),
            nom,
        })
        .collect();
    copies.sort_by(|a, b| b.nom.cmp(&a.nom));
    copies
}

/// Remet la copie `nom` à sa place d'origine : l'état actuel est d'abord sauvegardé (s'il existe), puis la copie
/// est recopiée. Rien n'est supprimé. Renvoie le fichier restauré.
///
/// # Errors
/// Un nom qui n'est pas une copie de `sauvegardes\`, une origine inconnue, ou une copie qui échoue ; l'état actuel
/// n'a alors pas changé.
pub fn restaurer(dossier: &Dossier, nom: &str) -> Result<PathBuf, String> {
    let racine = dossier.sauvegardes();
    let simple = Path::new(nom).file_name().is_some_and(|n| n == nom);
    let copie = racine.join(nom);
    if !simple || nom == INDEX || !copie.is_file() {
        return Err(format!("« {nom} » n'est pas une sauvegarde"));
    }
    let origine = lire_index(&racine)
        .get(nom)
        .cloned()
        .ok_or_else(|| format!("on ne sait pas d'où vient « {nom} » : restaure-la à la main"))?;
    if origine.is_file() {
        sauvegarder(dossier, &origine)
            .map_err(|e| format!("l'état actuel n'a pas pu être sauvegardé : {e}"))?;
    }
    let contenu = fs::read(&copie).map_err(|e| format!("la sauvegarde ne se lit pas : {e}"))?;
    if let Some(parent) = origine.parent() {
        fs::create_dir_all(parent).map_err(|e| format!("{} : {e}", parent.display()))?;
    }
    donnees::ecrire_entier(&origine, &contenu).map_err(|e| e.to_string())?;
    journal::consigner(
        dossier,
        "restauration",
        &format!("{} remis depuis la sauvegarde {nom}", origine.display()),
    );
    Ok(origine)
}

/// [`sauvegarder`], dans un dossier et à une date donnés (tests).
///
/// # Errors
/// [`Erreur::AbsentASauvegarder`] si le fichier n'existe pas, [`Erreur::Fichier`] si la copie échoue.
pub fn sauvegarder_a(dossier: &Path, fichier: &Path, quand: OffsetDateTime) -> Result<PathBuf, Erreur> {
    if !fichier.is_file() {
        return Err(Erreur::AbsentASauvegarder(fichier.to_path_buf()));
    }
    fs::create_dir_all(dossier).map_err(|e| Erreur::fichier("création de", dossier, e))?;
    let nom = fichier
        .file_name()
        .map(|n| n.to_string_lossy().into_owned())
        .unwrap_or_default();
    // La forme ne demande que des champs qu'une date a toujours : le formatage ne peut pas échouer.
    let horodatage = quand.format(FORME_HORODATAGE).unwrap_or_default();
    let (base, extension) = match nom.rsplit_once('.') {
        Some((b, e)) if !b.is_empty() => (b.to_owned(), format!(".{e}")),
        _ => (nom.clone(), String::new()),
    };
    // Deux sauvegardes du même fichier dans la même seconde : un suffixe, jamais un écrasement.
    for n in 1u32.. {
        let suffixe = if n == 1 { String::new() } else { format!("_{n}") };
        let cible = dossier.join(format!("{horodatage}_{base}{suffixe}{extension}"));
        match copier_sans_ecraser(fichier, &cible) {
            Ok(()) => return Ok(cible),
            Err(e) if e.kind() == ErrorKind::AlreadyExists => {}
            Err(e) => return Err(Erreur::fichier("sauvegarde vers", cible, e)),
        }
    }
    unreachable!("la boucle ne se termine que par un retour")
}

/// Copie seulement si la cible n'existe pas encore, sans course possible entre le test et la création.
fn copier_sans_ecraser(source: &Path, cible: &Path) -> io::Result<()> {
    let mut sortie = OpenOptions::new().write(true).create_new(true).open(cible)?;
    let mut entree = fs::File::open(source)?;
    io::copy(&mut entree, &mut sortie)?;
    sortie.sync_all()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn date() -> OffsetDateTime {
        time::macros::datetime!(2026-11-21 19:05:00 +1)
    }

    #[test]
    fn la_copie_est_datee_et_identique() {
        let temp = tempfile::tempdir().unwrap();
        let fichier = temp.path().join("hosts.txt");
        fs::write(&fichier, "127.0.0.1 localhost").unwrap();
        let copie = sauvegarder_a(&temp.path().join("sauvegardes"), &fichier, date()).unwrap();
        assert!(
            copie.ends_with("2026-11-21_19-05-00_hosts.txt"),
            "{}",
            copie.display()
        );
        assert_eq!(fs::read(&copie).unwrap(), fs::read(&fichier).unwrap());
    }

    #[test]
    fn deux_copies_dans_la_meme_seconde_ne_s_ecrasent_pas() {
        let temp = tempfile::tempdir().unwrap();
        let fichier = temp.path().join("a.json");
        fs::write(&fichier, "1").unwrap();
        let dossier = temp.path().join("sauvegardes");
        let premiere = sauvegarder_a(&dossier, &fichier, date()).unwrap();
        fs::write(&fichier, "2").unwrap();
        let seconde = sauvegarder_a(&dossier, &fichier, date()).unwrap();
        assert!(
            seconde.ends_with("2026-11-21_19-05-00_a_2.json"),
            "{}",
            seconde.display()
        );
        assert_eq!(fs::read_to_string(premiere).unwrap(), "1");
        assert_eq!(fs::read_to_string(seconde).unwrap(), "2");
    }

    #[test]
    fn un_fichier_absent_est_refuse() {
        let temp = tempfile::tempdir().unwrap();
        let erreur = sauvegarder_a(temp.path(), &temp.path().join("absent"), date()).unwrap_err();
        assert!(matches!(erreur, Erreur::AbsentASauvegarder(_)));
    }

    #[test]
    fn un_fichier_sans_extension_garde_son_nom() {
        let temp = tempfile::tempdir().unwrap();
        let fichier = temp.path().join("hosts");
        fs::write(&fichier, "x").unwrap();
        let copie = sauvegarder_a(&temp.path().join("s"), &fichier, date()).unwrap();
        assert!(
            copie.ends_with("2026-11-21_19-05-00_hosts"),
            "{}",
            copie.display()
        );
    }

    #[test]
    fn une_sauvegarde_se_restaure_apres_une_copie_de_l_etat_actuel() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path().join("donnees"));
        dossier.preparer().unwrap();
        let fichier = dossier.racine().join("notes.txt");
        fs::write(&fichier, "ancien").unwrap();
        let copie = sauvegarder(&dossier, &fichier).unwrap();
        fs::write(&fichier, "nouveau").unwrap();
        let nom = copie.file_name().unwrap().to_string_lossy().into_owned();
        let liste = lister(&dossier);
        assert_eq!(liste.len(), 1);
        assert!(liste[0].origine.as_deref().unwrap().ends_with("notes.txt"));
        restaurer(&dossier, &nom).unwrap();
        assert_eq!(fs::read_to_string(&fichier).unwrap(), "ancien");
        // L'état d'avant la restauration est gardé : deux copies, aucune perdue.
        let apres = lister(&dossier);
        assert_eq!(apres.len(), 2);
        let contenus: Vec<String> = apres
            .iter()
            .map(|s| fs::read_to_string(dossier.sauvegardes().join(&s.nom)).unwrap())
            .collect();
        assert!(contenus.contains(&"nouveau".to_owned()));
        assert!(contenus.contains(&"ancien".to_owned()));
    }

    #[test]
    fn restaurer_refuse_ce_qui_n_est_pas_une_sauvegarde() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path().join("donnees"));
        dossier.preparer().unwrap();
        fs::write(dossier.sauvegardes().join("orpheline.txt"), "x").unwrap();
        for nom in ["../reglages.json", INDEX, "absente.txt", "orpheline.txt"] {
            assert!(restaurer(&dossier, nom).is_err(), "{nom}");
        }
    }
}
