//! « Désinstaller », dans les Paramètres (vue `Parametres.svelte`) : retire le démarrage automatique, le raccourci
//! du Bureau s'il pointe vers l'appli, puis `programme\` : tout de suite si l'exe en cours n'en vient pas, sinon par
//! une copie de l'exe, une fois l'appli fermée (par « Fermer l'appli », ou de toute autre façon : voir
//! [`a_la_sortie`]). Les données restent, sauf si l'utilisateur coche « Supprimer aussi mes données » : la copie de
//! l'exe efface alors tout le dossier de données une fois l'appli fermée. Le dossier commun `.Aria` n'est jamais
//! touché.

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
use super::raccourci;

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
const EFFACEES_A_LA_FERMETURE: &str = "seront effacées une fois l'appli fermée";

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
    /// `programme\` ou les données seront supprimés une fois l'appli fermée : l'interface propose de la fermer.
    a_la_fermeture: bool,
    /// Le dossier de données sera effacé une fois l'appli fermée.
    donnees_effacees: bool,
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
/// fermée). Chaque étape est tentée même si une autre échoue ; le bilan dit laquelle. Les données restent, sauf avec
/// `effacer_donnees` (case cochée par l'utilisateur) : elles sont alors effacées une fois l'appli fermée.
// Sur un fil à part : la copie ne fige pas la fenêtre.
#[tauri::command(async)]
pub fn desinstaller(
    app: AppHandle,
    etat: State<'_, Etat>,
    effacer_donnees: bool,
) -> Result<BilanDesinstallation, String> {
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

    #[cfg(feature = "fichiers")]
    suivi.noter(
        "Associations de fichiers",
        super::fichiers::retirer_associations(),
    );

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

    let mut donnees_effacees = false;
    if effacer_donnees {
        let racine = etat.dossier.racine();
        let issue = if effacable(racine) {
            etat.effacer_donnees.store(true, Ordering::SeqCst);
            donnees_effacees = true;
            Ok(EFFACEES_A_LA_FERMETURE.to_owned())
        } else {
            Err(format!(
                "gardées : {} n'est pas un dossier de données d'appli Aria",
                racine.display()
            ))
        };
        suivi.noter("Données", issue);
    }

    Ok(BilanDesinstallation {
        etapes: suivi.etapes,
        donnees: etat.dossier.racine().display().to_string(),
        a_la_fermeture: sort == Programme::ALaFermeture || donnees_effacees,
        donnees_effacees,
    })
}

/// Le dossier de l'utilisateur, parent du dossier de données.
fn profil() -> Option<std::path::PathBuf> {
    std::env::var_os("USERPROFILE")
        .or_else(|| std::env::var_os("HOME"))
        .filter(|p| !p.is_empty())
        .map(std::path::PathBuf::from)
}

/// Le dossier de données est-il bien `<profil>\.Aria-<Nom>` de cette appli ? Sinon, rien ne s'efface.
fn effacable(racine: &Path) -> bool {
    racine.file_name().and_then(|n| n.to_str()) == Some(crate::identite::dossier_donnees())
        && profil().is_some_and(|p| desinstallation::racine_effacable(racine, &p))
}

/// Retire le raccourci du Bureau s'il pointe vers l'appli ; un raccourci vers autre chose reste.
fn retirer_raccourci(app: &AppHandle, exes: &[&Path]) -> Result<String, String> {
    let Some(lien) = raccourci::chemin(app) else {
        return Ok("Bureau introuvable, rien à retirer".to_owned());
    };
    if !lien.is_file() {
        return Ok("aucun raccourci".to_owned());
    }
    let cible = raccourci::cible(&lien).map_err(|e| format!("cible illisible, gardé : {e}"))?;
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
    confier(&etat.dossier, etat.effacer_donnees.load(Ordering::SeqCst))?;
    // Confié : la fermeture n'a plus à le faire.
    etat.desinstallee.store(false, Ordering::SeqCst);
    etat.effacer_donnees.store(false, Ordering::SeqCst);
    app.exit(0);
    Ok(())
}

/// À la fermeture de l'appli : désinstallée sans passer par « Fermer l'appli » (Quitter, croix), `programme\` est
/// confié maintenant à la copie de fin. Un échec est noté au journal : il n'y a plus de fenêtre pour le dire.
pub fn a_la_sortie(etat: &Etat) {
    let desinstallee = etat.desinstallee.swap(false, Ordering::SeqCst);
    let effacer = etat.effacer_donnees.swap(false, Ordering::SeqCst);
    if !desinstallee && !effacer {
        return;
    }
    if let Err(e) = confier(&etat.dossier, effacer) {
        journal::consigner(&etat.dossier, GENRE, &format!("Programme : {e}"));
    }
}

