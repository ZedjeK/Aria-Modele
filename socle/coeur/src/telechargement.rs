//! Le téléchargement vérifié (décision 0009, contrat « Modules en option ») : une source épinglée en HTTPS, la taille
//! et l'empreinte SHA-256 connues d'avance, contrôlées avant tout usage, et la reprise d'un téléchargement coupé. Le
//! fichier n'apparaît sous son nom qu'une fois vérifié ; avant, il s'appelle `<nom>.partiel`.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use sha2::{Digest, Sha256};
use ureq::Agent;
use ureq::tls::{RootCerts, TlsConfig, TlsProvider};

use crate::identite;

/// Ce qu'on attend d'un téléchargement : d'où, combien d'octets, et quelle empreinte.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Attendu {
    /// L'adresse, en HTTPS seulement, annoncée dans la fiche de l'appli.
    pub adresse: String,
    pub taille: u64,
    /// L'empreinte SHA-256, en hexadécimal.
    pub sha256: String,
}

/// Le fichier partiel d'un téléchargement vers `cible`.
pub fn partiel(cible: &Path) -> PathBuf {
    let mut nom = cible.as_os_str().to_owned();
    nom.push(".partiel");
    PathBuf::from(nom)
}

/// Vérifie la taille et l'empreinte d'un fichier.
///
/// # Errors
/// Le fichier ne se lit pas, ou n'est pas celui attendu.
pub fn verifier(fichier: &Path, attendu: &Attendu) -> Result<(), String> {
    let taille = fs::metadata(fichier)
        .map_err(|e| format!("{} : {e}", fichier.display()))?
        .len();
    if taille != attendu.taille {
        return Err(format!("taille {taille} au lieu de {}", attendu.taille));
    }
    let mut empreinte = Sha256::new();
    let mut lecture = File::open(fichier).map_err(|e| e.to_string())?;
    let mut tampon = vec![0u8; 256 * 1024];
    loop {
        let lu = lecture.read(&mut tampon).map_err(|e| e.to_string())?;
        if lu == 0 {
            break;
        }
        empreinte.update(&tampon[..lu]);
    }
    let obtenue = hexa(&empreinte.finalize());
    if !obtenue.eq_ignore_ascii_case(attendu.sha256.trim()) {
        return Err(format!(
            "empreinte {obtenue} au lieu de {}",
            attendu.sha256.trim()
        ));
    }
    Ok(())
}

/// Des octets en hexadécimal minuscule.
fn hexa(octets: &[u8]) -> String {
    octets.iter().map(|o| format!("{o:02x}")).collect()
}

/// Le client HTTPS : HTTPS seulement, certificats et proxy de Windows.
fn client() -> Agent {
    Agent::config_builder()
        .https_only(true)
        .timeout_connect(Some(Duration::from_secs(30)))
        .user_agent(format!("{}/{}", identite::nom_exe(), identite::version()))
        .tls_config(
            TlsConfig::builder()
                .provider(TlsProvider::NativeTls)
                .root_certs(RootCerts::PlatformVerifier)
                .build(),
        )
        .build()
        .into()
}

