//! Reprise des données d'une ancienne version de l'appli (autre dossier, autre format), une seule fois : l'ancien
//! dossier est d'abord copié dans `sauvegardes\reprise-<date>\`, l'appli reprend depuis cette copie, et l'ancien
//! dossier n'est jamais modifié. Ce qui n'a pas pu être repris est noté et dit (contrat, « Reprise des données »).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::{Dossier, journal};

/// Le témoin de la reprise, dans `sauvegardes\` : sa présence dit qu'elle a eu lieu.
pub const TEMOIN: &str = "reprise.json";

/// Ce que la reprise a fait.
#[derive(Debug, Clone, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Bilan {
    /// L'ancien dossier.
    pub ancien: String,
    /// La copie de l'ancien dossier, d'où l'appli a repris ; vide s'il n'y avait rien.
    pub copie: String,
    /// Ce qui a été repris, une ligne par élément.
    pub repris: Vec<String>,
    /// Ce qui n'a pas pu l'être, avec la raison.
    pub non_repris: Vec<String>,
}

/// Le bilan de la reprise déjà faite ; `None` si elle n'a pas eu lieu.
pub fn bilan(dossier: &Dossier) -> Option<Bilan> {
    let texte = fs::read_to_string(dossier.sauvegardes().join(TEMOIN)).ok()?;
    serde_json::from_str(&texte).ok()
}

/// Copie un dossier et tout ce qu'il contient. Les liens ne sont pas suivis.
fn copier_dossier(de: &Path, vers: &Path) -> io::Result<()> {
    fs::create_dir_all(vers)?;
    for entree in fs::read_dir(de)? {
        let entree = entree?;
        let genre = entree.file_type()?;
        let cible = vers.join(entree.file_name());
        if genre.is_dir() {
            copier_dossier(&entree.path(), &cible)?;
        } else if genre.is_file() {
            fs::copy(entree.path(), &cible)?;
        }
    }
    Ok(())
}

/// Reprend une seule fois les données de l'ancien dossier `ancien` : le copie, appelle `reprendre` sur la copie,
/// puis note le bilan au journal et dans le témoin. Rien si la reprise a déjà eu lieu ; un bilan vide si l'ancien
/// dossier n'existe pas.
///
/// `reprendre` reçoit la copie et le dossier de données, et remplit `repris` et `non_repris`.
///
/// # Errors
/// La copie de l'ancien dossier ou l'écriture du témoin échouent : rien n'est repris, et la reprise sera retentée
/// au prochain lancement.
pub fn une_fois(
    dossier: &Dossier,
    ancien: &Path,
    reprendre: impl FnOnce(&Path, &Dossier, &mut Bilan),
) -> Result<Option<Bilan>, String> {
    if bilan(dossier).is_some() {
        return Ok(None);
    }
    let mut bilan = Bilan {
        ancien: ancien.display().to_string(),
        ..Bilan::default()
    };
    if ancien.is_dir() {
        let horodatage = crate::maintenant().unix_timestamp();
        let copie: PathBuf = dossier.sauvegardes().join(format!("reprise-{horodatage}"));
        copier_dossier(ancien, &copie)
            .map_err(|e| format!("l'ancien dossier n'a pas pu être copié : {e}"))?;
        bilan.copie = copie.display().to_string();
        reprendre(&copie, dossier, &mut bilan);
    }
    fs::create_dir_all(dossier.sauvegardes()).map_err(|e| e.to_string())?;
    let texte = serde_json::to_string_pretty(&bilan).map_err(|e| e.to_string())?;
    crate::donnees::ecrire_entier(&dossier.sauvegardes().join(TEMOIN), texte.as_bytes())
        .map_err(|e| e.to_string())?;
    let resume = if bilan.copie.is_empty() {
        format!("rien à reprendre ({} n'existe pas)", bilan.ancien)
    } else {
        format!(
            "{} élément(s) repris, {} non repris, depuis une copie de {}",
            bilan.repris.len(),
            bilan.non_repris.len(),
            bilan.ancien
        )
    };
    journal::consigner(dossier, "reprise", &resume);
    for ligne in &bilan.non_repris {
        journal::consigner(dossier, "reprise", &format!("non repris : {ligne}"));
    }
    Ok(Some(bilan))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_reprise_part_d_une_copie_et_ne_se_fait_qu_une_fois() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path().join("donnees"));
        dossier.preparer().unwrap();
        let ancien = temp.path().join("ancien");
        fs::create_dir_all(ancien.join("sous")).unwrap();
        fs::write(ancien.join("config.ini"), "a=1").unwrap();
        fs::write(ancien.join("sous").join("illisible.bin"), "?").unwrap();
        let bilan = une_fois(&dossier, &ancien, |copie, _, bilan| {
            assert!(copie.join("config.ini").is_file());
            // La copie peut être modifiée : l'ancien dossier, jamais.
            fs::write(copie.join("config.ini"), "touche").unwrap();
            bilan.repris.push("config.ini".to_owned());
            bilan
                .non_repris
                .push("sous/illisible.bin : format inconnu".to_owned());
        })
        .unwrap()
        .unwrap();
        assert_eq!(fs::read_to_string(ancien.join("config.ini")).unwrap(), "a=1");
        assert_eq!(bilan.repris, ["config.ini"]);
        assert_eq!(super::bilan(&dossier), Some(bilan));
        let encore = une_fois(&dossier, &ancien, |_, _, _| panic!("déjà faite")).unwrap();
        assert_eq!(encore, None);
    }

    #[test]
    fn sans_ancien_dossier_le_bilan_est_vide() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path().join("donnees"));
        dossier.preparer().unwrap();
        let bilan = une_fois(&dossier, &temp.path().join("absent"), |_, _, _| {
            panic!("rien à reprendre")
        })
        .unwrap()
        .unwrap();
        assert!(bilan.copie.is_empty() && bilan.repris.is_empty());
    }
}
