//! Module « Fichiers et dossiers » (docs/CONTRAT.md, « Modules en option ») : ce qui ne demande pas la fenêtre.
//!
//! - [`reecrire`] : un fichier de l'utilisateur réécrit est d'abord copié dans `sauvegardes\`, d'où il se restaure
//!   depuis le Journal ;
//! - [`associations`] : les types de fichiers que l'appli déclare, associés seulement avec l'accord de l'utilisateur,
//!   dans la partie du registre qui lui est propre (`HKCU\Software\Classes`), et retirés à la désinstallation ;
//! - [`chemins_ouverts`] : les fichiers passés à l'exe par Windows (« Ouvrir avec », double-clic).
//!
//! Les fenêtres « Ouvrir » et « Enregistrer » et le glisser-déposer sont dans `fenetre::fichiers`.

use std::path::{Path, PathBuf};

use serde::Serialize;

use crate::{Dossier, Erreur, donnees, journal, sauvegardes};

/// Un type de fichier que l'appli sait ouvrir, déclaré par `Appli::types_de_fichiers`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TypeFichier {
    /// L'extension, sans point, en minuscules : 1 à 16 lettres ou chiffres (`md`, `jdr`).
    pub extension: &'static str,
    /// Ce que Windows affiche pour ce type (« Document Markdown »).
    pub description: &'static str,
}

/// Le genre des lignes de journal de ce module.
const GENRE: &str = "fichiers";

/// Réécrit un fichier de l'utilisateur : s'il existe, il est d'abord copié dans `sauvegardes\` (et se restaure
/// depuis le Journal), puis le nouveau contenu remplace l'ancien d'un coup, sans jamais le laisser à moitié écrit.
/// Renvoie le chemin de la copie, `None` pour un fichier nouveau.
///
/// # Errors
/// La copie échoue (rien n'est alors écrit), ou l'écriture échoue (le fichier d'origine reste intact).
pub fn reecrire(dossier: &Dossier, chemin: &Path, contenu: &[u8]) -> Result<Option<PathBuf>, Erreur> {
    let copie = if chemin.is_file() {
        Some(sauvegardes::sauvegarder(dossier, chemin)?)
    } else {
        None
    };
    donnees::ecrire_entier(chemin, contenu)?;
    let bilan = match &copie {
        Some(c) => format!(
            "{} réécrit, copie d'avant : {}",
            chemin.display(),
            c.file_name()
                .map_or_else(String::new, |n| n.to_string_lossy().into_owned())
        ),
        None => format!("{} créé", chemin.display()),
    };
    journal::consigner(dossier, GENRE, &bilan);
    Ok(copie)
}

/// Une extension acceptable : 1 à 16 lettres minuscules ou chiffres, sans point. Tout le reste est refusé, pour
/// qu'une erreur de déclaration ne touche jamais une autre clé du registre.
pub fn extension_valide(extension: &str) -> bool {
    (1..=16).contains(&extension.len())
        && extension
            .bytes()
            .all(|c| c.is_ascii_lowercase() || c.is_ascii_digit())
}

/// Le nom de la classe de fichiers de l'appli pour une extension : `Aria.<Nom>.<extension>`.
pub fn classe(nom_appli: &str, extension: &str) -> String {
    format!("{}{extension}", prefixe_classe(nom_appli))
}

/// Le début des classes de l'appli : `Aria.<Nom>.`. La désinstallation retire toutes celles qui le portent.
pub fn prefixe_classe(nom_appli: &str) -> String {
    format!("Aria.{nom_appli}.")
}

/// La commande que Windows lance pour ouvrir un fichier : l'exe installé, et le fichier entre guillemets.
pub fn commande_ouvrir(exe: &Path) -> String {
    format!("\"{}\" \"%1\"", exe.display())
}

/// Les fichiers passés à l'exe en arguments : les chemins complets de fichiers qui existent. Les options (`--…`) et
/// le reste sont ignorés.
pub fn chemins_ouverts<I, S>(arguments: I) -> Vec<PathBuf>
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
{
    arguments
        .into_iter()
        .filter(|a| !a.as_ref().starts_with('-'))
        .map(|a| PathBuf::from(a.as_ref()))
        .filter(|p| p.is_absolute() && p.is_file())
        .collect()
}

