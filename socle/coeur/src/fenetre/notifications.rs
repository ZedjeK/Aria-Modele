//! Module « Notifications », côté fenêtre (fonction `notifications`) : une notification de Windows pour un événement
//! que l'utilisateur doit voir alors qu'il regarde ailleurs.
//!
//! - **Réglage** : `notifications` dans `reglages.json` (case des Paramètres) ; décoché, rien n'est montré.
//! - **Pas quand la fenêtre est devant** : l'utilisateur voit déjà la ligne d'état de l'appli ; [`notifier`] rend
//!   alors `false` et ne montre rien.
//! - **Un clic** sur la notification ramène la fenêtre.
//!
//! Les appels à Windows passent par un seul fil, préparé pour COM une fois pour toutes : peu importe le fil d'où vient
//! la demande.
//!
//! ```ignore
//! notifications::notifier(&app, "Analyse finie", "12 fichiers analysés, 2 à revoir.")?;
//! ```

use std::sync::mpsc::{self, Sender};
use std::sync::{Mutex, OnceLock, PoisonError};

use tauri::{AppHandle, Manager, Wry, ipc::Invoke};

use super::{Etat, FENETRE};
use crate::notifications::{self, inscription};
use crate::{Reglages, identite, journal};

/// Les commandes du module, présentes seulement avec la fonction `notifications` (voir `liste.rs`).
pub fn commandes() -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![commande::notifier]
}

/// Une demande au fil des notifications, et où rendre la réponse.
struct Demande {
    contenu: String,
    reponse: Sender<Result<(), String>>,
}

/// Le fil des notifications, lancé à la première.
static FIL: OnceLock<Mutex<Sender<Demande>>> = OnceLock::new();

fn fil(app: &AppHandle) -> &'static Mutex<Sender<Demande>> {
    FIL.get_or_init(|| {
        let (envoi, reception) = mpsc::channel::<Demande>();
        let app = app.clone();
        std::thread::spawn(move || windows_seul::travailler(&app, &reception));
        Mutex::new(envoi)
    })
}

/// L'inscription auprès de Windows (nom et icône), faite à la première notification de chaque lancement.
static INSCRITE: OnceLock<Result<(), String>> = OnceLock::new();

fn inscrire(app: &AppHandle) -> Result<(), String> {
    INSCRITE
        .get_or_init(|| {
            let dossier = &app.state::<Etat>().dossier;
            // L'icône de la fenêtre, en PNG dans le cache : Windows la montre à côté de la notification.
            let icone = app.default_window_icon().and_then(|image| {
                let png = notifications::png(image.width(), image.height(), image.rgba())?;
                let chemin = dossier.cache().join("notification.png");
                crate::donnees::ecrire_entier(&chemin, &png).ok()?;
                Some(chemin)
            });
            inscription::inscrire(
                &inscription::Cles::utilisateur(),
                &notifications::identifiant(identite::identite().nom),
                identite::nom(),
                icone.as_deref(),
            )
            .map_err(|e| format!("inscription auprès de Windows impossible : {e}"))
        })
        .clone()
}

/// La fenêtre principale est-elle visible et devant ?
fn fenetre_devant(app: &AppHandle) -> bool {
    app.get_webview_window(FENETRE).is_some_and(|f| {
        f.is_visible().unwrap_or(false)
            && !f.is_minimized().unwrap_or(false)
            && f.is_focused().unwrap_or(false)
    })
}

/// Montre une notification de Windows : un titre et un texte courts (coupés au-delà de 80 et 300 caractères), sans
/// son. Rend `false` sans rien montrer si le réglage est décoché ou si la fenêtre est devant.
///
/// # Errors
/// L'inscription ou l'envoi à Windows ont échoué ; c'est aussi noté au journal.
pub fn notifier(app: &AppHandle, titre: &str, texte: &str) -> Result<bool, String> {
    let dossier = app.state::<Etat>().dossier.clone();
    if !Reglages::charger(&dossier).notifications || fenetre_devant(app) {
        return Ok(false);
    }
    let resultat = inscrire(app).and_then(|()| {
        let (reponse, attente) = mpsc::channel();
        fil(app)
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .send(Demande {
                contenu: notifications::xml(titre, texte),
                reponse,
            })
            .map_err(|_| "le fil des notifications s'est arrêté".to_owned())?;
        attente
            .recv()
            .map_err(|_| "le fil des notifications s'est arrêté".to_owned())?
    });
    match resultat {
        Ok(()) => Ok(true),
        Err(e) => {
            journal::consigner(&dossier, "notifications", &format!("pas montrée : {e}"));
            Err(e)
        }
    }
}

