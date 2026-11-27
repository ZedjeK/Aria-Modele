//! « Désinstaller », dans les Paramètres (vue `Parametres.svelte`) : retire le démarrage automatique, le raccourci
//! du Bureau s'il pointe vers l'appli, puis `programme\` : tout de suite si l'exe en cours n'en vient pas, sinon par
//! une copie de l'exe, une fois l'appli fermée (par « Fermer l'appli », ou de toute autre façon : voir
//! [`a_la_sortie`]). Les données restent ; l'interface dit où elles sont.

use std::fs;
use std::path::Path;
use std::process::Command;
use std::sync::atomic::Ordering;
use std::time::Duration;

use crate::desinstallation::{self, Programme};
use crate::installation::nom_exe;
use crate::{Dossier, journal};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

use super::Etat;
use super::icone::MENU;
use super::installation::chemin_raccourci;

/// Le genre des lignes du journal.
const GENRE: &str = "désinstallation";

/// Ce que la copie de fin attend au plus la fermeture de l'appli.
const ATTENTE_FERMETURE: Duration = Duration::from_secs(60);

/// Ce que la copie de fin reprend au plus la suppression d'un fichier encore verrouillé.
const REPRISE: Duration = Duration::from_secs(5);

const PORTABLE: &str = "aucune appli installée (mode portable)";
const A_LA_FERMETURE: &str = "sera supprimé une fois l'appli fermée";
const CONFIE: &str = "Programme : confié à une copie de l'exe, supprimé une fois l'appli fermée";
const HORS_DU_CACHE: &str = "Programme : rien supprimé, la copie de fin ne tourne pas depuis le cache";
const PAS_FERMEE: &str = "Programme : rien supprimé, l'appli ne s'est pas fermée à temps";

/// Une étape de la désinstallation, telle que l'interface la montre.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Etape {
    etape: &'static str,
    reussie: bool,
    bilan: String,
}

/// Ce que la désinstallation a fait.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct BilanDesinstallation {
    etapes: Vec<Etape>,
    /// Le dossier de données, gardé.
    donnees: String,
    /// `programme\` sera supprimé une fois l'appli fermée : l'interface propose de la fermer.
    a_la_fermeture: bool,
}

/// Les étapes faites, notées au journal au fur et à mesure.
struct Suivi<'a> {
    dossier: &'a Dossier,
    etapes: Vec<Etape>,
}

impl Suivi<'_> {
    fn noter(&mut self, etape: &'static str, issue: Result<String, String>) {
        let (reussie, bilan) = match issue {
            Ok(bilan) => (true, bilan),
            Err(bilan) => (false, bilan),
        };
        journal::consigner(self.dossier, GENRE, &format!("{etape} : {bilan}"));
        self.etapes.push(Etape {
            etape,
            reussie,
            bilan,
        });
    }
}

/// Désinstalle : démarrage automatique, raccourci du Bureau, puis `programme\` (tout de suite, ou une fois l'appli
/// fermée). Chaque étape est tentée même si une autre échoue ; le bilan dit laquelle. Les données restent.
#[tauri::command]
pub fn desinstaller(app: AppHandle, etat: State<'_, Etat>) -> Result<BilanDesinstallation, String> {
    let exe = std::env::current_exe().map_err(|e| format!("L'exe de l'appli est introuvable : {e}"))?;
    let programme = etat.dossier.programme();
    let mut suivi = Suivi {
        dossier: &etat.dossier,
        etapes: Vec::new(),
    };

    let demarrage = systeme::oublier_demarrage()
        .map(|()| "retiré".to_owned())
        .map_err(|e| format!("pas retiré : {e}"));
    suivi.noter("Démarrage avec Windows", demarrage);
    // Le menu de l'icône relit sa case « Démarrer avec Windows » ; sans icône, personne n'écoute.
    let _ = app.emit_to(MENU, "etat-icone", ());

    let installe = programme.join(nom_exe());
    let raccourci = retirer_raccourci(&app, &[installe.as_path(), exe.as_path()]);
    suivi.noter("Raccourci du Bureau", raccourci);

    let sort = desinstallation::sort_de_programme(&exe, &programme);
    let issue = match sort {
        Programme::Absent => Ok(PORTABLE.to_owned()),
        Programme::Maintenant => desinstallation::vider_programme(&programme).resume(&programme),
        Programme::ALaFermeture => Ok(A_LA_FERMETURE.to_owned()),
    };
    suivi.noter("Programme", issue);
    // Quelle que soit la façon dont l'appli se fermera, programme\ sera confié à la copie de fin.
    if sort == Programme::ALaFermeture {
        etat.desinstallee.store(true, Ordering::SeqCst);
    }

    Ok(BilanDesinstallation {
        etapes: suivi.etapes,
        donnees: etat.dossier.racine().display().to_string(),
        a_la_fermeture: sort == Programme::ALaFermeture,
    })
}

