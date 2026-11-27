//! Le raccourci du Bureau, « Aria Nom.lnk » : le lire, le créer ou le mettre à jour. Un raccourci du même nom qui
//! pointe vers un autre programme n'est jamais remplacé ni retiré (contrat, « Installation »).

use std::io;
use std::path::{Path, PathBuf};

use serde::Serialize;
use tauri::{AppHandle, Manager};

use crate::identite;

/// Le chemin du raccourci du Bureau de l'utilisateur ; `None` si le Bureau est introuvable.
pub fn chemin(app: &AppHandle) -> Option<PathBuf> {
    app.path()
        .desktop_dir()
        .ok()
        .map(|bureau| bureau.join(format!("{}.lnk", identite::nom())))
}

/// Ce que l'on trouve à l'emplacement du raccourci.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Existant {
    /// Rien : le raccourci peut être créé.
    Absent,
    /// Un raccourci vers l'appli : il peut être mis à jour.
    ANous,
    /// Un raccourci vers un autre programme, ou illisible : il reste tel quel.
    Ailleurs,
}

/// Que trouve-t-on en `lien`, sachant que l'appli est l'un des `exes` (installé, en cours) ?
pub fn existant(lien: &Path, exes: &[&Path]) -> Existant {
    if !lien.is_file() {
        return Existant::Absent;
    }
    match systeme::cible(lien) {
        Ok(cible) if crate::desinstallation::raccourci_a_nous(&cible, exes) => Existant::ANous,
        _ => Existant::Ailleurs,
    }
}

/// Crée le raccourci `lien` vers `cible`, ou le met à jour s'il pointe déjà vers l'appli.
///
/// # Errors
/// Un raccourci du même nom pointe vers un autre programme (il reste tel quel), ou Windows refuse l'écriture.
pub fn creer_ou_mettre_a_jour(lien: &Path, cible: &Path, exes: &[&Path]) -> io::Result<()> {
    if existant(lien, exes) == Existant::Ailleurs {
        return Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            "un raccourci du même nom pointe vers un autre programme : il reste tel quel",
        ));
    }
    systeme::ecrire(lien, cible)
}

/// La cible d'un raccourci `.lnk`.
///
/// # Errors
/// Le raccourci ne se lit pas, ou hors de Windows.
pub fn cible(lien: &Path) -> io::Result<PathBuf> {
    systeme::cible(lien)
}

/// Ce que le raccourci demande à Windows.
#[cfg(windows)]
mod systeme {
    use std::io;
    use std::path::{Path, PathBuf};

    use crate::identite;
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
        IPersistFile, STGM_READ,
    };
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{HSTRING, Interface};

    /// Écrit le raccourci `lien` vers `cible`, avec l'icône de l'exe. Sur un fil à part, qui prépare COM pour lui
    /// seul.
    pub fn ecrire(lien: &Path, cible: &Path) -> io::Result<()> {
        let (lien, cible) = (HSTRING::from(lien), cible.to_path_buf());
        std::thread::spawn(move || ecrire_com(&lien, &cible))
            .join()
            .map_err(|_| io::Error::other("création du raccourci interrompue"))?
            .map_err(io::Error::other)
    }

    #[allow(unsafe_code)]
    fn ecrire_com(lien: &HSTRING, cible: &Path) -> windows::core::Result<()> {
        // SAFETY : COM est préparé puis libéré sur ce fil, qui ne sert qu'à ça ; les objets COM sont relâchés
        // avant CoUninitialize, à la fin de la fermeture ; les textes sont des HSTRING valides pendant les appels.
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let resultat = (|| {
                let raccourci: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
                raccourci.SetPath(&HSTRING::from(cible))?;
                if let Some(dossier) = cible.parent() {
                    raccourci.SetWorkingDirectory(&HSTRING::from(dossier))?;
                }
                raccourci.SetIconLocation(&HSTRING::from(cible), 0)?;
                raccourci.SetDescription(&HSTRING::from(identite::nom()))?;
                raccourci.cast::<IPersistFile>()?.Save(lien, true)
            })();
            CoUninitialize();
            resultat
        }
    }

    /// La cible d'un raccourci `.lnk`. Sur un fil à part, qui prépare COM pour lui seul.
    pub fn cible(lien: &Path) -> io::Result<PathBuf> {
        let lien = HSTRING::from(lien);
        let cible = std::thread::spawn(move || lire_cible(&lien))
            .join()
            .map_err(|_| io::Error::other("lecture du raccourci interrompue"))?
            .map_err(io::Error::other)?;
        if cible.as_os_str().is_empty() {
            return Err(io::Error::other("cible illisible"));
        }
        Ok(cible)
    }

    #[allow(unsafe_code)]
    fn lire_cible(lien: &HSTRING) -> windows::core::Result<PathBuf> {
        // SAFETY : COM est préparé puis libéré sur ce fil, qui ne sert qu'à ça ; les objets COM sont relâchés avant
        // CoUninitialize, à la fin de la fermeture ; le tampon vit pendant tout l'appel à GetPath, qui en reçoit la
        // taille ; aucune donnée de recherche n'est demandée (pointeur nul).
        unsafe {
            CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
            let resultat = (|| -> windows::core::Result<PathBuf> {
                let raccourci: IShellLinkW = CoCreateInstance(&ShellLink, None, CLSCTX_INPROC_SERVER)?;
                raccourci.cast::<IPersistFile>()?.Load(lien, STGM_READ)?;
                let mut tampon = vec![0u16; 32_768];
                raccourci.GetPath(&mut tampon, std::ptr::null_mut(), 0)?;
                let utile = tampon.split(|&c| c == 0).next().unwrap_or_default();
                Ok(PathBuf::from(String::from_utf16_lossy(utile)))
            })();
            CoUninitialize();
            resultat
        }
    }
}

/// Hors de Windows (tests, outils) : pas de raccourci.
#[cfg(not(windows))]
mod systeme {
    use std::io;
    use std::path::{Path, PathBuf};

    pub fn ecrire(_lien: &Path, _cible: &Path) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows seulement"))
    }

    pub fn cible(_lien: &Path) -> io::Result<PathBuf> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows seulement"))
    }
}