/// L'état d'un type de fichier, tel que les Paramètres le montrent.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct EtatAssociation {
    pub extension: String,
    pub description: String,
    /// L'utilisateur a accepté l'association : l'appli figure dans « Ouvrir avec » pour ce type.
    pub associee: bool,
    /// L'appli est aussi celle qui ouvre ce type par un double-clic.
    pub par_defaut: bool,
}

/// Les associations de types de fichiers, dans `HKCU\Software\Classes` : jamais dans la partie de la machine, jamais
/// à la place d'une autre appli. L'appli s'ajoute à « Ouvrir avec » ; elle n'ouvre le type par un double-clic que si
/// aucune autre appli ne le fait déjà, et Windows laisse l'utilisateur changer ce choix.
pub mod associations {
    use std::io;
    use std::path::Path;

    use super::{EtatAssociation, TypeFichier, classe, commande_ouvrir, extension_valide, prefixe_classe};

    /// Où écrire : `Software\Classes` pour de vrai, une clé à part dans les tests.
    #[derive(Debug, Clone)]
    pub struct Cles {
        pub classes: String,
        /// Lire aussi la vue fusionnée (`HKEY_CLASSES_ROOT`) pour savoir si une autre appli ouvre déjà un type.
        pub vue_fusionnee: bool,
    }

    impl Cles {
        /// Les clés de l'utilisateur.
        pub fn utilisateur() -> Self {
            Self {
                classes: r"Software\Classes".to_owned(),
                vue_fusionnee: true,
            }
        }
    }

    fn refuser(extension: &str) -> io::Error {
        io::Error::new(
            io::ErrorKind::InvalidInput,
            format!("extension refusée : « {extension} » (1 à 16 lettres minuscules ou chiffres)"),
        )
    }

    /// L'état de chaque type déclaré.
    pub fn etat(cles: &Cles, nom_appli: &str, types: &[TypeFichier]) -> Vec<EtatAssociation> {
        types
            .iter()
            .filter(|t| extension_valide(t.extension))
            .map(|t| {
                let classe = classe(nom_appli, t.extension);
                EtatAssociation {
                    extension: t.extension.to_owned(),
                    description: t.description.to_owned(),
                    associee: systeme::classe_existe(cles, &classe),
                    par_defaut: systeme::proprietaire(cles, t.extension).as_deref() == Some(classe.as_str()),
                }
            })
            .collect()
    }

    /// Associe un type à l'exe installé : la classe de l'appli, sa place dans « Ouvrir avec », et l'ouverture par
    /// double-clic seulement si aucune autre appli ne l'a.
    ///
    /// # Errors
    /// Une extension refusée, ou le registre qui ne s'écrit pas.
    pub fn associer(cles: &Cles, nom_appli: &str, type_fichier: &TypeFichier, exe: &Path) -> io::Result<()> {
        if !extension_valide(type_fichier.extension) {
            return Err(refuser(type_fichier.extension));
        }
        let classe = classe(nom_appli, type_fichier.extension);
        systeme::ecrire_classe(
            cles,
            &classe,
            type_fichier.description,
            exe,
            &commande_ouvrir(exe),
        )?;
        systeme::ajouter_ouvrir_avec(cles, type_fichier.extension, &classe)?;
        if systeme::proprietaire(cles, type_fichier.extension).is_none() {
            systeme::devenir_proprietaire(cles, type_fichier.extension, &classe)?;
        }
        systeme::prevenir_explorateur();
        Ok(())
    }

