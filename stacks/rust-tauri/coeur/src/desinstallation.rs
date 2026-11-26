//! Désinstallation, par le bouton « Désinstaller » des Paramètres : retirer ce que l'appli a écrit hors de son
//! dossier de données (démarrage automatique, raccourci du Bureau s'il pointe vers elle), puis `programme\`. Les
//! données restent : les supprimer est un geste de l'utilisateur.
//!
//! Un exe qui tourne ne peut pas supprimer son propre fichier. Quand l'appli tourne depuis `programme\`, elle se
//! copie dans `cache\desinstallation\` et lance cette copie avec [`ARGUMENT_FINIR`] : la copie attend la fin de
//! l'appli, vide `programme\`, puis s'arrête. Elle ne peut pas se supprimer non plus : le lancement suivant retire
//! son dossier ([`nettoyer_aide`]) ; sinon elle reste dans le dossier de données, avec le reste.
//!
//! Ici, les règles qui se testent partout ; le registre est dans [`crate::demarrage`], la lecture du raccourci et
//! l'attente de la fin de l'appli dans l'exe.

use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use crate::installation::{self, LISTE, Place, SUFFIXE_ANCIEN, SUFFIXE_NOUVEAU};

/// Argument de la copie de l'exe qui finit la désinstallation, suivi du numéro du processus de l'appli à attendre.
pub const ARGUMENT_FINIR: &str = "--finir-desinstallation";

/// Le dossier du cache où l'exe se copie pour finir la désinstallation.
pub const DOSSIER_AIDE: &str = "desinstallation";

/// Pause entre deux essais de [`vider_programme_en_attendant`].
const PAUSE: Duration = Duration::from_millis(250);

/// Ce que la désinstallation fait de `programme\`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Programme {
    /// Pas d'appli installée (mode portable) : rien à supprimer.
    Absent,
    /// Une appli est installée, mais l'exe en cours tourne ailleurs : `programme\` se supprime tout de suite.
    Maintenant,
    /// L'exe en cours vient de `programme\` : une copie de l'exe le supprime une fois l'appli fermée.
    ALaFermeture,
}

/// Ce que devient `programme\`, d'après l'endroit d'où tourne l'exe.
pub fn sort_de_programme(exe: &Path, programme: &Path) -> Programme {
    if !programme.is_dir() {
        return Programme::Absent;
    }
    match installation::place_de(exe, programme) {
        Place::Installee => Programme::ALaFermeture,
        Place::Ailleurs => Programme::Maintenant,
    }
}

/// Le raccourci pointe-t-il vers l'appli ? Sa cible est comparée, sans tenir compte de la casse, à l'exe installé et
/// à l'exe en cours : un raccourci vers autre chose n'est jamais retiré.
pub fn raccourci_a_nous(cible: &Path, exes: &[&Path]) -> bool {
    if cible.as_os_str().is_empty() {
        return false;
    }
    let normal = |p: &Path| {
        let complet = fs::canonicalize(p).unwrap_or_else(|_| p.to_path_buf());
        complet.to_string_lossy().to_lowercase()
    };
    let cible = normal(cible);
    exes.iter().any(|&exe| normal(exe) == cible)
}

/// Ce que [`vider_programme`] a fait.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Bilan {
    /// Les fichiers de l'appli supprimés.
    pub supprimes: Vec<String>,
    /// Les fichiers de l'appli encore là : verrouillés, ou la liste, gardée tant qu'il en reste un.
    pub restants: Vec<String>,
    /// Ce que l'appli n'a pas installé (fichier ajouté à la main, dossier) : gardé.
    pub gardes: Vec<String>,
    /// `programme\` a été retiré.
    pub dossier_retire: bool,
}

impl Bilan {
    /// Le bilan en une ligne, pour le journal et l'interface ; une erreur si des fichiers de l'appli restent.
    ///
    /// # Errors
    /// Des fichiers de l'appli n'ont pas pu être supprimés : la ligne les nomme.
    pub fn resume(&self, programme: &Path) -> Result<String, String> {
        if !self.restants.is_empty() {
            return Err(format!(
                "pas entièrement supprimé, encore là : {} (dans {})",
                self.restants.join(", "),
                programme.display()
            ));
        }
        if !self.gardes.is_empty() {
            return Ok(format!(
                "fichiers de l'appli supprimés ; gardé, car ajouté hors de l'appli : {} (dans {})",
                self.gardes.join(", "),
                programme.display()
            ));
        }
        if self.dossier_retire {
            return Ok(format!("{} supprimé", programme.display()));
        }
        Ok("rien à supprimer".to_owned())
    }
}

