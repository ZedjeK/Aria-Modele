//! Le dossier commun à la famille, `%USERPROFILE%\.Aria` (décision 0009) : `modeles\` pour les modèles,
//! `moteurs\<moteur>\<version>\` pour les moteurs locaux, et `usage.json`, le registre de qui utilise quoi. Ni
//! réglage, ni secret, ni donnée personnelle.
//!
//! L'utilisateur peut le placer sur un autre disque, une fois, pour toute la famille : `%USERPROFILE%\.Aria\ailleurs.txt`
//! donne alors son nouvel emplacement. Une seule appli écrit à la fois, sous un verrou ; la lecture reste libre.

use std::fs::{self, OpenOptions};
use std::io::{self, ErrorKind};
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Le nom du dossier commun, dans le profil de l'utilisateur.
pub const NOM: &str = ".Aria";

/// Le fichier qui dit où le dossier commun a été déplacé.
pub const AILLEURS: &str = "ailleurs.txt";

/// Le verrou d'écriture.
const VERROU: &str = "verrou";

/// Un verrou plus vieux que ça est abandonné (appli arrêtée en pleine écriture) : il est repris.
const VERROU_ABANDONNE: Duration = Duration::from_secs(60 * 60);

/// Le dossier commun.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Commun {
    racine: PathBuf,
}

impl Commun {
    /// Le dossier commun de l'utilisateur, déplacé ou non.
    ///
    /// # Errors
    /// Ni `USERPROFILE` ni `HOME`.
    pub fn du_profil() -> Result<Self, String> {
        let profil = std::env::var_os("USERPROFILE")
            .or_else(|| std::env::var_os("HOME"))
            .ok_or_else(|| {
                "le dossier de l'utilisateur est introuvable (ni USERPROFILE ni HOME)".to_owned()
            })?;
        Ok(Self::depuis(&PathBuf::from(profil).join(NOM)))
    }

    /// Le dossier commun d'origine `origine`, ou celui vers lequel il a été déplacé.
    pub fn depuis(origine: &Path) -> Self {
        let deplace = fs::read_to_string(origine.join(AILLEURS))
            .ok()
            .map(|texte| PathBuf::from(texte.trim()))
            .filter(|chemin| chemin.is_absolute());
        Self {
            racine: deplace.unwrap_or_else(|| origine.to_path_buf()),
        }
    }

    /// Un dossier commun à cet endroit précis (tests).
    pub fn a(racine: impl Into<PathBuf>) -> Self {
        Self {
            racine: racine.into(),
        }
    }

    pub fn racine(&self) -> &Path {
        &self.racine
    }

    /// Les modèles, partagés par toutes les applis.
    pub fn modeles(&self) -> PathBuf {
        self.racine.join("modeles")
    }

    /// Les moteurs locaux, une version par dossier : `moteurs\<moteur>\<version>\`.
    pub fn moteurs(&self) -> PathBuf {
        self.racine.join("moteurs")
    }

    /// Le registre : quelle appli utilise quel moteur et quel modèle.
    pub fn registre(&self) -> PathBuf {
        self.racine.join("usage.json")
    }

    /// Les déclarations de mémoire vidéo (module « partage de la carte graphique »).
    pub fn carte_graphique(&self) -> PathBuf {
        self.racine.join("carte-graphique.json")
    }

    /// Prend le verrou d'écriture, ou dit quelle appli le tient. Il est rendu quand le [`Verrou`] est lâché.
    ///
    /// # Errors
    /// Une autre appli écrit en ce moment, ou le dossier ne s'écrit pas.
    pub fn verrouiller(&self, appli: &str) -> Result<Verrou, String> {
        fs::create_dir_all(&self.racine).map_err(|e| format!("{} : {e}", self.racine.display()))?;
        let chemin = self.racine.join(VERROU);
        for _ in 0..2 {
            match OpenOptions::new().write(true).create_new(true).open(&chemin) {
                Ok(_) => {
                    fs::write(&chemin, appli).map_err(|e| e.to_string())?;
                    return Ok(Verrou { chemin });
                }
                Err(e) if e.kind() == ErrorKind::AlreadyExists => {
                    let age = fs::metadata(&chemin)
                        .and_then(|m| m.modified())
                        .ok()
                        .and_then(|date| SystemTime::now().duration_since(date).ok());
                    if age.is_some_and(|age| age > VERROU_ABANDONNE) {
                        let _ = fs::remove_file(&chemin);
                        continue;
                    }
                    let tenu_par = fs::read_to_string(&chemin).unwrap_or_default();
                    return Err(format!(
                        "le dossier commun est en cours d'écriture par {} : réessaie dans un moment",
                        if tenu_par.trim().is_empty() {
                            "une autre appli"
                        } else {
                            tenu_par.trim()
                        }
                    ));
                }
                Err(e) => return Err(format!("{} : {e}", chemin.display())),
            }
        }
        Err("le verrou du dossier commun n'a pas pu être pris".to_owned())
    }

