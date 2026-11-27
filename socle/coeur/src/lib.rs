//! Le socle des applis Aria : ce que toute appli doit tenir (docs/CONTRAT.md du dépôt modèle).
//!
//! Hors du module `fenetre`, tout se teste sur n'importe quel système, sauf les accès à Windows (registre, DPAPI),
//! marqués `cfg(windows)` et testés par la CI Windows. Le module `fenetre` (fonction `fenetre`, que toute appli
//! active) porte la fenêtre Tauri, ses commandes et le lancement de l'appli.

pub mod demarrage;
pub mod desinstallation;
pub mod diagnostic;
pub mod donnees;
#[cfg(feature = "fenetre")]
pub mod fenetre;
pub mod identite;
pub mod installation;
pub mod journal;
pub mod mises_a_jour;
pub mod presence;
pub mod reglages;
pub mod reprise;
pub mod sauvegardes;
pub mod secrets;

pub use donnees::Dossier;
pub use reglages::{Reglages, ReglagesFond, Vitesse};

use std::path::PathBuf;

use time::OffsetDateTime;

/// Maintenant, à l'heure locale ; à l'heure UTC si Windows ne donne pas le décalage.
pub(crate) fn maintenant() -> OffsetDateTime {
    OffsetDateTime::now_local().unwrap_or_else(|_| OffsetDateTime::now_utc())
}

/// Les erreurs du cœur, en français : elles peuvent être montrées telles quelles à l'utilisateur.
#[derive(Debug, thiserror::Error)]
pub enum Erreur {
    /// Ni `USERPROFILE` ni `HOME` : impossible de savoir où ranger le dossier de données.
    #[error("le dossier de l'utilisateur est introuvable (ni USERPROFILE ni HOME)")]
    ProfilIntrouvable,

    /// Une lecture ou une écriture a échoué ; le chemin dit où.
    #[error("{action} {chemin} : {source}")]
    Fichier {
        action: &'static str,
        chemin: PathBuf,
        #[source]
        source: std::io::Error,
    },

    /// Le fichier à sauvegarder n'existe pas.
    #[error("fichier à sauvegarder introuvable : {0}")]
    AbsentASauvegarder(PathBuf),

    /// Un nom de secret qui pourrait sortir du dossier des secrets.
    #[error(
        "nom de secret refusé : « {0} » (minuscules, chiffres et tirets, 64 au plus, sans nom réservé de Windows)"
    )]
    NomDeSecret(String),
}

impl Erreur {
    /// Raccourci pour une erreur de fichier.
    pub(crate) fn fichier(action: &'static str, chemin: impl Into<PathBuf>, source: std::io::Error) -> Self {
        Self::Fichier {
            action,
            chemin: chemin.into(),
            source,
        }
    }
}
