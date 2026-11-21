//! Nom de l'appli, de son exe et de son dossier de données : un seul endroit pour toute l'appli, rempli par
//! init.ps1.

/// Nom affiché.
pub const NOM: &str = "Aria {{NOM}}";

/// Nom de l'exe, sans extension.
pub const NOM_EXE: &str = "Aria-{{NOM}}";

/// Nom du dossier de données, dans le profil de l'utilisateur.
pub const DOSSIER_DONNEES: &str = ".Aria-{{NOM}}";

/// La promesse de l'appli, en une phrase.
pub const PROMESSE: &str = "{{PROMESSE}}";

/// Version X.Y.Z, celle de `[workspace.package]` dans Cargo.toml.
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// L'appli sert en arrière-plan : icône de notification, la croix range la fenêtre dans l'icône, démarrage
/// automatique proposé (désactivé par défaut). Faux : ni icône ni démarrage automatique, la croix quitte.
/// Réglé par init.ps1 d'après la fiche projet.
pub const EN_FOND: bool = {{EN_FOND}};

/// L'exe propose de se ranger dans le dossier `programme\` du dossier de données (appli de bureau). Faux : exe
/// portable seulement. Réglé par init.ps1 d'après la fiche projet.
pub const INSTALLABLE: bool = {{INSTALLABLE}};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_version_est_x_y_z() {
        let morceaux: Vec<&str> = VERSION.split('.').collect();
        assert_eq!(morceaux.len(), 3, "{VERSION}");
        assert!(morceaux.iter().all(|m| m.parse::<u32>().is_ok()), "{VERSION}");
    }

    #[test]
    fn les_noms_se_correspondent() {
        assert_eq!(NOM.replace(' ', "-"), NOM_EXE);
        assert_eq!(DOSSIER_DONNEES, format!(".{NOM_EXE}"));
    }
}
