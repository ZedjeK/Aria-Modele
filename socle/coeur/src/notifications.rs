//! Module « Notifications » (docs/CONTRAT.md, « Modules en option ») : ce qui ne demande pas la fenêtre.
//!
//! Une notification de Windows demande que l'appli soit inscrite sous un identifiant (`AppUserModelId`) : son nom et
//! son icône, dans `HKCU\Software\Classes\AppUserModelId\Aria.<Nom>`, retirés à la désinstallation. Le contenu est un
//! petit document XML (titre et texte), échappé ici.
//!
//! L'envoi, le réglage et la fenêtre sont dans `fenetre::notifications`.

/// L'identifiant de l'appli pour Windows : `Aria.<Nom>`.
pub fn identifiant(nom_appli: &str) -> String {
    format!("Aria.{nom_appli}")
}

/// Le plus de caractères gardés pour un titre, puis pour un texte : une notification se lit d'un coup d'œil.
pub const TITRE_MAX: usize = 80;
pub const TEXTE_MAX: usize = 300;

fn couper(texte: &str, max: usize) -> String {
    let texte = texte.trim();
    if texte.chars().count() <= max {
        return texte.to_owned();
    }
    let mut court: String = texte.chars().take(max - 1).collect();
    court.push('…');
    court
}

fn echapper(texte: &str) -> String {
    let mut sortie = String::with_capacity(texte.len());
    for c in texte.chars() {
        match c {
            '&' => sortie.push_str("&amp;"),
            '<' => sortie.push_str("&lt;"),
            '>' => sortie.push_str("&gt;"),
            '"' => sortie.push_str("&quot;"),
            '\'' => sortie.push_str("&apos;"),
            // Les caractères de contrôle ne passent pas en XML ; un retour à la ligne devient une espace.
            c if c.is_control() => sortie.push(' '),
            c => sortie.push(c),
        }
    }
    sortie
}

/// Le contenu de la notification : un titre et un texte, coupés et échappés. Sans son, sans bouton : elle informe.
pub fn xml(titre: &str, texte: &str) -> String {
    format!(
        "<toast><visual><binding template=\"ToastGeneric\"><text>{}</text><text>{}</text></binding></visual>\
         <audio silent=\"true\"/></toast>",
        echapper(&couper(titre, TITRE_MAX)),
        echapper(&couper(texte, TEXTE_MAX))
    )
}

/// Une image RGBA en PNG, sans compression (blocs « stockés ») : de quoi donner à Windows l'icône de l'appli sans
/// ajouter de dépendance. `None` si la taille ne correspond pas aux pixels.
pub fn png(largeur: u32, hauteur: u32, rgba: &[u8]) -> Option<Vec<u8>> {
    let ligne = usize::try_from(largeur).ok()?.checked_mul(4)?;
    let lignes = usize::try_from(hauteur).ok()?;
    if largeur == 0 || hauteur == 0 || rgba.len() != ligne.checked_mul(lignes)? {
        return None;
    }
    // Chaque ligne commence par son filtre (0 : aucun).
    let mut brut = Vec::with_capacity((ligne + 1) * lignes);
    for morceau in rgba.chunks(ligne) {
        brut.push(0);
        brut.extend_from_slice(morceau);
    }
    // zlib : en-tête, blocs stockés de 65 535 octets au plus, puis l'Adler-32 du contenu.
    let mut zlib = vec![0x78, 0x01];
    let mut blocs = brut.chunks(0xFFFF).peekable();
    while let Some(bloc) = blocs.next() {
        zlib.push(u8::from(blocs.peek().is_none()));
        let longueur = u16::try_from(bloc.len()).ok()?;
        zlib.extend_from_slice(&longueur.to_le_bytes());
        zlib.extend_from_slice(&(!longueur).to_le_bytes());
        zlib.extend_from_slice(bloc);
    }
    zlib.extend_from_slice(&adler32(&brut).to_be_bytes());

    let mut entete = Vec::with_capacity(13);
    entete.extend_from_slice(&largeur.to_be_bytes());
    entete.extend_from_slice(&hauteur.to_be_bytes());
    // 8 bits par canal, RGBA, compression, filtre et entrelacement standard.
    entete.extend_from_slice(&[8, 6, 0, 0, 0]);

    let mut sortie = vec![0x89, b'P', b'N', b'G', 0x0D, 0x0A, 0x1A, 0x0A];
    for (genre, donnees) in [(*b"IHDR", entete), (*b"IDAT", zlib), (*b"IEND", Vec::new())] {
        sortie.extend_from_slice(&u32::try_from(donnees.len()).ok()?.to_be_bytes());
        let debut = sortie.len();
        sortie.extend_from_slice(&genre);
        sortie.extend_from_slice(&donnees);
        let crc = crc32(&sortie[debut..]);
        sortie.extend_from_slice(&crc.to_be_bytes());
    }
    Some(sortie)
}