/// Lance la copie de l'exe qui videra `programme\` (si l'appli tourne depuis là) et, avec `effacer`, tout le dossier
/// de données, une fois l'appli fermée.
fn confier(dossier: &Dossier, effacer: bool) -> Result<(), String> {
    let exe = std::env::current_exe().map_err(|e| format!("L'exe de l'appli est introuvable : {e}"))?;
    let programme = dossier.programme();
    if desinstallation::sort_de_programme(&exe, &programme) != Programme::ALaFermeture && !effacer {
        return Ok(());
    }
    let a_effacer = if effacer {
        dossier.racine()
    } else {
        programme.as_path()
    };
    let a_la_main = format!(
        "Supprime {} à la main, une fois l'appli fermée.",
        a_effacer.display()
    );
    let echec = |e: std::io::Error| format!("Préparation impossible : {e}. {a_la_main}");
    let aide = desinstallation::preparer_aide(&exe, &dossier.cache()).map_err(echec)?;
    // La copie tourne seule : l'appli ne l'attend pas, elle se ferme juste après.
    let mut copie = Command::new(&aide);
    copie
        .arg(desinstallation::ARGUMENT_FINIR)
        .arg(std::process::id().to_string())
        .current_dir(dossier.racine());
    if effacer {
        copie.arg(desinstallation::ARGUMENT_EFFACER);
    }
    let _copie = copie.spawn().map_err(echec)?;
    journal::consigner(dossier, GENRE, CONFIE);
    Ok(())
}

/// La copie de l'exe, lancée avec `--finir-desinstallation <numéro>` : attend la fin de l'appli, vide
/// `programme\` et note le bilan au journal. Avec `effacer`, efface ensuite tout le dossier de données, elle comprise
/// (par `cmd`, une fois arrêtée). Seulement depuis `cache\desinstallation\` ; ailleurs, rien.
pub fn finir(dossier: &Dossier, processus: u32, effacer: bool) {
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
    if effacer {
        effacer_donnees(dossier);
    }
}

/// Efface le dossier de données : tout de suite, sauf la copie de fin, qui tourne ; puis le dossier entier, par
/// `cmd`, une fois la copie arrêtée. Rien n'est effacé si le dossier n'est pas celui d'une appli Aria.
fn effacer_donnees(dossier: &Dossier) {
    let racine = dossier.racine();
    let Some(profil) = profil().filter(|_| effacable(racine)) else {
        journal::consigner(
            dossier,
            GENRE,
            "Données : gardées, le dossier n'est pas celui d'une appli Aria",
        );
        return;
    };
    let restes = desinstallation::effacer_donnees_sauf_aide(racine);
    let Some(commande) = desinstallation::commande_effacement(racine) else {
        // Le journal a déjà été effacé : il est recréé pour dire ce qui reste.
        journal::consigner(
            dossier,
            GENRE,
            "Données : effacées, sauf la copie de fin (chemin que cmd ne peut pas lire) : supprime le dossier à la main",
        );
        return;
    };
    // Cette ligne part avec le dossier si cmd l'efface ; elle ne reste que si l'effacement a échoué.
    let mut note = format!(
        "Données : pas entièrement effacées, supprime {} à la main",
        racine.display()
    );
    if !restes.is_empty() {
        note.push_str(&format!(
            " (encore utilisés au premier essai : {})",
            restes.join(" ; ")
        ));
    }
    if let Err(e) = systeme::effacer_apres(&commande, &profil) {
        note.push_str(&format!(" (cmd n'a pas pu être lancé : {e})"));
    }
    journal::consigner(dossier, GENRE, &note);
}

/// Ce que la désinstallation demande à Windows.
#[cfg(windows)]
mod systeme {
    use std::io;
    use std::time::Duration;

    use windows::Win32::Foundation::{CloseHandle, ERROR_INVALID_PARAMETER, WAIT_OBJECT_0};
    use windows::Win32::System::Threading::{OpenProcess, PROCESS_SYNCHRONIZE, WaitForSingleObject};

    /// Retire le démarrage automatique : `Run` et `StartupApproved`.
    pub fn oublier_demarrage() -> io::Result<()> {
        crate::demarrage::windows::oublier()
    }

    /// Lance `cmd` avec cette ligne, sans fenêtre, depuis `dossier`, sans l'attendre.
    pub fn effacer_apres(ligne: &str, dossier: &std::path::Path) -> io::Result<()> {
        use std::os::windows::process::CommandExt;
        /// Pas de console pour cmd.
        const CREATE_NO_WINDOW: u32 = 0x0800_0000;
        let systeme = std::env::var_os("SystemRoot").map_or_else(
            || std::path::PathBuf::from(r"C:\Windows"),
            std::path::PathBuf::from,
        );
        std::process::Command::new(systeme.join("System32").join("cmd.exe"))
            .raw_arg(ligne)
            .current_dir(dossier)
            .creation_flags(CREATE_NO_WINDOW)
            .spawn()
            .map(|_| ())
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

/// Hors de Windows (tests, outils) : ni registre ni attente.
#[cfg(not(windows))]
mod systeme {
    use std::io;
    use std::time::Duration;

    pub fn oublier_demarrage() -> io::Result<()> {
        Ok(())
    }

    pub fn attendre_fin(_processus: u32, _delai: Duration) -> bool {
        true
    }

    pub fn effacer_apres(_ligne: &str, _dossier: &std::path::Path) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::Unsupported, "Windows seulement"))
    }
}