/// Retire le raccourci du Bureau s'il pointe vers l'appli ; un raccourci vers autre chose reste.
fn retirer_raccourci(app: &AppHandle, exes: &[&Path]) -> Result<String, String> {
    let Some(lien) = chemin_raccourci(app) else {
        return Ok("Bureau introuvable, rien à retirer".to_owned());
    };
    if !lien.is_file() {
        return Ok("aucun raccourci".to_owned());
    }
    let cible = systeme::cible_raccourci(&lien).map_err(|e| format!("cible illisible, gardé : {e}"))?;
    if cible.as_os_str().is_empty() {
        return Ok("cible illisible, gardé".to_owned());
    }
    if !desinstallation::raccourci_a_nous(&cible, exes) {
        return Ok("il pointe vers un autre programme : gardé".to_owned());
    }
    fs::remove_file(&lien).map_err(|e| format!("pas retiré : {e}"))?;
    Ok("retiré".to_owned())
}

/// Ferme l'appli après la désinstallation. Si elle tourne depuis `programme\`, une copie de l'exe, rangée dans le
/// cache, attend sa fermeture puis vide `programme\`.
#[tauri::command]
pub fn fermer_apres_desinstallation(app: AppHandle, etat: State<'_, Etat>) -> Result<(), String> {
    confier_programme(&etat.dossier)?;
    // Confié : la fermeture n'a plus à le faire.
    etat.desinstallee.store(false, Ordering::SeqCst);
    app.exit(0);
    Ok(())
}

/// À la fermeture de l'appli : désinstallée sans passer par « Fermer l'appli » (Quitter, croix), `programme\` est
/// confié maintenant à la copie de fin. Un échec est noté au journal : il n'y a plus de fenêtre pour le dire.
pub fn a_la_sortie(etat: &Etat) {
    if !etat.desinstallee.swap(false, Ordering::SeqCst) {
        return;
    }
    if let Err(e) = confier_programme(&etat.dossier) {
        journal::consigner(&etat.dossier, GENRE, &format!("Programme : {e}"));
    }
}

/// Si l'appli tourne depuis `programme\`, lance la copie de l'exe qui le videra une fois l'appli fermée.
fn confier_programme(dossier: &Dossier) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("L'exe de l'appli est introuvable : {e}"))?;
    let programme = dossier.programme();
    if desinstallation::sort_de_programme(&exe, &programme) != Programme::ALaFermeture {
        return Ok(());
    }
    let a_la_main = format!(
        "Supprime {} à la main, une fois l'appli fermée.",
        programme.display()
    );
    let echec = |e: std::io::Error| format!("Préparation impossible : {e}. {a_la_main}");
    let aide = desinstallation::preparer_aide(&exe, &dossier.cache()).map_err(echec)?;
    // La copie tourne seule : l'appli ne l'attend pas, elle se ferme juste après.
    let _copie = Command::new(&aide)
        .arg(desinstallation::ARGUMENT_FINIR)
        .arg(std::process::id().to_string())
        .current_dir(dossier.racine())
        .spawn()
        .map_err(echec)?;
    journal::consigner(dossier, GENRE, CONFIE);
    Ok(())
}