    /// Retire l'association d'un type : la classe de l'appli, sa place dans « Ouvrir avec », et l'ouverture par
    /// double-clic si c'était elle. Ce qui appartient à d'autres applis reste.
    ///
    /// # Errors
    /// Une extension refusée, ou le registre qui ne s'écrit pas.
    pub fn dissocier(cles: &Cles, nom_appli: &str, extension: &str) -> io::Result<()> {
        if !extension_valide(extension) {
            return Err(refuser(extension));
        }
        let classe = classe(nom_appli, extension);
        systeme::retirer(cles, extension, &classe)?;
        systeme::prevenir_explorateur();
        Ok(())
    }

    /// Retire toutes les associations de l'appli, même celles d'un type qu'elle ne déclare plus (désinstallation).
    /// Renvoie les extensions retirées.
    ///
    /// # Errors
    /// Le registre qui ne se lit pas ou ne s'écrit pas.
    pub fn dissocier_tout(cles: &Cles, nom_appli: &str) -> io::Result<Vec<String>> {
        let prefixe = prefixe_classe(nom_appli);
        let mut retirees = Vec::new();
        for classe in systeme::classes_commencant_par(cles, &prefixe)? {
            let extension = &classe[prefixe.len()..];
            if extension_valide(extension) {
                systeme::retirer(cles, extension, &classe)?;
                retirees.push(extension.to_owned());
            }
        }
        if !retirees.is_empty() {
            systeme::prevenir_explorateur();
        }
        retirees.sort();
        Ok(retirees)
    }

    #[cfg(windows)]
    mod systeme {
        use std::io;
        use std::path::Path;

        use winreg::enums::{HKEY_CLASSES_ROOT, HKEY_CURRENT_USER, RegType};
        use winreg::{RegKey, RegValue};

        use super::Cles;

        fn utilisateur() -> RegKey {
            RegKey::predef(HKEY_CURRENT_USER)
        }

        fn chemin(cles: &Cles, sous: &str) -> String {
            format!(r"{}\{sous}", cles.classes)
        }

        pub fn classe_existe(cles: &Cles, classe: &str) -> bool {
            utilisateur().open_subkey(chemin(cles, classe)).is_ok()
        }

        /// La classe qui ouvre le type par un double-clic, s'il y en a une.
        pub fn proprietaire(cles: &Cles, extension: &str) -> Option<String> {
            let lire = |racine: &RegKey, cle: &str| {
                racine
                    .open_subkey(cle)
                    .and_then(|c| c.get_value::<String, _>(""))
                    .ok()
                    .filter(|v| !v.trim().is_empty())
            };
            lire(&utilisateur(), &chemin(cles, &format!(".{extension}"))).or_else(|| {
                cles.vue_fusionnee
                    .then(|| lire(&RegKey::predef(HKEY_CLASSES_ROOT), &format!(".{extension}")))
                    .flatten()
            })
        }

        pub fn ecrire_classe(
            cles: &Cles,
            classe: &str,
            description: &str,
            exe: &Path,
            commande: &str,
        ) -> io::Result<()> {
            let (cle, _) = utilisateur().create_subkey(chemin(cles, classe))?;
            cle.set_value("", &description)?;
            let (icone, _) = cle.create_subkey("DefaultIcon")?;
            icone.set_value("", &format!("\"{}\",0", exe.display()))?;
            let (ouvrir, _) = cle.create_subkey(r"shell\open\command")?;
            ouvrir.set_value("", &commande)
        }

        pub fn ajouter_ouvrir_avec(cles: &Cles, extension: &str, classe: &str) -> io::Result<()> {
            let (liste, _) =
                utilisateur().create_subkey(chemin(cles, &format!(r".{extension}\OpenWithProgids")))?;
            let vide = RegValue {
                vtype: RegType::REG_NONE,
                bytes: Vec::new().into(),
            };
            liste.set_raw_value(classe, &vide)
        }

        pub fn devenir_proprietaire(cles: &Cles, extension: &str, classe: &str) -> io::Result<()> {
            let (cle, _) = utilisateur().create_subkey(chemin(cles, &format!(".{extension}")))?;
            cle.set_value("", &classe)
        }

