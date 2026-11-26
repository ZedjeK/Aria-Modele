//! Aria {{NOM}} : l'exe Windows. Il prépare le dossier de données, ouvre la fenêtre Tauri et expose à l'interface
//! les seules commandes listées dans build.rs.

// Pas de console derrière la fenêtre, sauf en débogage.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commandes;
mod demarrage_auto;
mod desinstallation;
mod disque;
mod erreur_fatale;
mod icone;
mod installation;
mod mise_a_jour;

use std::path::PathBuf;
use std::sync::atomic::AtomicBool;

use aria_coeur::{Dossier, Reglages, demarrage, identite, journal};
use tauri::window::Color;
use tauri::{AppHandle, Manager, RunEvent, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent};

/// L'étiquette de la fenêtre principale, citée par capabilities\principale.json.
pub const FENETRE: &str = "principale";

/// Ce que les commandes partagent.
pub struct Etat {
    pub dossier: Dossier,
    /// Contrat : WebView2 range ses données dans le dossier de l'appli, jamais dans %LOCALAPPDATA%.
    pub cache_webview: PathBuf,
    /// Désinstallée, `programme\` reste à supprimer à la fermeture (voir `desinstallation::a_la_sortie`).
    pub desinstallee: AtomicBool,
}

fn main() {
    if let Err(message) = lancer() {
        erreur_fatale::afficher(&message);
        std::process::exit(1);
    }
}

fn lancer() -> Result<(), String> {
    let dossier = Dossier::du_profil().map_err(|e| e.to_string())?;
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // Copie de l'exe qui finit une désinstallation : ni fenêtre ni Tauri ; elle vide programme\ et s'arrête.
    if let Some(processus) = aria_coeur::desinstallation::processus_a_attendre(&arguments) {
        desinstallation::finir(&dossier, processus);
        return Ok(());
    }
    dossier.preparer().map_err(|e| e.to_string())?;
    journal::consigner(
        &dossier,
        "lancement",
        &format!("{} {}", identite::NOM, identite::VERSION),
    );
    // Restes d'un remplacement précédent, maintenant que l'ancienne version est arrêtée.
    aria_coeur::installation::nettoyer(&dossier.programme());
    // La copie qui a fini une désinstallation précédente : elle ne pouvait pas se supprimer elle-même.
    aria_coeur::desinstallation::nettoyer_aide(&dossier.cache());
    // Sans WebView2, la fenêtre ne peut pas s'afficher : l'appli le dit et propose la page de Microsoft.
    if !webview2_present() {
        journal::consigner(&dossier, "lancement", WEBVIEW2_ABSENT);
        erreur_fatale::webview2_absent();
        std::process::exit(1);
    }
    // Lancé à l'ouverture de session : l'icône seule, la fenêtre reste rangée jusqu'à ce qu'on l'ouvre.
    let silencieux = identite::EN_FOND && demarrage::est_silencieux(&arguments);
    // Exe lancé hors de programme\ (zip décompressé) : proposition de s'installer, avant la fenêtre de l'appli.
    let proposer = !silencieux
        && std::env::current_exe().is_ok_and(|exe| {
            aria_coeur::installation::doit_proposer(&exe, &dossier, &Reglages::charger(&dossier), &arguments)
        });
    let cache_webview = dossier.cache_webview();

    tauri::Builder::default()
        // Toujours le premier greffon : relancer l'exe ramène la fenêtre existante au lieu d'en ouvrir une autre.
        // Un second lancement silencieux (ouverture de session) ne fait pas surgir la fenêtre.
        .plugin(tauri_plugin_single_instance::init(|app, arguments, _dossier| {
            if !demarrage::est_silencieux(arguments.iter().skip(1)) {
                ramener(app);
            }
        }))
        .manage(Etat {
            dossier,
            cache_webview,
            desinstallee: AtomicBool::new(false),
        })
        .manage(icone::Presence::default())
        .manage(mise_a_jour::Disponible::default())
        .invoke_handler(tauri::generate_handler![
            commandes::identite,
            commandes::lire_reglages,
            commandes::enregistrer_reglages,
            commandes::mesurer_disque,
            commandes::lire_journal,
            commandes::quitter,
            demarrage_auto::lire_demarrage_auto,
            demarrage_auto::regler_demarrage_auto,
            desinstallation::desinstaller,
            desinstallation::fermer_apres_desinstallation,
            icone::etat_menu,
            icone::action_menu,
            icone::taille_menu,
            installation::infos_installation,
            installation::installer,
            installation::lancer_sans_installer,
            mise_a_jour::mise_a_jour_disponible,
            mise_a_jour::voir_notes,
            mise_a_jour::installer_mise_a_jour,
        ])
        .setup(move |app| {
            if proposer {
                installation::ouvrir_fenetre(app.handle())?;
            } else {
                ouvrir_principale(app.handle(), !silencieux)?;
            }
            Ok(())
        })
        .build(tauri::generate_context!())
        .map_err(|e| format!("l'interface n'a pas pu démarrer : {e}"))?
        .run(|app, evenement| {
            if let RunEvent::Exit = evenement {
                let etat = app.state::<Etat>();
                journal::consigner(&etat.dossier, "arrêt", "appli fermée");
                desinstallation::a_la_sortie(&etat);
            }
        });
    Ok(())
}