/// Télécharge `attendu` vers `cible`, en reprenant un `.partiel` laissé par un téléchargement coupé. `avancer` reçoit
/// les octets reçus et le total ; `annulee` est lue entre deux morceaux, et une annulation garde le partiel pour une
/// reprise. Le fichier n'est renommé en `cible` qu'une fois sa taille et son empreinte vérifiées.
///
/// # Errors
/// Adresse hors HTTPS, réponse refusée, annulation, fichier trop long, ou fichier différent de celui attendu (le
/// partiel est alors effacé : il ne servirait à rien de le reprendre).
pub fn telecharger(
    attendu: &Attendu,
    cible: &Path,
    mut avancer: impl FnMut(u64, u64),
    annulee: impl Fn() -> bool,
) -> Result<(), String> {
    if !attendu.adresse.starts_with("https://") {
        return Err(format!("adresse refusée, HTTPS seulement : {}", attendu.adresse));
    }
    if let Some(dossier) = cible.parent() {
        fs::create_dir_all(dossier).map_err(|e| format!("{} : {e}", dossier.display()))?;
    }
    let partiel = partiel(cible);
    let deja = fs::metadata(&partiel).map(|m| m.len()).unwrap_or(0);
    let deja = if deja >= attendu.taille { 0 } else { deja };
    let mut requete = client().get(&attendu.adresse);
    if deja > 0 {
        requete = requete.header("Range", format!("bytes={deja}-"));
    }
    let mut reponse = requete
        .call()
        .map_err(|e| format!("téléchargement impossible : {e}"))?;
    // 206 : le serveur reprend où on en était ; sinon il renvoie tout, et le partiel repart de zéro.
    let reprise = deja > 0 && reponse.status().as_u16() == 206;
    let mut sortie = OpenOptions::new()
        .create(true)
        .write(true)
        .append(reprise)
        .truncate(!reprise)
        .open(&partiel)
        .map_err(|e| format!("{} : {e}", partiel.display()))?;
    let mut recu = if reprise { deja } else { 0 };
    let mut flux = reponse.body_mut().as_reader();
    let mut tampon = vec![0u8; 256 * 1024];
    loop {
        if annulee() {
            return Err("téléchargement annulé ; il reprendra où il en était".to_owned());
        }
        let lu = flux
            .read(&mut tampon)
            .map_err(|e| format!("téléchargement coupé : {e}"))?;
        if lu == 0 {
            break;
        }
        recu += lu as u64;
        if recu > attendu.taille {
            drop(sortie);
            let _ = fs::remove_file(&partiel);
            return Err("le serveur envoie plus que la taille annoncée : fichier refusé".to_owned());
        }
        sortie.write_all(&tampon[..lu]).map_err(|e| e.to_string())?;
        avancer(recu, attendu.taille);
    }
    sortie.sync_all().map_err(|e| e.to_string())?;
    drop(sortie);
    if let Err(e) = verifier(&partiel, attendu) {
        let _ = fs::remove_file(&partiel);
        return Err(format!("fichier refusé : {e}"));
    }
    fs::rename(&partiel, cible).map_err(|e| format!("{} : {e}", cible.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn la_verification_regarde_la_taille_et_l_empreinte() {
        let temp = tempfile::tempdir().unwrap();
        let fichier = temp.path().join("m.gguf");
        fs::write(&fichier, "poids").unwrap();
        let sha = hexa(&Sha256::digest(b"poids"));
        let attendu = Attendu {
            adresse: "https://exemple.invalid/m.gguf".to_owned(),
            taille: 5,
            sha256: sha.to_uppercase(),
        };
        verifier(&fichier, &attendu).unwrap();
        assert!(
            verifier(
                &fichier,
                &Attendu {
                    taille: 6,
                    ..attendu.clone()
                }
            )
            .is_err()
        );
        assert!(
            verifier(
                &fichier,
                &Attendu {
                    sha256: "0".repeat(64),
                    ..attendu
                }
            )
            .is_err()
        );
    }

    #[test]
    fn une_adresse_hors_https_est_refusee() {
        let temp = tempfile::tempdir().unwrap();
        let attendu = Attendu {
            adresse: "http://exemple.invalid/m.gguf".to_owned(),
            taille: 1,
            sha256: "0".repeat(64),
        };
        let erreur = telecharger(&attendu, &temp.path().join("m"), |_, _| {}, || false).unwrap_err();
        assert!(erreur.contains("HTTPS"), "{erreur}");
    }

    #[test]
    fn le_partiel_porte_son_nom() {
        assert!(partiel(Path::new("modeles/m.gguf")).ends_with("m.gguf.partiel"));
    }
}