/// Les noms de fichiers que l'appli a installés : ceux de `fichiers.txt`, et la liste elle-même. Sans liste lisible,
/// l'exe de l'appli et la liste seulement.
fn noms_installes(programme: &Path) -> Vec<String> {
    let mut noms = installation::lire_liste(programme)
        .ok()
        .flatten()
        .unwrap_or_else(|| vec![installation::nom_exe()]);
    noms.push(LISTE.to_owned());
    noms
}

/// Le fichier vient-il de l'appli : un nom installé, ou le reste d'un remplacement (« .ancien », « .nouveau ») ?
fn est_a_nous(nom: &str, noms: &[String]) -> bool {
    let nom = nom.to_lowercase();
    let base = nom
        .strip_suffix(SUFFIXE_ANCIEN)
        .or_else(|| nom.strip_suffix(SUFFIXE_NOUVEAU))
        .unwrap_or(nom.as_str());
    let correspond = |n: &String| n.eq_ignore_ascii_case(&nom) || n.eq_ignore_ascii_case(base);
    noms.iter().any(correspond)
}

/// Supprime de `programme\` ce que l'appli y a installé, et rien d'autre : un fichier ajouté à la main ou un dossier
/// restent, et le bilan les nomme. `fichiers.txt` part en dernier, quand tout le reste est parti : tant qu'un
/// fichier est verrouillé, la liste dit encore ce qui est à l'appli. Le dossier n'est retiré que s'il est vide.
pub fn vider_programme(programme: &Path) -> Bilan {
    let mut bilan = Bilan::default();
    let Ok(entrees) = fs::read_dir(programme) else {
        return bilan;
    };
    let noms = noms_installes(programme);
    let mut liste = None;
    for entree in entrees.flatten() {
        let nom = entree.file_name().to_string_lossy().into_owned();
        // Un lien ou un dossier n'est jamais suivi ni supprimé.
        let fichier = entree.file_type().is_ok_and(|t| t.is_file());
        if !fichier || !est_a_nous(&nom, &noms) {
            bilan.gardes.push(nom);
        } else if nom.eq_ignore_ascii_case(LISTE) {
            liste = Some((nom, entree.path()));
        } else if fs::remove_file(entree.path()).is_ok() {
            bilan.supprimes.push(nom);
        } else {
            bilan.restants.push(nom);
        }
    }
    if let Some((nom, chemin)) = liste {
        if bilan.restants.is_empty() && fs::remove_file(&chemin).is_ok() {
            bilan.supprimes.push(nom);
        } else {
            bilan.restants.push(nom);
        }
    }
    bilan.supprimes.sort();
    bilan.restants.sort();
    bilan.gardes.sort();
    let vide = bilan.restants.is_empty() && bilan.gardes.is_empty();
    bilan.dossier_retire = vide && fs::remove_dir(programme).is_ok();
    bilan
}

/// [`vider_programme`], repris tant qu'un fichier de l'appli reste verrouillé (l'exe qui vient de se fermer), au plus
/// pendant `delai`.
pub fn vider_programme_en_attendant(programme: &Path, delai: Duration) -> Bilan {
    let fin = Instant::now() + delai;
    let mut supprimes = Vec::new();
    loop {
        let mut bilan = vider_programme(programme);
        supprimes.append(&mut bilan.supprimes);
        if bilan.restants.is_empty() || Instant::now() >= fin {
            supprimes.sort();
            bilan.supprimes = supprimes;
            return bilan;
        }
        thread::sleep(PAUSE);
    }
}

/// Le numéro du processus à attendre, si l'exe a été lancé pour finir une désinstallation.
pub fn processus_a_attendre<I, S>(arguments: I) -> Option<u32>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    let mut suite = arguments.into_iter();
    while let Some(argument) = suite.next() {
        if argument.as_ref().eq_ignore_ascii_case(ARGUMENT_FINIR) {
            return suite.next().and_then(|n| n.as_ref().parse().ok());
        }
    }
    None
}

