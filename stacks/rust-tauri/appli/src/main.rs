//! Aria {{NOM}} : l'exe Windows. Il prépare le dossier de données, ouvre la fenêtre Tauri et expose à l'interface
//! les seules commandes listées dans build.rs.

// Pas de console derrière la fenêtre, sauf en débogage.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commandes;
mod disque;
mod erreur_fatale;

use aria_coeur::{Dossier, identite, journal};
use tauri::window::Color;
use tauri::{AppHandle, Manager, RunEvent, Theme, WebviewUrl, WebviewWindowBuilder};

/// L'étiquette de la fenêtre principale, citée par capabilities\principale.json.
pub const FENETRE: &str = "principale";

/// Ce que les commandes partagent.
pub struct Etat {
    pub dossier: Dossier,
}

fn main() {
    if let Err(message) = lancer() {
        erreur_fatale::afficher(&message);
        std::process::exit(1);
    }
}

fn lancer() -> Result<(), String> {
    let dossier = Dossier::du_profil().map_err(|e| e.to_string())?;
    dossier.preparer().map_err(|e| e.to_string())?;
    journal::consigner(
        &dossier,
        "lancement",
        &format!("{} {}", identite::NOM, identite::VERSION),
    );
    // Contrat : WebView2 range ses données dans le dossier de l'appli, jamais dans %LOCALAPPDATA%.
    let cache_webview = dossier.cache_webview();

    tauri::Builder::default()
        // Toujours le premier greffon : relancer l'exe ramène la fenêtre existante au lieu d'en ouvrir une autre.
        .plugin(tauri_plugin_single_instance::init(|app, _arguments, _dossier| {
            ramener(app);
        }))
        .manage(Etat { dossier })
        .invoke_handler(tauri::generate_handler![
            commandes::identite,
            commandes::lire_reglages,
            commandes::enregistrer_reglages,
            commandes::mesurer_disque,
        ])
        .setup(move |app| {
            WebviewWindowBuilder::new(app, FENETRE, WebviewUrl::default())
                .title(identite::NOM)
                .inner_size(1100.0, 740.0)
                .min_inner_size(900.0, 600.0)
                .center()
                .theme(Some(Theme::Dark))
                // La couleur du fond de l'appli : pas d'éclair blanc avant le premier affichage.
                .background_color(Color(3, 8, 18, 255))
                .data_directory(cache_webview)
                .build()?;
            Ok(())
        })
        .build(tauri::generate_context!())
        .map_err(|e| format!("l'interface n'a pas pu démarrer : {e}"))?
        .run(|app, evenement| {
            if let RunEvent::Exit = evenement {
                journal::consigner(&app.state::<Etat>().dossier, "arrêt", "appli fermée");
            }
        });
    Ok(())
}

/// Ramène la fenêtre au premier plan, même réduite ou cachée.
fn ramener(app: &AppHandle) {
    if let Some(fenetre) = app.get_webview_window(FENETRE) {
        // Chaque étape est indépendante : si l'une échoue, les autres ramènent quand même la fenêtre.
        let _ = fenetre.unminimize();
        let _ = fenetre.show();
        let _ = fenetre.set_focus();
    }
}
