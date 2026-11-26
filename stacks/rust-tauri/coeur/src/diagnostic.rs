//! Le diagnostic à coller dans un signalement de bug : version, build, Windows, WebView2, réglages et dernières lignes
//! du journal. Rien n'est envoyé : le texte part dans le presse-papiers, l'utilisateur le relit et le colle lui-même.
//!
//! Aucun secret : `secrets\` n'est jamais lu, et les réglages n'en contiennent pas. Le chemin du profil de
//! l'utilisateur devient `%USERPROFILE%` : le nom du compte Windows n'apparaît pas dans un signalement public.

use crate::{Dossier, Reglages, identite, journal};

/// Les lignes du journal reprises, les plus récentes.
pub const LIGNES_JOURNAL: usize = 50;

/// Ce que l'exe sait de lui-même et de Windows, lu hors du cœur.
#[derive(Debug, Clone, Default)]
pub struct Contexte {
    /// Le contenu de `BUILDINFO.txt`, à côté de l'exe ; rien pour un exe compilé à la main.
    pub build: Option<String>,
    /// La version de Windows, lisible (voir [`windows_lisible`]).
    pub windows: Option<String>,
    /// La version de WebView2.
    pub webview2: Option<String>,
    /// Le dossier du profil de l'utilisateur, remplacé par `%USERPROFILE%` dans le texte.
    pub profil: Option<String>,
}

/// Le texte du diagnostic.
pub fn texte(dossier: &Dossier, contexte: &Contexte) -> String {
    let valeur = |v: &Option<String>| {
        v.as_deref()
            .map(str::trim)
            .filter(|s| !s.is_empty())
            .unwrap_or("inconnue")
            .to_owned()
    };
    let mut lignes = vec![
        format!("Diagnostic de {}", identite::NOM),
        format!("Version : {}", identite::VERSION),
        format!("Windows : {}", valeur(&contexte.windows)),
        format!("WebView2 : {}", valeur(&contexte.webview2)),
        format!("Dossier de données : {}", dossier.racine().display()),
        String::new(),
        "Build :".to_owned(),
    ];
    match contexte.build.as_deref().map(str::trim).filter(|b| !b.is_empty()) {
        Some(build) => lignes.extend(build.lines().map(|l| format!("  {l}"))),
        None => {
            lignes.push("  inconnu : pas de BUILDINFO.txt à côté de l'exe (exe compilé à la main)".to_owned())
        }
    }

    lignes.push(String::new());
    lignes.push("Réglages :".to_owned());
    let reglages = serde_json::to_string_pretty(&Reglages::charger(dossier))
        .unwrap_or_else(|e| format!("illisibles : {e}"));
    lignes.extend(reglages.lines().map(|l| format!("  {l}")));

    lignes.push(String::new());
    lignes.push(format!(
        "Journal ({LIGNES_JOURNAL} dernières opérations au plus, la plus récente en bas) :"
    ));
    let operations = journal::lire(dossier);
    let debut = operations.len().saturating_sub(LIGNES_JOURNAL);
    for operation in operations.iter().skip(debut) {
        lignes.push(format!(
            "  {}  {}  {}",
            operation.date, operation.genre, operation.bilan
        ));
    }

    // Le profil d'après le dossier de données (son parent), qui est la source de tous les chemins du texte, puis
    // celui que donne Windows s'il diffère.
    let profil_du_dossier = dossier.racine().parent().map(|p| p.display().to_string());
    let texte = masquer_profil(&lignes.join("\n"), profil_du_dossier.as_deref());
    masquer_profil(&texte, contexte.profil.as_deref())
}

/// Remplace le chemin du profil de l'utilisateur par `%USERPROFILE%`, sans tenir compte de la casse (comme Windows).
pub fn masquer_profil(texte: &str, profil: Option<&str>) -> String {
    let Some(profil) = profil
        .map(|p| p.trim_end_matches(['\\', '/']))
        .filter(|p| p.len() > 3)
    else {
        return texte.to_owned();
    };
    let mut resultat = String::with_capacity(texte.len());
    let mut reste = texte;
    while let Some(debut) = trouver(reste, profil) {
        let (Some(avant), Some(apres)) = (reste.get(..debut), reste.get(debut + profil.len()..)) else {
            break;
        };
        resultat.push_str(avant);
        resultat.push_str("%USERPROFILE%");
        reste = apres;
    }
    resultat.push_str(reste);
    resultat
}

/// La première position de `motif` dans `texte`, sans tenir compte de la casse des lettres ASCII.
fn trouver(texte: &str, motif: &str) -> Option<usize> {
    texte.char_indices().map(|(i, _)| i).find(|&i| {
        texte
            .get(i..i + motif.len())
            .is_some_and(|s| s.eq_ignore_ascii_case(motif))
    })
}

/// La version de Windows, lisible : « Windows 11 Pro 24H2 (build 26100.2894) ». Windows 11 se déclare encore
/// « Windows 10 » dans le registre (`ProductName`) : le numéro de build, 22000 et plus, tranche.
pub fn windows_lisible(produit: &str, version: Option<&str>, build: u32, revision: Option<u32>) -> String {
    let mut texte = if build >= 22_000 {
        produit.replacen("Windows 10", "Windows 11", 1)
    } else {
        produit.to_owned()
    };
    if let Some(version) = version.map(str::trim).filter(|v| !v.is_empty()) {
        texte.push(' ');
        texte.push_str(version);
    }
    match revision {
        Some(revision) => format!("{texte} (build {build}.{revision})"),
        None => format!("{texte} (build {build})"),
    }
}

