//! Les moteurs locaux partagés (décision 0009) : une version par dossier, `moteurs\<moteur>\<version>\` du dossier
//! commun, qui coexistent côte à côte. Chaque appli déclare la plage de versions qu'elle accepte ; une mise à jour
//! n'est proposée que dans cette plage ; après une mise à jour, un essai vérifie que le moteur démarre et répond, et la
//! version précédente reste en place s'il échoue. Le registre `usage.json` dit quelle appli utilise quelle version ;
//! rien ne s'efface sans demande de l'utilisateur, à qui l'on dit d'abord quelles applis s'en servent.

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::commun::Commun;
use crate::mises_a_jour::{Version, ecrire_version, lire_version};

/// Les versions acceptées d'un moteur : de `minimum` compris à `maximum` exclu (`1.4.0` ≤ v < `2.0.0`, par exemple).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Plage {
    pub minimum: Version,
    pub maximum: Version,
}

impl Plage {
    /// Toutes les versions de la même version majeure, à partir de `minimum` : une version majeure attend une version
    /// de l'appli qui l'a validée.
    pub fn majeure(minimum: Version) -> Self {
        Self {
            minimum,
            maximum: (minimum.0 + 1, 0, 0),
        }
    }

    pub fn contient(&self, version: Version) -> bool {
        self.minimum <= version && version < self.maximum
    }
}

/// Un nom de moteur sûr pour un nom de dossier : lettres minuscules, chiffres et tirets.
fn nom_valide(moteur: &str) -> bool {
    !moteur.is_empty()
        && moteur.len() <= 40
        && moteur
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || c == b'-')
}

fn dossier_moteur(commun: &Commun, moteur: &str) -> Result<PathBuf, String> {
    if !nom_valide(moteur) {
        return Err(format!("nom de moteur refusé : « {moteur} »"));
    }
    Ok(commun.moteurs().join(moteur))
}

/// Les versions installées d'un moteur, de la plus ancienne à la plus récente. Un dossier en cours d'installation
/// (`.tmp`) ou dont le nom n'est pas une version n'est pas compté.
pub fn versions(commun: &Commun, moteur: &str) -> Vec<Version> {
    let Ok(dossier) = dossier_moteur(commun, moteur) else {
        return Vec::new();
    };
    let mut trouvees: Vec<Version> = fs::read_dir(dossier)
        .into_iter()
        .flatten()
        .flatten()
        .filter(|e| e.file_type().is_ok_and(|t| t.is_dir()))
        .filter_map(|e| lire_version(&e.file_name().to_string_lossy()))
        .collect();
    trouvees.sort_unstable();
    trouvees
}

/// La version la plus récente installée dans la plage, et son dossier.
pub fn choisir(commun: &Commun, moteur: &str, plage: Plage) -> Option<(Version, PathBuf)> {
    let version = versions(commun, moteur)
        .into_iter()
        .rev()
        .find(|v| plage.contient(*v))?;
    let dossier = dossier_moteur(commun, moteur).ok()?.join(ecrire_version(version));
    Some((version, dossier))
}

/// Le dossier d'où lancer le moteur. Une appli élevée ne lance rien depuis le dossier commun, que n'importe quel
/// programme de l'utilisateur peut modifier : elle garde ses moteurs dans son propre dossier protégé.
///
/// # Errors
/// L'appli tourne élevée, ou aucune version installée n'est dans la plage.
pub fn a_lancer(
    commun: &Commun,
    moteur: &str,
    plage: Plage,
    eleve: bool,
) -> Result<(Version, PathBuf), String> {
    if eleve {
        return Err(
            "une appli élevée ne lance pas de moteur depuis le dossier commun : elle garde les siens dans son \
             propre dossier protégé"
                .to_owned(),
        );
    }
    choisir(commun, moteur, plage).ok_or_else(|| {
        format!(
            "aucune version de {moteur} entre {} et {} (exclue) n'est installée",
            ecrire_version(plage.minimum),
            ecrire_version(plage.maximum)
        )
    })
}

/// Ce qu'une mise à jour a donné.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Issue {
    /// La nouvelle version est installée et a passé l'essai.
    Installee(PathBuf),
    /// Hors de la plage de l'appli : rien n'a été fait.
    HorsPlage,
    /// Déjà installée : rien n'a été fait.
    DejaLa(PathBuf),
    /// L'essai a échoué : la nouvelle version est retirée, la précédente reste en place.
    EssaiRate(String),
}

