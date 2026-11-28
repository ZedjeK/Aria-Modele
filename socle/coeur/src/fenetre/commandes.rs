//! Les commandes générales du socle que l'interface peut appeler. Chacune figure aussi dans `liste.rs` et dans la
//! capacité de sa fenêtre, sinon l'interface ne peut pas l'appeler.
//!
//! Les erreurs remontent en texte français : l'interface les montre telles quelles. Jamais de secret dedans.

use crate::journal::Operation;
use crate::sauvegardes::{self, Sauvegarde};
use crate::{Reglages, diagnostic, identite, installation, journal, reprise};
use serde::Serialize;
use tauri::{AppHandle, Emitter, State};

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
    /// Les modules en option activés (`fichiers`…) : l'interface montre leurs réglages.
    modules: Vec<&'static str>,
}

#[tauri::command]
pub fn identite(etat: State<'_, Etat>) -> Identite {
    Identite {
        nom: identite::nom(),
        version: identite::version(),
        promesse: identite::promesse(),
        en_fond: identite::en_fond(),
        dossier: etat.dossier.racine().display().to_string(),
        modules: modules(),
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
    // Le journal dit déjà quels réglages ont changé (Reglages::enregistrer).
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

/// Les sauvegardes du dossier de données, la plus récente d'abord.
#[tauri::command]
pub fn lister_sauvegardes(etat: State<'_, Etat>) -> Vec<Sauvegarde> {
    sauvegardes::lister(&etat.dossier)
}

/// Restaure une sauvegarde après avoir sauvegardé l'état actuel ; l'interface relit ce qui a pu changer
/// (événement « restauree »).
#[tauri::command(async)]
pub fn restaurer_sauvegarde(app: AppHandle, etat: State<'_, Etat>, nom: String) -> Result<String, String> {
    let fichier = sauvegardes::restaurer(&etat.dossier, &nom)?;
    let _ = app.emit("restauree", fichier.display().to_string());
    Ok(fichier.display().to_string())
}

/// Ce que la restauration d'une sauvegarde changerait, champ par champ, pour un fichier JSON.
#[tauri::command]
pub fn comparer_sauvegarde(etat: State<'_, Etat>, nom: String) -> Result<sauvegardes::Comparaison, String> {
    sauvegardes::comparer(&etat.dossier, &nom)
}

/// Le bilan de la reprise des données d'une ancienne version, si elle a eu lieu.
#[tauri::command]
pub fn bilan_reprise(etat: State<'_, Etat>) -> Option<reprise::Bilan> {
    reprise::bilan(&etat.dossier)
}

/// Les modules en option activés, par leur nom.
fn modules() -> Vec<&'static str> {
    vec![
        #[cfg(feature = "fichiers")]
        "fichiers",
        #[cfg(feature = "notifications")]
        "notifications",
    ]
}