fn crc32(octets: &[u8]) -> u32 {
    let mut crc = 0xFFFF_FFFF_u32;
    for &octet in octets {
        crc ^= u32::from(octet);
        for _ in 0..8 {
            crc = if crc & 1 == 1 {
                (crc >> 1) ^ 0xEDB8_8320
            } else {
                crc >> 1
            };
        }
    }
    !crc
}

fn adler32(octets: &[u8]) -> u32 {
    let (mut a, mut b) = (1_u32, 0_u32);
    for &octet in octets {
        a = (a + u32::from(octet)) % 65_521;
        b = (b + a) % 65_521;
    }
    (b << 16) | a
}

/// L'inscription de l'appli auprès de Windows, dans `HKCU\Software\Classes\AppUserModelId`.
pub mod inscription {
    use std::io;
    use std::path::Path;

    /// Où écrire : `Software\Classes\AppUserModelId` pour de vrai, une clé à part dans les tests.
    #[derive(Debug, Clone)]
    pub struct Cles {
        pub racine: String,
    }

    impl Cles {
        pub fn utilisateur() -> Self {
            Self {
                racine: r"Software\Classes\AppUserModelId".to_owned(),
            }
        }
    }

    /// Inscrit (ou met à jour) le nom affiché et l'icône de l'appli.
    ///
    /// # Errors
    /// Le registre qui ne s'écrit pas.
    #[cfg(windows)]
    pub fn inscrire(cles: &Cles, identifiant: &str, nom: &str, icone: Option<&Path>) -> io::Result<()> {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        let (cle, _) =
            RegKey::predef(HKEY_CURRENT_USER).create_subkey(format!(r"{}\{identifiant}", cles.racine))?;
        cle.set_value("DisplayName", &nom)?;
        match icone {
            Some(icone) => cle.set_value("IconUri", &icone.display().to_string()),
            None => match cle.delete_value("IconUri") {
                Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(()),
                autre => autre,
            },
        }
    }

    /// Retire l'inscription ; rien à faire si elle n'existe pas.
    ///
    /// # Errors
    /// Le registre qui ne s'écrit pas.
    #[cfg(windows)]
    pub fn retirer(cles: &Cles, identifiant: &str) -> io::Result<bool> {
        use winreg::RegKey;
        use winreg::enums::HKEY_CURRENT_USER;
        match RegKey::predef(HKEY_CURRENT_USER).delete_subkey_all(format!(r"{}\{identifiant}", cles.racine)) {
            Ok(()) => Ok(true),
            Err(e) if e.kind() == io::ErrorKind::NotFound => Ok(false),
            Err(e) => Err(e),
        }
    }

    #[cfg(not(windows))]
    pub fn inscrire(_cles: &Cles, _identifiant: &str, _nom: &str, _icone: Option<&Path>) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows seulement"))
    }

    #[cfg(not(windows))]
    pub fn retirer(_cles: &Cles, _identifiant: &str) -> io::Result<bool> {
        Ok(false)
    }

    #[cfg(all(test, windows))]
    mod tests {
        use super::*;

        #[test]
        fn inscrire_puis_retirer() {
            let cles = Cles {
                racine: format!(r"Software\Aria-Tests\notifications-{}", std::process::id()),
            };
            inscrire(&cles, "Aria.Essai", "Aria Essai", None).unwrap();
            assert!(retirer(&cles, "Aria.Essai").unwrap());
            assert!(!retirer(&cles, "Aria.Essai").unwrap());
            let _ = winreg::RegKey::predef(winreg::enums::HKEY_CURRENT_USER).delete_subkey_all(&cles.racine);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn le_contenu_est_echappe_et_coupe() {
        let contenu = xml("Rapport <prêt> & \"envoyé\"", &"x".repeat(400));
        assert!(
            contenu.contains("Rapport &lt;prêt&gt; &amp; &quot;envoyé&quot;"),
            "{contenu}"
        );
        assert!(contenu.contains(&format!("{}…", "x".repeat(TEXTE_MAX - 1))));
        assert!(!contenu.contains(&"x".repeat(TEXTE_MAX)));
        assert!(xml("a\nb", "c").contains("<text>a b</text>"));
    }

    #[test]
    fn sommes_de_controle_connues() {
        assert_eq!(crc32(b"123456789"), 0xCBF4_3926);
        assert_eq!(adler32(b"Wikipedia"), 0x11E6_0398);
    }

    #[test]
    fn le_png_a_sa_forme() {
        let image = png(2, 2, &[255; 16]).unwrap();
        assert_eq!(&image[..8], b"\x89PNG\r\n\x1a\n");
        assert_eq!(&image[12..16], b"IHDR");
        assert_eq!(&image[image.len() - 8..image.len() - 4], b"IEND");
        assert!(png(2, 2, &[0; 15]).is_none());
        assert!(png(0, 0, &[]).is_none());
        // Une grande image passe par plusieurs blocs stockés.
        assert!(png(256, 256, &vec![7; 256 * 256 * 4]).is_some());
    }

    #[test]
    fn identifiant_de_l_appli() {
        assert_eq!(identifiant("Atlas"), "Aria.Atlas");
    }
}