/// Installe la version `nouvelle` d'un moteur à côté des autres, si elle est dans la plage : `installer` remplit le
/// dossier de travail (téléchargement vérifié, décompression), puis `essayer` vérifie que le moteur démarre et
/// répond. Rien n'est remplacé : les versions précédentes restent, et un essai raté retire seulement la nouvelle.
///
/// # Errors
/// Le verrou du dossier commun, l'installation ou le rangement échouent ; rien n'a alors changé.
pub fn mettre_a_jour(
    commun: &Commun,
    appli: &str,
    moteur: &str,
    plage: Plage,
    nouvelle: Version,
    installer: impl FnOnce(&Path) -> Result<(), String>,
    essayer: impl FnOnce(&Path) -> Result<(), String>,
) -> Result<Issue, String> {
    if !plage.contient(nouvelle) {
        return Ok(Issue::HorsPlage);
    }
    let dossier = dossier_moteur(commun, moteur)?;
    let cible = dossier.join(ecrire_version(nouvelle));
    if cible.is_dir() {
        return Ok(Issue::DejaLa(cible));
    }
    let _verrou = commun.verrouiller(appli)?;
    let travail = dossier.join(format!("{}.tmp", ecrire_version(nouvelle)));
    if travail.exists() {
        fs::remove_dir_all(&travail).map_err(|e| format!("{} : {e}", travail.display()))?;
    }
    fs::create_dir_all(&travail).map_err(|e| format!("{} : {e}", travail.display()))?;
    if let Err(e) = installer(&travail) {
        let _ = fs::remove_dir_all(&travail);
        return Err(e);
    }
    fs::rename(&travail, &cible).map_err(|e| {
        let _ = fs::remove_dir_all(&travail);
        format!("{} : {e}", cible.display())
    })?;
    match essayer(&cible) {
        Ok(()) => Ok(Issue::Installee(cible)),
        Err(raison) => {
            // Seule la version qui vient d'être posée part : les précédentes n'ont pas bougé.
            let _ = fs::remove_dir_all(&cible);
            Ok(Issue::EssaiRate(raison))
        }
    }
}

/// Le registre commun : moteur → version → applis, et modèle → applis.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct Registre {
    #[serde(default)]
    pub moteurs: BTreeMap<String, BTreeMap<String, BTreeSet<String>>>,
    #[serde(default)]
    pub modeles: BTreeMap<String, BTreeSet<String>>,
}

/// Le registre ; vide s'il manque ou ne se lit pas.
pub fn registre(commun: &Commun) -> Registre {
    fs::read_to_string(commun.registre())
        .ok()
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_default()
}

fn modifier_registre(
    commun: &Commun,
    appli: &str,
    modifier: impl FnOnce(&mut Registre),
) -> Result<(), String> {
    let _verrou = commun.verrouiller(appli)?;
    let mut lu = registre(commun);
    modifier(&mut lu);
    let texte = serde_json::to_string_pretty(&lu).map_err(|e| e.to_string())?;
    crate::donnees::ecrire_entier(&commun.registre(), texte.as_bytes()).map_err(|e| e.to_string())
}

/// Note que l'appli utilise cette version du moteur (et plus les autres).
///
/// # Errors
/// Le verrou ou l'écriture du registre échouent.
pub fn declarer_moteur(commun: &Commun, appli: &str, moteur: &str, version: Version) -> Result<(), String> {
    dossier_moteur(commun, moteur)?;
    modifier_registre(commun, appli, |registre| {
        let versions = registre.moteurs.entry(moteur.to_owned()).or_default();
        for applis in versions.values_mut() {
            applis.remove(appli);
        }
        versions.retain(|_, applis| !applis.is_empty());
        versions
            .entry(ecrire_version(version))
            .or_default()
            .insert(appli.to_owned());
    })
}

/// Note que l'appli utilise ce modèle (nom de fichier dans `modeles\`).
///
/// # Errors
/// Le verrou ou l'écriture du registre échouent.
pub fn declarer_modele(commun: &Commun, appli: &str, modele: &str) -> Result<(), String> {
    modifier_registre(commun, appli, |registre| {
        registre
            .modeles
            .entry(modele.to_owned())
            .or_default()
            .insert(appli.to_owned());
    })
}

/// Les applis qui utilisent cette version du moteur.
pub fn utilisateurs(commun: &Commun, moteur: &str, version: Version) -> Vec<String> {
    registre(commun)
        .moteurs
        .get(moteur)
        .and_then(|versions| versions.get(&ecrire_version(version)))
        .map(|applis| applis.iter().cloned().collect())
        .unwrap_or_default()
}