/// La copie de l'exe, lancée avec `--finir-desinstallation <numéro>` : attend la fin de l'appli, vide
/// `programme\` et note le bilan au journal. Seulement depuis `cache\desinstallation\` ; ailleurs, rien.
pub fn finir(dossier: &Dossier, processus: u32) {
    let cache = dossier.cache();
    let aide = std::env::current_exe().is_ok_and(|exe| desinstallation::est_l_aide(&exe, &cache));
    if !aide {
        journal::consigner(dossier, GENRE, HORS_DU_CACHE);
        return;
    }
    if !systeme::attendre_fin(processus, ATTENTE_FERMETURE) {
        journal::consigner(dossier, GENRE, PAS_FERMEE);
        return;
    }
    let programme = dossier.programme();
    let bilan = desinstallation::vider_programme_en_attendant(&programme, REPRISE);
    let texte = match bilan.resume(&programme) {
        Ok(texte) | Err(texte) => texte,
    };
    journal::consigner(dossier, GENRE, &format!("Programme : {texte}"));
}

/// Ce que la désinstallation demande à Windows.
#[cfg(windows)]
mod systeme {
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    use windows::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0};
    use windows::Win32::System::Com::{
        CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoUninitialize,
        IPersistFile, STGM_READ,
    };
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject};
    use windows::Win32::UI::Shell::{IShellLinkW, ShellLink};
    use windows::core::{HSTRING, Interface};

    /// Retire le démarrage automatique : `Run` et `StartupApproved`.
    pub fn oublier_demarrage() -> io::Result<()> {
        crate::demarrage::windows::oublier()
    }

    /// La cible d'un raccourci `.lnk`. Sur un fil à part, qui prépare COM pour lui seul.
    pub fn cible_raccourci(lien: &Path) -> io::Result<PathBuf> {
        let lien = HSTRING::from(lien);
        std::thread::spawn(move || lire_cible(&lien))
            .join()
            .map_err(|_| io::Error::other("lecture du raccourci interrompue"))?
            .map_err(io::Error::other)
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

    /// Attend la fin du processus, au plus `delai`. Vrai s'il s'est terminé, ou si son numéro n'existe plus (déjà
    /// terminé) ; faux s'il tourne encore ou ne s'ouvre pas pour une autre raison (accès refusé).
    #[allow(unsafe_code)]
    pub fn attendre_fin(processus: u32, delai: Duration) -> bool {
        let millisecondes = u32::try_from(delai.as_millis()).unwrap_or(u32::MAX);
        // SAFETY : le handle ouvert ici ne sert que dans ce bloc, puis il est fermé ; aucun tampon n'est passé.
        unsafe {
            let handle = match OpenProcess(PROCESS_SYNCHRONIZE, false, processus) {
                Ok(handle) => handle,
                Err(e) => return e.code() == ERROR_INVALID_PARAMETER.to_hresult(),
            };
            let attente = WaitForSingleObject(handle, millisecondes);
            // Un handle qui ne se ferme pas sera rendu à la fin du processus, juste après.
            let _ = CloseHandle(handle);
            attente == WAIT_OBJECT_0
        }
    }
}

/// Hors de Windows (tests, outils) : ni registre, ni raccourci, ni attente.
#[cfg(not(windows))]
mod systeme {
    use std::io;
    use std::path::{Path, PathBuf};
    use std::time::Duration;

    pub fn oublier_demarrage() -> io::Result<()> {
        Ok(())
    }

    pub fn cible_raccourci(_lien: &Path) -> io::Result<PathBuf> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows seulement"))
    }

    pub fn attendre_fin(_processus: u32, _delai: Duration) -> bool {
        true
    }
}
