//! Le partage de la carte graphique (contrat, « Modules en option ») : l'appli qui charge un modèle déclare la mémoire
//! vidéo qu'elle occupe dans le dossier commun, et peut savoir si une autre appli en occupe déjà. Une déclaration
//! disparaît quand l'appli la lâche, ou quand son processus n'existe plus.

use std::fs;

use serde::{Deserialize, Serialize};

use crate::commun::Commun;

/// Une appli qui occupe de la mémoire vidéo.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Occupation {
    pub appli: String,
    pub processus: u32,
    /// En mégaoctets.
    pub mo: u64,
}

fn lire(commun: &Commun) -> Vec<Occupation> {
    fs::read_to_string(commun.carte_graphique())
        .ok()
        .and_then(|texte| serde_json::from_str(&texte).ok())
        .unwrap_or_default()
}

fn ecrire(commun: &Commun, appli: &str, occupations: &[Occupation]) -> Result<(), String> {
    let _verrou = commun.verrouiller(appli)?;
    let texte = serde_json::to_string_pretty(occupations).map_err(|e| e.to_string())?;
    crate::donnees::ecrire_entier(&commun.carte_graphique(), texte.as_bytes()).map_err(|e| e.to_string())
}

/// Les occupations dont le processus tourne encore.
fn vivantes(commun: &Commun, vivant: &impl Fn(u32) -> bool) -> Vec<Occupation> {
    lire(commun).into_iter().filter(|o| vivant(o.processus)).collect()
}

/// La déclaration de l'appli, retirée quand elle est lâchée.
#[derive(Debug)]
pub struct Declaration {
    commun: Commun,
    appli: String,
    processus: u32,
}

impl Drop for Declaration {
    fn drop(&mut self) {
        let restantes: Vec<Occupation> = lire(&self.commun)
            .into_iter()
            .filter(|o| !(o.appli == self.appli && o.processus == self.processus))
            .collect();
        let _ = ecrire(&self.commun, &self.appli, &restantes);
    }
}

/// Déclare que ce processus de l'appli occupe `mo` mégaoctets de mémoire vidéo.
///
/// # Errors
/// Le verrou ou l'écriture du dossier commun échouent.
pub fn declarer(commun: &Commun, appli: &str, mo: u64) -> Result<Declaration, String> {
    declarer_pour(commun, appli, std::process::id(), mo, &systeme::processus_vivant)
}

fn declarer_pour(
    commun: &Commun,
    appli: &str,
    processus: u32,
    mo: u64,
    vivant: &impl Fn(u32) -> bool,
) -> Result<Declaration, String> {
    let mut occupations: Vec<Occupation> = vivantes(commun, vivant)
        .into_iter()
        .filter(|o| !(o.appli == appli && o.processus == processus))
        .collect();
    occupations.push(Occupation {
        appli: appli.to_owned(),
        processus,
        mo,
    });
    ecrire(commun, appli, &occupations)?;
    Ok(Declaration {
        commun: commun.clone(),
        appli: appli.to_owned(),
        processus,
    })
}

/// Les autres applis qui occupent la mémoire vidéo en ce moment : à dire à l'utilisateur avant de charger un modèle.
pub fn autres(commun: &Commun, appli: &str) -> Vec<Occupation> {
    vivantes(commun, &systeme::processus_vivant)
        .into_iter()
        .filter(|o| o.appli != appli)
        .collect()
}

#[cfg(windows)]
mod systeme {
    use windows::Win32::Foundation::{CloseHandle, STILL_ACTIVE};
    use windows::Win32::System::Threading::{
        GetExitCodeProcess, OpenProcess, PROCESS_QUERY_LIMITED_INFORMATION,
    };

    /// Le processus tourne-t-il encore ?
    #[allow(unsafe_code)]
    pub fn processus_vivant(processus: u32) -> bool {
        // SAFETY : le handle ouvert ici ne sert que dans ce bloc, puis il est fermé ; le code de sortie est écrit
        // dans une variable locale valide.
        unsafe {
            let Ok(handle) = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, false, processus) else {
                return false;
            };
            let mut code = 0u32;
            let vivant =
                GetExitCodeProcess(handle, &raw mut code).is_ok() && code == STILL_ACTIVE.0.cast_unsigned();
            let _ = CloseHandle(handle);
            vivant
        }
    }
}

#[cfg(not(windows))]
mod systeme {
    /// Hors de Windows (tests) : un processus est vivant si /proc le connaît.
    pub fn processus_vivant(processus: u32) -> bool {
        std::path::Path::new(&format!("/proc/{processus}")).exists()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn une_declaration_se_voit_puis_disparait() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        let tous_vivants = |_: u32| true;
        let studio = declarer_pour(&commun, "Aria-Studio", 100, 6000, &tous_vivants).unwrap();
        let _md = declarer_pour(&commun, "Aria-MD", 200, 2000, &tous_vivants).unwrap();
        let vues: Vec<String> = vivantes(&commun, &tous_vivants)
            .into_iter()
            .map(|o| o.appli)
            .collect();
        assert_eq!(vues, ["Aria-Studio", "Aria-MD"]);
        drop(studio);
        let vues: Vec<String> = vivantes(&commun, &tous_vivants)
            .into_iter()
            .map(|o| o.appli)
            .collect();
        assert_eq!(vues, ["Aria-MD"]);
    }

    #[test]
    fn un_processus_arrete_ne_compte_plus() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        let _vieille = declarer_pour(&commun, "Aria-JDR", 300, 4000, &|_| true).unwrap();
        assert!(vivantes(&commun, &|p| p != 300).is_empty());
    }
}