/// Le dossier de la copie qui finit la désinstallation.
pub fn dossier_aide(cache: &Path) -> PathBuf {
    cache.join(DOSSIER_AIDE)
}

/// L'exe en cours est-il la copie rangée dans le cache ? Seule elle finit une désinstallation : un exe lancé
/// ailleurs avec [`ARGUMENT_FINIR`] ne supprime rien.
pub fn est_l_aide(exe: &Path, cache: &Path) -> bool {
    installation::place_de(exe, &dossier_aide(cache)) == Place::Installee
}

/// Copie l'exe dans `cache\desinstallation\` (contenu seul) et renvoie le chemin de la copie, à lancer avec
/// [`ARGUMENT_FINIR`].
///
/// # Errors
/// Une copie qui échoue.
pub fn preparer_aide(exe: &Path, cache: &Path) -> io::Result<PathBuf> {
    let dossier = dossier_aide(cache);
    fs::create_dir_all(&dossier)?;
    let copie = dossier.join(installation::nom_exe());
    installation::copier_contenu(exe, &copie)?;
    Ok(copie)
}

/// Retire la copie laissée par une désinstallation précédente ; encore utilisée, elle le sera au lancement suivant.
pub fn nettoyer_aide(cache: &Path) {
    let dossier = dossier_aide(cache);
    if dossier.is_dir() {
        // Encore verrouillée : le prochain lancement la retirera.
        let _ = fs::remove_dir_all(dossier);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::installation::{INFOS_BUILD, nom_exe};

    /// Un `programme\` installé : l'exe, BUILDINFO.txt et la liste.
    fn installer_dans(programme: &Path) {
        fs::create_dir_all(programme).unwrap();
        fs::write(programme.join(nom_exe()), "exe").unwrap();
        fs::write(programme.join(INFOS_BUILD), "infos").unwrap();
        let liste = [nom_exe(), INFOS_BUILD.to_owned()].join("\n");
        fs::write(programme.join(LISTE), liste).unwrap();
    }

    #[test]
    fn vider_supprime_les_fichiers_de_l_appli_et_le_dossier() {
        let temp = tempfile::tempdir().unwrap();
        let programme = temp.path().join("programme");
        installer_dans(&programme);
        let ancien = programme.join(format!("{}.ancien", nom_exe()));
        fs::write(&ancien, "vieil exe").unwrap();
        let bilan = vider_programme(&programme);
        assert!(bilan.restants.is_empty(), "{bilan:?}");
        assert!(bilan.gardes.is_empty(), "{bilan:?}");
        assert_eq!(bilan.supprimes.len(), 4, "{bilan:?}");
        assert!(bilan.dossier_retire);
        assert!(!programme.exists());
        assert!(bilan.resume(&programme).is_ok());
    }

    #[test]
    fn vider_garde_ce_que_l_appli_n_a_pas_installe() {
        let temp = tempfile::tempdir().unwrap();
        let programme = temp.path().join("programme");
        installer_dans(&programme);
        fs::write(programme.join("notes.txt"), "à moi").unwrap();
        let sous_dossier = programme.join("sous-dossier");
        fs::create_dir_all(&sous_dossier).unwrap();
        fs::write(sous_dossier.join(nom_exe()), "autre").unwrap();
        let bilan = vider_programme(&programme);
        let gardes = ["notes.txt", "sous-dossier"];
        assert_eq!(bilan.gardes, gardes);
        assert!(!bilan.dossier_retire);
        let notes = fs::read_to_string(programme.join("notes.txt")).unwrap();
        assert_eq!(notes, "à moi");
        assert!(sous_dossier.join(nom_exe()).is_file());
        assert!(!programme.join(nom_exe()).exists());
        let resume = bilan.resume(&programme).unwrap();
        assert!(resume.contains("notes.txt"), "{resume}");
    }

    #[test]
    fn vider_ne_touche_rien_hors_de_programme() {
        let temp = tempfile::tempdir().unwrap();
        let programme = temp.path().join("programme");
        installer_dans(&programme);
        // Une liste qui nommerait un fichier voisin est refusée : seuls l'exe de l'appli et la liste partent.
        fs::write(temp.path().join("voisin.txt"), "voisin").unwrap();
        let liste = format!("{}\n..\\voisin.txt\n", nom_exe());
        fs::write(programme.join(LISTE), liste).unwrap();
        let bilan = vider_programme(&programme);
        assert!(temp.path().join("voisin.txt").is_file());
        assert_eq!(bilan.gardes, [INFOS_BUILD]);
        assert!(!programme.join(nom_exe()).exists());
    }

    #[test]
    fn vider_un_dossier_absent_ne_fait_rien() {
        let temp = tempfile::tempdir().unwrap();
        let absent = temp.path().join("absent");
        let bilan = vider_programme(&absent);
        assert_eq!(bilan, Bilan::default());
        assert_eq!(bilan.resume(&absent).unwrap(), "rien à supprimer");
        let bilan = vider_programme_en_attendant(&absent, Duration::from_secs(5));
        assert_eq!(bilan, Bilan::default());
    }

    #[test]
    fn un_fichier_restant_est_une_erreur_qui_le_nomme() {
        let bilan = Bilan {
            restants: vec![nom_exe()],
            ..Bilan::default()
        };
        let erreur = bilan.resume(Path::new("programme")).unwrap_err();
        assert!(erreur.contains(&nom_exe()), "{erreur}");
    }

    #[test]
    fn ce_que_devient_programme() {
        let temp = tempfile::tempdir().unwrap();
        let programme = temp.path().join("programme");
        let ailleurs = temp.path().join("zip").join(nom_exe());
        let sort = |exe: &Path| sort_de_programme(exe, &programme);
        assert_eq!(sort(&ailleurs), Programme::Absent);
        installer_dans(&programme);
        assert_eq!(sort(&ailleurs), Programme::Maintenant);
        assert_eq!(sort(&programme.join(nom_exe())), Programme::ALaFermeture);
    }

    #[test]
    fn seul_un_raccourci_vers_l_appli_est_a_nous() {
        let temp = tempfile::tempdir().unwrap();
        let programme = temp.path().join("programme");
        installer_dans(&programme);
        let installe = programme.join(nom_exe());
        let en_cours = temp.path().join("zip").join(nom_exe());
        let exes = [installe.as_path(), en_cours.as_path()];
        assert!(raccourci_a_nous(&installe, &exes));
        let autre_casse = PathBuf::from(installe.to_string_lossy().to_uppercase());
        assert!(raccourci_a_nous(&autre_casse, &exes));
        assert!(raccourci_a_nous(&en_cours, &exes));
        assert!(!raccourci_a_nous(&temp.path().join("autre.exe"), &exes));
        assert!(!raccourci_a_nous(Path::new(""), &exes));
    }

    #[test]
    fn le_processus_a_attendre_se_lit_dans_les_arguments() {
        let lire = |arguments: &[&str]| processus_a_attendre(arguments);
        assert_eq!(lire(&["--finir-desinstallation", "4242"]), Some(4242));
        assert_eq!(lire(&["x", "--FINIR-DESINSTALLATION", "7"]), Some(7));
        assert_eq!(lire(&["--finir-desinstallation"]), None);
        assert_eq!(lire(&["--finir-desinstallation", "abc"]), None);
        assert_eq!(lire(&["--silencieux"]), None);
    }

    #[test]
    fn l_aide_est_une_copie_dans_le_cache_retiree_ensuite() {
        let temp = tempfile::tempdir().unwrap();
        let cache = temp.path().join("cache");
        let programme = temp.path().join("programme");
        fs::create_dir_all(&programme).unwrap();
        let exe = programme.join(nom_exe());
        fs::write(&exe, "exe").unwrap();
        assert!(!est_l_aide(&exe, &cache));
        let copie = preparer_aide(&exe, &cache).unwrap();
        assert_eq!(fs::read_to_string(&copie).unwrap(), "exe");
        assert!(est_l_aide(&copie, &cache));
        nettoyer_aide(&cache);
        assert!(!dossier_aide(&cache).exists());
        assert!(exe.is_file());
    }
}