/// La version de Windows, lue dans le registre (`HKLM\SOFTWARE\Microsoft\Windows NT\CurrentVersion`, sans droits
/// administrateur) ; rien si elle ne se lit pas.
#[cfg(windows)]
pub fn version_windows() -> Option<String> {
    use winreg::RegKey;
    use winreg::enums::HKEY_LOCAL_MACHINE;

    let cle = RegKey::predef(HKEY_LOCAL_MACHINE)
        .open_subkey(r"SOFTWARE\Microsoft\Windows NT\CurrentVersion")
        .ok()?;
    let produit: String = cle.get_value("ProductName").ok()?;
    let version: Option<String> = cle.get_value("DisplayVersion").ok();
    let build: String = cle.get_value("CurrentBuild").ok()?;
    let revision: Option<u32> = cle.get_value("UBR").ok();
    Some(windows_lisible(
        &produit,
        version.as_deref(),
        build.trim().parse().ok()?,
        revision,
    ))
}

/// Hors de Windows : pas de version de Windows.
#[cfg(not(windows))]
pub fn version_windows() -> Option<String> {
    None
}

#[cfg(test)]
mod tests {
    use std::io;

    use super::*;
    use crate::secrets::{Coffre, Protection};

    /// Une « protection » qui ne chiffre rien : la valeur du secret est en clair dans `secrets\`, et le test n'en est
    /// que plus sévère.
    struct EnClair;

    impl Protection for EnClair {
        fn proteger(&self, clair: &[u8]) -> io::Result<Vec<u8>> {
            Ok(clair.to_vec())
        }
        fn deproteger(&self, chiffre: &[u8]) -> io::Result<Vec<u8>> {
            Ok(chiffre.to_vec())
        }
    }

    #[test]
    fn aucun_secret_dans_le_diagnostic() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path());
        dossier.preparer().unwrap();
        let coffre = Coffre::du_dossier(&dossier, &EnClair);
        coffre.enregistrer("jeton-api", "VALEUR-SECRETE-7f3a").unwrap();
        journal::consigner(&dossier, "secret", "jeton-api enregistré");
        let texte = texte(&dossier, &Contexte::default());
        assert!(!texte.contains("VALEUR-SECRETE-7f3a"), "{texte}");
        assert!(texte.contains("jeton-api enregistré"), "{texte}");
    }

    #[test]
    fn le_diagnostic_reprend_les_50_dernieres_lignes_du_journal() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path());
        for n in 0..LIGNES_JOURNAL + 5 {
            journal::consigner(&dossier, "essai", &format!("ligne-{n:03}"));
        }
        let texte = texte(&dossier, &Contexte::default());
        assert!(!texte.contains("ligne-004"), "{texte}");
        assert!(texte.contains("ligne-005"), "{texte}");
        assert!(texte.contains("ligne-054"), "{texte}");
    }

    #[test]
    fn le_profil_est_masque() {
        let profil = r"C:\Users\jdupont";
        let contexte = Contexte {
            build: Some("Aria Essai 1.2.3\nCommit : abc".to_owned()),
            windows: Some("Windows 11 Pro 24H2 (build 26100.1)".to_owned()),
            webview2: Some("131.0.2903.70".to_owned()),
            profil: Some(format!("{profil}\\")),
        };
        // Un dossier qui n'existe pas : réglages par défaut, journal vide ; seul son chemin compte ici.
        let dossier = Dossier::a(format!(r"{profil}\.Aria-Essai"));
        let texte = texte(&dossier, &contexte);
        assert!(!texte.contains("jdupont"), "{texte}");
        assert!(texte.contains(r"%USERPROFILE%\.Aria-Essai"), "{texte}");
        assert!(texte.contains("Commit : abc"), "{texte}");
        assert!(texte.contains("WebView2 : 131.0.2903.70"), "{texte}");
    }

    #[test]
    fn le_profil_est_masque_quelle_que_soit_la_casse() {
        let texte = "installé dans c:\\USERS\\JDupont\\.Aria-Essai\\programme ; C:\\Users\\jdupont";
        let masque = masquer_profil(texte, Some(r"C:\Users\jdupont\"));
        assert_eq!(
            masque,
            "installé dans %USERPROFILE%\\.Aria-Essai\\programme ; %USERPROFILE%"
        );
        assert_eq!(masquer_profil("rien", Some("C:")), "rien");
    }

    #[test]
    fn aucun_reglage_ne_porte_un_nom_de_secret() {
        // Les secrets vivent dans secrets\, jamais dans reglages.json : le diagnostic recopie les réglages tels quels.
        let json = serde_json::to_string(&Reglages::default()).unwrap();
        let bas = json.to_lowercase();
        for mot in [
            "secret",
            "jeton",
            "token",
            "password",
            "motdepasse",
            "mot_de_passe",
            "cle",
            "key",
        ] {
            assert!(!bas.contains(&format!("{mot}\"")), "{mot} : {json}");
        }
    }

    #[test]
    fn sans_build_ni_windows_le_diagnostic_le_dit() {
        let temp = tempfile::tempdir().unwrap();
        let texte = texte(&Dossier::a(temp.path()), &Contexte::default());
        assert!(texte.contains("pas de BUILDINFO.txt"), "{texte}");
        assert!(texte.contains("Windows : inconnue"), "{texte}");
    }

    #[test]
    fn windows_11_se_reconnait_a_son_build() {
        let onze = windows_lisible("Windows 10 Pro", Some("24H2"), 26_100, Some(2894));
        assert_eq!(onze, "Windows 11 Pro 24H2 (build 26100.2894)");
        let dix = windows_lisible("Windows 10 Pro", Some("22H2"), 19_045, None);
        assert_eq!(dix, "Windows 10 Pro 22H2 (build 19045)");
    }
}