/// La ligne du journal quand WebView2 manque.
const WEBVIEW2_ABSENT: &str = "WebView2 absent : l'appli ne peut pas s'ouvrir";

/// WebView2 est-il installé ? Par la méthode que documente Microsoft, `GetAvailableCoreWebView2BrowserVersionString`
/// (appelée par `tauri::webview_version`) : sans moteur, pas de version.
fn webview2_present() -> bool {
    tauri::webview_version().is_ok_and(|version| !version.trim().is_empty())
}

/// Ouvre la fenêtre de l'appli et, pour une appli qui sert en arrière-plan, son icône de notification.
pub fn ouvrir_principale(app: &AppHandle, visible: bool) -> tauri::Result<()> {
    let cache_webview = app.state::<Etat>().cache_webview.clone();
    let fenetre = WebviewWindowBuilder::new(app, FENETRE, WebviewUrl::default())
        .title(identite::NOM)
        .inner_size(1100.0, 740.0)
        .min_inner_size(900.0, 600.0)
        .center()
        // Sans cadre de Windows : la barre de titre est celle de l'interface (BarreTitre.svelte).
        .decorations(false)
        .visible(visible)
        .theme(Some(Theme::Dark))
        // La couleur du fond de l'appli : pas d'éclair blanc avant le premier affichage.
        .background_color(Color(3, 8, 18, 255))
        .data_directory(cache_webview.clone())
        .build()?;
    if identite::EN_FOND {
        icone::creer(app, cache_webview)?;
        // Fermer la fenêtre ne suffit pas à arrêter l'appli : elle reste dans son icône.
        let rangee = fenetre.clone();
        fenetre.on_window_event(move |evenement| {
            if let WindowEvent::CloseRequested { api, .. } = evenement {
                api.prevent_close();
                let _ = rangee.hide();
            }
        });
    }
    // Une requête vers GitHub, seulement si le réglage le permet ; aucune installation sans accord.
    let dossier = app.state::<Etat>().dossier.clone();
    mise_a_jour::chercher_au_lancement(app, &dossier);
    Ok(())
}

/// Ramène au premier plan la fenêtre ouverte (celle de l'appli, ou la proposition d'installation), même réduite
/// ou cachée.
fn ramener(app: &AppHandle) {
    let fenetre = app
        .get_webview_window(FENETRE)
        .or_else(|| app.get_webview_window(installation::FENETRE_INSTALLATION));
    if let Some(fenetre) = fenetre {
        // Chaque étape est indépendante : si l'une échoue, les autres ramènent quand même la fenêtre.
        let _ = fenetre.unminimize();
        let _ = fenetre.show();
        let _ = fenetre.set_focus();
    }
}