        /// Retire la classe, la place dans « Ouvrir avec » et la propriété du type si c'était la nôtre ; puis les clés
        /// de l'extension si elles sont vides (créées par l'appli, sans rien d'autre dedans).
        pub fn retirer(cles: &Cles, extension: &str, classe: &str) -> io::Result<()> {
            let racine = utilisateur();
            let cle_extension = chemin(cles, &format!(".{extension}"));
            let cle_liste = format!(r"{cle_extension}\OpenWithProgids");
            if let Ok(liste) = racine.open_subkey_with_flags(&cle_liste, winreg::enums::KEY_ALL_ACCESS) {
                ignorer_absent(liste.delete_value(classe))?;
            }
            if let Ok(cle) = racine.open_subkey_with_flags(&cle_extension, winreg::enums::KEY_ALL_ACCESS)
                && cle.get_value::<String, _>("").is_ok_and(|v| v == classe)
            {
                ignorer_absent(cle.delete_value(""))?;
            }
            ignorer_absent(racine.delete_subkey_all(chemin(cles, classe)))?;
            for cle in [&cle_liste, &cle_extension] {
                let vide = racine
                    .open_subkey(cle)
                    .and_then(|c| c.query_info())
                    .is_ok_and(|i| i.sub_keys == 0 && i.values == 0);
                if vide {
                    ignorer_absent(racine.delete_subkey(cle))?;
                }
            }
            Ok(())
        }

        pub fn classes_commencant_par(cles: &Cles, prefixe: &str) -> io::Result<Vec<String>> {
            let Ok(classes) = utilisateur().open_subkey(&cles.classes) else {
                return Ok(Vec::new());
            };
            Ok(classes
                .enum_keys()
                .filter_map(Result::ok)
                .filter(|n| n.starts_with(prefixe))
                .collect())
        }

        fn ignorer_absent(resultat: io::Result<()>) -> io::Result<()> {
            match resultat {
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
                autre => autre,
            }
        }

        /// Dit à l'Explorateur que les associations ont changé, pour que les icônes et « Ouvrir avec » suivent.
        #[allow(unsafe_code)]
        pub fn prevenir_explorateur() {
            use windows::Win32::UI::Shell::{SHCNE_ASSOCCHANGED, SHCNF_IDLIST, SHChangeNotify};
            // SAFETY : SHCNE_ASSOCCHANGED ne prend aucun élément ; les deux pointeurs sont nuls, comme le demande
            // la documentation de SHChangeNotify.
            unsafe { SHChangeNotify(SHCNE_ASSOCCHANGED, SHCNF_IDLIST, None, None) };
        }
    }

    #[cfg(not(windows))]
    mod systeme {
        use std::io;
        use std::path::Path;

        use super::Cles;

        fn absent() -> io::Error {
            io::Error::new(io::ErrorKind::Unsupported, "Windows seulement")
        }

        pub fn classe_existe(_cles: &Cles, _classe: &str) -> bool {
            false
        }

        pub fn proprietaire(_cles: &Cles, _extension: &str) -> Option<String> {
            None
        }

        pub fn ecrire_classe(_: &Cles, _: &str, _: &str, _: &Path, _: &str) -> io::Result<()> {
            Err(absent())
        }

        pub fn ajouter_ouvrir_avec(_: &Cles, _: &str, _: &str) -> io::Result<()> {
            Err(absent())
        }

        pub fn devenir_proprietaire(_: &Cles, _: &str, _: &str) -> io::Result<()> {
            Err(absent())
        }

        pub fn retirer(_: &Cles, _: &str, _: &str) -> io::Result<()> {
            Ok(())
        }

        pub fn classes_commencant_par(_: &Cles, _: &str) -> io::Result<Vec<String>> {
            Ok(Vec::new())
        }

        pub fn prevenir_explorateur() {}
    }

    #[cfg(all(test, windows))]
    mod tests {
        use super::*;
        use std::path::PathBuf;

        /// Une clé à part, propre à chaque test, retirée à la fin.
        struct CleEssai(Cles);

        impl CleEssai {
            fn nouvelle(nom: &str) -> Self {
                Self(Cles {
                    classes: format!(r"Software\Aria-Tests\associations-{nom}-{}", std::process::id()),
                    vue_fusionnee: false,
                })
            }
        }

