//! Les commandes générales du socle que l'interface peut appeler. Chacune figure aussi dans `liste.rs` et dans la
//! capacité de sa fenêtre, sinon l'interface ne peut pas l'appeler.
//!
//! Les erreurs remontent en texte français : l'interface les montre telles quelles. Jamais de secret dedans.

use crate::journal::Operation;
use crate::{Reglages, diagnostic, identite, installation, journal};
use serde::Serialize;
use tauri::{AppHandle, State};

use super::Etat;

/// Ce que l'interface affiche de l'appli.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Identite {
    nom: &'static str,
    version: &'static str,
    promesse: &'static str,
    en_fond: bool,
    dossier: String,
}

#[tauri::command]
pub fn identite(etat: State<'_, Etat>) -> Identite {
    Identite {
        nom: identite::nom(),
        version: identite::version(),
        promesse: identite::promesse(),
        en_fond: identite::en_fond(),
        dossier: etat.dossier.racine().display().to_string(),
    }
}

#[tauri::command]
pub fn lire_reglages(etat: State<'_, Etat>) -> Reglages {
    Reglages::charger(&etat.dossier)
}

/// Enregistre les réglages et renvoie ceux qui ont vraiment été écrits (bornes ramenées).
#[tauri::command]
pub fn enregistrer_reglages(etat: State<'_, Etat>, reglages: Reglages) -> Result<Reglages, String> {
    let reglages = reglages.assaini();
    reglages
        .enregistrer(&etat.dossier)
        .map_err(|e| format!("réglages non enregistrés : {e}"))?;
    journal::consigner(&etat.dossier, "réglages", "réglages enregistrés");
    Ok(reglages)
}

/// Les dernières opérations du journal, la plus récente d'abord.
#[tauri::command]
pub fn lire_journal(etat: State<'_, Etat>) -> Vec<Operation> {
    let mut operations = journal::lire(&etat.dossier);
    operations.reverse();
    operations
}

/// Quitte l'appli, icône de notification comprise.
#[tauri::command]
pub fn quitter(app: AppHandle) {
    app.exit(0);
}

/// Le diagnostic à coller dans un signalement de bug : rien n'est envoyé, l'interface le copie dans le
/// presse-papiers. Aucun secret, et le chemin du profil devient `%USERPROFILE%` (voir `crate::diagnostic`).
#[tauri::command]
pub fn diagnostic(etat: State<'_, Etat>) -> String {
    let build = std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().map(|d| d.join(installation::INFOS_BUILD)))
        .and_then(|chemin| std::fs::read_to_string(chemin).ok());
    let contexte = diagnostic::Contexte {
        build,
        windows: diagnostic::version_windows(),
        webview2: tauri::webview_version().ok(),
        profil: std::env::var("USERPROFILE").ok(),
    };
    diagnostic::texte(&etat.dossier, &contexte)
}
