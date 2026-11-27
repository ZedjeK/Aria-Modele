//! Nom de l'appli, de son exe et de son dossier de données, sa promesse et sa version : un seul endroit, rempli par
//! l'appli au lancement (`definir`), avant tout autre appel au socle.

use std::sync::OnceLock;

/// Ce que l'appli dit d'elle-même. Les noms dérivés (exe, dossier de données) se calculent depuis `nom`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Identite {
    /// Le nom sans « Aria » : une majuscule, puis des lettres ou des chiffres (exemple : `Atlas`).
    pub nom: &'static str,
    /// La promesse de l'appli, en une phrase.
    pub promesse: &'static str,
    /// La version X.Y.Z de l'appli : `env!("CARGO_PKG_VERSION")` dans l'appli, jamais celle du socle.
    pub version: &'static str,
    /// L'appli sert-elle en arrière-plan (icône de notification, démarrage avec Windows) ?
    pub en_fond: bool,
}

/// L'identité retenue et ses noms dérivés.
struct Retenue {
    identite: Identite,
    nom: String,
    nom_exe: String,
    dossier: String,
}

impl Retenue {
    fn de(identite: Identite) -> Self {
        Self {
            identite,
            nom: format!("Aria {}", identite.nom),
            nom_exe: format!("Aria-{}", identite.nom),
            dossier: format!(".Aria-{}", identite.nom),
        }
    }
}

static IDENTITE: OnceLock<Retenue> = OnceLock::new();

/// L'identité des tests et des outils, tant que l'appli n'a rien défini.
const PAR_DEFAUT: Identite = Identite {
    nom: "Essai",
    promesse: "Une appli pour essayer le socle.",
    version: "0.0.0",
    en_fond: false,
};

/// Retient l'identité de l'appli. Seul le premier appel compte : un nom ne change pas en cours de route.
///
/// # Errors
/// Le nom ne suit pas la règle (une majuscule, puis 1 à 30 lettres ou chiffres), ou une identité est déjà retenue.
pub fn definir(identite: Identite) -> Result<(), String> {
    if !nom_valide(identite.nom) {
        return Err(format!("nom d'appli refusé : « {} »", identite.nom));
    }
    IDENTITE
        .set(Retenue::de(identite))
        .map_err(|_| "l'identité de l'appli est déjà définie".to_owned())
}

/// Le nom suit-il la règle d'init.ps1 ?
fn nom_valide(nom: &str) -> bool {
    let mut lettres = nom.chars();
    lettres.next().is_some_and(|c| c.is_ascii_uppercase())
        && (2..=31).contains(&nom.len())
        && lettres.all(|c| c.is_ascii_alphanumeric())
}

fn retenue() -> &'static Retenue {
    IDENTITE.get_or_init(|| Retenue::de(PAR_DEFAUT))
}

/// L'identité retenue.
pub fn identite() -> Identite {
    retenue().identite
}

/// Nom affiché : « Aria Nom ».
pub fn nom() -> &'static str {
    &retenue().nom
}

/// Nom de l'exe, sans extension : « Aria-Nom ».
pub fn nom_exe() -> &'static str {
    &retenue().nom_exe
}

/// Nom du dossier de données, dans le profil de l'utilisateur : « .Aria-Nom ».
pub fn dossier_donnees() -> &'static str {
    &retenue().dossier
}

/// La promesse de l'appli.
pub fn promesse() -> &'static str {
    retenue().identite.promesse
}

/// La version X.Y.Z de l'appli.
pub fn version() -> &'static str {
    retenue().identite.version
}

/// L'appli sert-elle en arrière-plan ?
pub fn en_fond() -> bool {
    retenue().identite.en_fond
}

/// La version du socle lui-même, celle de cette crate.
pub const VERSION_SOCLE: &str = env!("CARGO_PKG_VERSION");

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_version_est_x_y_z() {
        for version in [version(), VERSION_SOCLE] {
            let morceaux: Vec<&str> = version.split('.').collect();
            assert_eq!(morceaux.len(), 3, "{version}");
            assert!(morceaux.iter().all(|m| m.parse::<u32>().is_ok()), "{version}");
        }
    }

    #[test]
    fn les_noms_se_correspondent() {
        assert_eq!(nom().replace(' ', "-"), nom_exe());
        assert_eq!(dossier_donnees(), format!(".{}", nom_exe()));
    }

    #[test]
    fn un_nom_hors_regle_est_refuse() {
        for nom in [
            "",
            "a",
            "atlas",
            "Atlas Pro",
            "Atlàs",
            "A",
            "Abcdefghijklmnopqrstuvwxyz123456",
        ] {
            assert!(!nom_valide(nom), "{nom}");
        }
        for nom in ["Ab", "Atlas", "Studio2", "Abcdefghijklmnopqrstuvwxyz12345"] {
            assert!(nom_valide(nom), "{nom}");
        }
    }

    #[test]
    fn une_seconde_definition_est_refusee() {
        // L'identité par défaut a déjà pu être retenue par un autre test : dans les deux cas, un second appel échoue.
        let _ = definir(PAR_DEFAUT);
        assert!(definir(PAR_DEFAUT).is_err());
    }
}