        impl Drop for CleEssai {
            fn drop(&mut self) {
                let _ = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER)
                    .delete_subkey_all(&self.0.classes);
            }
        }

        const MD: TypeFichier = TypeFichier {
            extension: "md",
            description: "Document Markdown",
        };

        #[test]
        fn associer_puis_tout_retirer() {
            let cle = CleEssai::nouvelle("aller-retour");
            let exe = PathBuf::from(r"C:\Users\x\.Aria-Atlas\programme\Aria-Atlas.exe");
            assert!(!etat(&cle.0, "Atlas", &[MD])[0].associee);
            associer(&cle.0, "Atlas", &MD, &exe).unwrap();
            let vu = &etat(&cle.0, "Atlas", &[MD])[0];
            assert!(vu.associee && vu.par_defaut);
            assert_eq!(dissocier_tout(&cle.0, "Atlas").unwrap(), ["md"]);
            let vu = &etat(&cle.0, "Atlas", &[MD])[0];
            assert!(!vu.associee && !vu.par_defaut);
        }

        #[test]
        fn le_type_d_une_autre_appli_reste_a_elle() {
            let cle = CleEssai::nouvelle("autre");
            let exe = PathBuf::from(r"C:\x\Aria-Atlas.exe");
            systeme::devenir_proprietaire(&cle.0, "md", "Autre.Editeur").unwrap();
            associer(&cle.0, "Atlas", &MD, &exe).unwrap();
            let vu = &etat(&cle.0, "Atlas", &[MD])[0];
            assert!(vu.associee && !vu.par_defaut);
            dissocier(&cle.0, "Atlas", "md").unwrap();
            assert_eq!(
                systeme::proprietaire(&cle.0, "md").as_deref(),
                Some("Autre.Editeur")
            );
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reecrire_copie_d_abord() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path().join(".Aria-Essai"));
        dossier.preparer().unwrap();
        let fichier = temp.path().join("notes.md");
        assert_eq!(reecrire(&dossier, &fichier, b"un").unwrap(), None);
        let copie = reecrire(&dossier, &fichier, b"deux").unwrap().unwrap();
        assert_eq!(std::fs::read(&fichier).unwrap(), b"deux");
        assert_eq!(std::fs::read(&copie).unwrap(), b"un");
        let liste = sauvegardes::lister(&dossier);
        assert_eq!(liste.len(), 1);
        assert!(
            liste[0]
                .origine
                .as_deref()
                .is_some_and(|o| o.ends_with("notes.md"))
        );
    }

    #[test]
    fn les_extensions_sont_bornees() {
        for bonne in ["md", "jdr", "mp3", "a"] {
            assert!(extension_valide(bonne), "{bonne}");
        }
        for mauvaise in ["", ".md", "MD", "m d", r"md\..\x", "abcdefghijklmnopq", "é"] {
            assert!(!extension_valide(mauvaise), "{mauvaise}");
        }
    }

    #[test]
    fn noms_et_commande() {
        assert_eq!(classe("Atlas", "md"), "Aria.Atlas.md");
        assert!(classe("Atlas", "md").starts_with(&prefixe_classe("Atlas")));
        assert_eq!(
            commande_ouvrir(Path::new(r"C:\Users\Jean Dupont\Aria-Atlas.exe")),
            r#""C:\Users\Jean Dupont\Aria-Atlas.exe" "%1""#
        );
    }

    #[test]
    fn seuls_les_fichiers_existants_sont_ouverts() {
        let temp = tempfile::tempdir().unwrap();
        let fichier = temp.path().join("a.md");
        std::fs::write(&fichier, "x").unwrap();
        let texte = fichier.display().to_string();
        let dossier = temp.path().display().to_string();
        let ouverts = chemins_ouverts(["--silencieux", "relatif.md", &dossier, &texte, "/absent/b.md"]);
        assert_eq!(ouverts, [fichier]);
    }
}