/// La commande de l'interface, à part pour garder le nom `notifier`.
pub mod commande {
    use tauri::AppHandle;

    /// [`super::notifier`], depuis l'interface : la fin d'une tâche que l'utilisateur attend, par exemple.
    #[tauri::command(async)]
    pub fn notifier(app: AppHandle, titre: String, texte: String) -> Result<bool, String> {
        super::notifier(&app, &titre, &texte)
    }
}

/// Retire l'inscription auprès de Windows (désinstallation).
///
/// # Errors
/// Le registre qui ne s'écrit pas.
pub fn retirer_inscription() -> Result<String, String> {
    let identifiant = notifications::identifiant(identite::identite().nom);
    match inscription::retirer(&inscription::Cles::utilisateur(), &identifiant) {
        Ok(true) => Ok("inscription auprès de Windows retirée".to_owned()),
        Ok(false) => Ok("aucune inscription".to_owned()),
        Err(e) => Err(format!("pas retirée : {e}")),
    }
}

#[cfg(windows)]
mod windows_seul {
    use std::collections::VecDeque;
    use std::sync::mpsc::Receiver;

    use tauri::AppHandle;
    use windows::Data::Xml::Dom::XmlDocument;
    use windows::Foundation::TypedEventHandler;
    use windows::UI::Notifications::{ToastNotification, ToastNotificationManager};
    use windows::Win32::System::Com::{COINIT_MULTITHREADED, CoInitializeEx};
    use windows::core::HSTRING;

    use super::Demande;
    use crate::{identite, notifications};

    /// Les notifications gardées en vie, pour que leur clic arrive encore.
    const GARDEES: usize = 20;

    #[allow(unsafe_code)]
    pub fn travailler(app: &AppHandle, reception: &Receiver<Demande>) {
        // SAFETY : ce fil ne sert qu'aux notifications ; COM y est préparé une fois, en multithread, et y reste
        // jusqu'à la fin de l'appli.
        let pret = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) }.ok();
        let identifiant = HSTRING::from(notifications::identifiant(identite::identite().nom));
        let mut gardees: VecDeque<ToastNotification> = VecDeque::new();
        for demande in reception {
            let resultat = pret
                .clone()
                .and_then(|()| montrer(app, &identifiant, &demande.contenu))
                .map(|toast| {
                    gardees.push_back(toast);
                    if gardees.len() > GARDEES {
                        gardees.pop_front();
                    }
                })
                .map_err(|e| e.to_string());
            let _ = demande.reponse.send(resultat);
        }
    }

    fn montrer(
        app: &AppHandle,
        identifiant: &HSTRING,
        contenu: &str,
    ) -> windows::core::Result<ToastNotification> {
        let document = XmlDocument::new()?;
        document.LoadXml(&HSTRING::from(contenu))?;
        let toast = ToastNotification::CreateToastNotification(&document)?;
        let app = app.clone();
        toast.Activated(&TypedEventHandler::new(move |_, _| {
            super::super::ramener(&app);
            Ok(())
        }))?;
        ToastNotificationManager::CreateToastNotifierWithId(identifiant)?.Show(&toast)?;
        Ok(toast)
    }
}

#[cfg(not(windows))]
mod windows_seul {
    use std::sync::mpsc::Receiver;

    use tauri::AppHandle;

    use super::Demande;

    pub fn travailler(_app: &AppHandle, reception: &Receiver<Demande>) {
        for demande in reception {
            let _ = demande
                .reponse
                .send(Err("notifications : Windows seulement".to_owned()));
        }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn le_gestionnaire_suit_la_liste() {
        let source = include_str!("notifications.rs");
        let debut = source.find("generate_handler![").unwrap() + "generate_handler![".len();
        let fin = debut + source[debut..].find(']').unwrap();
        let mut gestionnaire: Vec<&str> = source[debut..fin]
            .split(',')
            .filter_map(|c| c.trim().rsplit("::").next())
            .filter(|c| !c.is_empty())
            .collect();
        let mut liste = super::super::liste::COMMANDES_NOTIFICATIONS.to_vec();
        gestionnaire.sort_unstable();
        liste.sort_unstable();
        assert_eq!(gestionnaire, liste);
    }
}