/// Efface une version d'un moteur, sur demande de l'utilisateur. `confirme` dit qu'il a vu la liste des applis qui
/// s'en servent (voir [`utilisateurs`]) : sans elle, rien n'est effacé si une autre appli que `appli` s'en sert.
///
/// # Errors
/// Une autre appli s'en sert et l'utilisateur n'a pas confirmé, ou l'effacement échoue.
pub fn effacer(
    commun: &Commun,
    appli: &str,
    moteur: &str,
    version: Version,
    confirme: bool,
) -> Result<(), String> {
    let autres: Vec<String> = utilisateurs(commun, moteur, version)
        .into_iter()
        .filter(|a| a != appli)
        .collect();
    if !autres.is_empty() && !confirme {
        return Err(format!(
            "{moteur} {} sert aussi à : {}",
            ecrire_version(version),
            autres.join(", ")
        ));
    }
    let dossier = dossier_moteur(commun, moteur)?.join(ecrire_version(version));
    let verrou = commun.verrouiller(appli)?;
    if dossier.is_dir() {
        fs::remove_dir_all(&dossier).map_err(|e| format!("{} : {e}", dossier.display()))?;
    }
    drop(verrou);
    modifier_registre(commun, appli, |registre| {
        if let Some(versions) = registre.moteurs.get_mut(moteur) {
            versions.remove(&ecrire_version(version));
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn poser(commun: &Commun, moteur: &str, version: &str) {
        fs::create_dir_all(commun.moteurs().join(moteur).join(version)).unwrap();
    }

    #[test]
    fn la_plage_garde_la_version_majeure() {
        let plage = Plage::majeure((1, 4, 0));
        assert!(plage.contient((1, 4, 0)) && plage.contient((1, 9, 3)));
        assert!(!plage.contient((1, 3, 9)) && !plage.contient((2, 0, 0)));
    }

    #[test]
    fn le_choix_prend_la_plus_recente_dans_la_plage() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        for v in ["1.2.0", "1.10.1", "2.0.0", "pas-une-version", "1.11.0.tmp"] {
            poser(&commun, "llama", v);
        }
        assert_eq!(versions(&commun, "llama"), [(1, 2, 0), (1, 10, 1), (2, 0, 0)]);
        let (version, dossier) = choisir(&commun, "llama", Plage::majeure((1, 0, 0))).unwrap();
        assert_eq!(version, (1, 10, 1));
        assert!(dossier.ends_with("1.10.1"));
        assert!(choisir(&commun, "llama", Plage::majeure((3, 0, 0))).is_none());
        assert!(a_lancer(&commun, "llama", Plage::majeure((1, 0, 0)), true).is_err());
        assert!(versions(&commun, "../x").is_empty());
    }

    #[test]
    fn une_mise_a_jour_hors_plage_ou_ratee_ne_touche_a_rien() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        poser(&commun, "llama", "1.2.0");
        let plage = Plage::majeure((1, 0, 0));
        let hors = mettre_a_jour(
            &commun,
            "Aria-Essai",
            "llama",
            plage,
            (2, 0, 0),
            |_| Ok(()),
            |_| Ok(()),
        )
        .unwrap();
        assert_eq!(hors, Issue::HorsPlage);
        let ratee = mettre_a_jour(
            &commun,
            "Aria-Essai",
            "llama",
            plage,
            (1, 3, 0),
            |d| fs::write(d.join("llama.exe"), "x").map_err(|e| e.to_string()),
            |_| Err("ne répond pas".to_owned()),
        )
        .unwrap();
        assert_eq!(ratee, Issue::EssaiRate("ne répond pas".to_owned()));
        assert_eq!(versions(&commun, "llama"), [(1, 2, 0)]);
        let reussie = mettre_a_jour(
            &commun,
            "Aria-Essai",
            "llama",
            plage,
            (1, 3, 0),
            |d| fs::write(d.join("llama.exe"), "x").map_err(|e| e.to_string()),
            |d| {
                if d.join("llama.exe").is_file() {
                    Ok(())
                } else {
                    Err("absent".to_owned())
                }
            },
        )
        .unwrap();
        assert!(matches!(reussie, Issue::Installee(_)));
        assert_eq!(versions(&commun, "llama"), [(1, 2, 0), (1, 3, 0)]);
    }

    #[test]
    fn rien_ne_s_efface_sans_dire_qui_s_en_sert() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        poser(&commun, "llama", "1.2.0");
        declarer_moteur(&commun, "Aria-Studio", "llama", (1, 2, 0)).unwrap();
        declarer_moteur(&commun, "Aria-MD", "llama", (1, 2, 0)).unwrap();
        declarer_modele(&commun, "Aria-MD", "qwen.gguf").unwrap();
        assert_eq!(
            utilisateurs(&commun, "llama", (1, 2, 0)),
            ["Aria-MD", "Aria-Studio"]
        );
        let refus = effacer(&commun, "Aria-MD", "llama", (1, 2, 0), false).unwrap_err();
        assert!(refus.contains("Aria-Studio"), "{refus}");
        assert!(commun.moteurs().join("llama").join("1.2.0").is_dir());
        effacer(&commun, "Aria-MD", "llama", (1, 2, 0), true).unwrap();
        assert!(!commun.moteurs().join("llama").join("1.2.0").exists());
        assert!(utilisateurs(&commun, "llama", (1, 2, 0)).is_empty());
        assert!(registre(&commun).modeles["qwen.gguf"].contains("Aria-MD"));
    }

    #[test]
    fn une_appli_ne_declare_qu_une_version_a_la_fois() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        declarer_moteur(&commun, "Aria-MD", "llama", (1, 2, 0)).unwrap();
        declarer_moteur(&commun, "Aria-MD", "llama", (1, 3, 0)).unwrap();
        assert!(utilisateurs(&commun, "llama", (1, 2, 0)).is_empty());
        assert_eq!(utilisateurs(&commun, "llama", (1, 3, 0)), ["Aria-MD"]);
    }
}