    /// Place le dossier commun sur un autre disque, une seule fois pour toute la famille : son contenu est copié vers
    /// `vers`, puis `ailleurs.txt` y renvoie. L'ancien contenu n'est pas supprimé : c'est un geste de l'utilisateur.
    ///
    /// # Errors
    /// Déjà déplacé, `vers` n'est pas un dossier vide et absolu, ou la copie échoue (rien ne renvoie alors ailleurs).
    pub fn deplacer(origine: &Path, vers: &Path) -> Result<Self, String> {
        if origine.join(AILLEURS).exists() {
            return Err("le dossier commun a déjà été déplacé : il ne se déplace qu'une fois".to_owned());
        }
        if !vers.is_absolute() {
            return Err("le nouvel emplacement doit être un chemin complet".to_owned());
        }
        if vers.exists() && fs::read_dir(vers).map_err(|e| e.to_string())?.next().is_some() {
            return Err(format!("{} n'est pas vide", vers.display()));
        }
        if origine.is_dir() {
            copier(origine, vers).map_err(|e| format!("copie vers {} : {e}", vers.display()))?;
        } else {
            fs::create_dir_all(vers).map_err(|e| e.to_string())?;
        }
        fs::create_dir_all(origine).map_err(|e| e.to_string())?;
        crate::donnees::ecrire_entier(&origine.join(AILLEURS), vers.display().to_string().as_bytes())
            .map_err(|e| e.to_string())?;
        Ok(Self::a(vers))
    }
}

/// Copie un dossier et tout ce qu'il contient, sauf le verrou.
fn copier(de: &Path, vers: &Path) -> io::Result<()> {
    fs::create_dir_all(vers)?;
    for entree in fs::read_dir(de)? {
        let entree = entree?;
        if entree.file_name() == VERROU {
            continue;
        }
        let cible = vers.join(entree.file_name());
        let genre = entree.file_type()?;
        if genre.is_dir() {
            copier(&entree.path(), &cible)?;
        } else if genre.is_file() {
            fs::copy(entree.path(), &cible)?;
        }
    }
    Ok(())
}

/// Le verrou d'écriture du dossier commun, rendu à la fin de sa portée.
#[derive(Debug)]
pub struct Verrou {
    chemin: PathBuf,
}

impl Drop for Verrou {
    fn drop(&mut self) {
        let _ = fs::remove_file(&self.chemin);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_seul_ecrivain_a_la_fois() {
        let temp = tempfile::tempdir().unwrap();
        let commun = Commun::a(temp.path());
        let verrou = commun.verrouiller("Aria-Studio").unwrap();
        let refus = commun.verrouiller("Aria-MD").unwrap_err();
        assert!(refus.contains("Aria-Studio"), "{refus}");
        drop(verrou);
        commun.verrouiller("Aria-MD").unwrap();
    }

    #[test]
    fn le_dossier_ne_se_deplace_qu_une_fois() {
        let temp = tempfile::tempdir().unwrap();
        let origine = temp.path().join(NOM);
        fs::create_dir_all(origine.join("modeles")).unwrap();
        fs::write(origine.join("modeles").join("m.gguf"), "poids").unwrap();
        let vers = temp.path().join("D").join("Aria");
        let deplace = Commun::deplacer(&origine, &vers).unwrap();
        assert_eq!(deplace.racine(), vers);
        assert_eq!(Commun::depuis(&origine), deplace);
        assert_eq!(
            fs::read_to_string(vers.join("modeles").join("m.gguf")).unwrap(),
            "poids"
        );
        // L'ancien contenu reste : le supprimer est un geste de l'utilisateur.
        assert!(origine.join("modeles").join("m.gguf").is_file());
        assert!(Commun::deplacer(&origine, &temp.path().join("E")).is_err());
    }

    #[test]
    fn un_emplacement_relatif_ou_plein_est_refuse() {
        let temp = tempfile::tempdir().unwrap();
        let origine = temp.path().join(NOM);
        assert!(Commun::deplacer(&origine, Path::new("relatif")).is_err());
        let plein = temp.path().join("plein");
        fs::create_dir_all(&plein).unwrap();
        fs::write(plein.join("x"), "x").unwrap();
        assert!(Commun::deplacer(&origine, &plein).is_err());
        assert!(!origine.join(AILLEURS).exists());
    }
}
