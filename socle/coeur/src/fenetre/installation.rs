//! La proposition d'installation (fenêtre « installation », vue `Installation.svelte`) : montrée au lancement d'un
//! exe hors de `programme\`, avant la fenêtre de l'appli. « Installer » range l'appli dans le dossier de données et
//! la relance de là ; « Lancer sans installer » ouvre l'appli sur place ; la croix ferme tout.

use std::path::{Path, PathBuf};
use std::process::Command;

use crate::installation::{self, Chemins};
use crate::{Reglages, identite, journal};
use serde::Serialize;
use tauri::window::Color;
use tauri::{AppHandle, Manager, State, Theme, WebviewUrl, WebviewWindowBuilder};

use super::{Etat, erreur_fatale, ouvrir_principale, raccourci};

/// L'étiquette de la fenêtre, citée par capabilities\installation.json.
pub const FENETRE_INSTALLATION: &str = "installation";

/// Ouvre la proposition d'installation.
pub fn ouvrir_fenetre(app: &AppHandle) -> tauri::Result<()> {
    let cache_webview = app.state::<Etat>().cache_webview.clone();
    WebviewWindowBuilder::new(app, FENETRE_INSTALLATION, WebviewUrl::default())
        .title(format!("Installer {}", identite::nom()))
        .inner_size(620.0, 560.0)
        .resizable(false)
        .maximizable(false)
        .center()
        .theme(Some(Theme::Dark))
        .background_color(Color(3, 8, 18, 255))
        .data_directory(cache_webview)
        .build()?;
    Ok(())
}

/// Ce que la fenêtre affiche.
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct InfosInstallation {
    nom: &'static str,
    version: &'static str,
    dossier: String,
    version_installee: Option<String>,
    /// Le raccourci du Bureau ; `None` si le Bureau est introuvable.
    raccourci: Option<String>,
    /// Ce qui est déjà à la place du raccourci : rien, un raccourci vers l'appli, ou vers un autre programme.
    raccourci_existant: raccourci::Existant,
}

/// L'exe installé et l'exe en cours : un raccourci vers l'un des deux est celui de l'appli.
fn exes_de_l_appli(etat: &Etat) -> Vec<PathBuf> {
    let mut exes = vec![etat.dossier.programme().join(installation::nom_exe())];
    exes.extend(std::env::current_exe().ok());
    exes
}

#[tauri::command]
pub fn infos_installation(app: AppHandle, etat: State<'_, Etat>) -> InfosInstallation {
    let programme = etat.dossier.programme();
    let lien = raccourci::chemin(&app);
    let exes = exes_de_l_appli(&etat);
    let exes: Vec<&Path> = exes.iter().map(PathBuf::as_path).collect();
    InfosInstallation {
        nom: identite::nom(),
        version: identite::version(),
        dossier: programme.display().to_string(),
        version_installee: installation::version_installee(&programme),
        raccourci_existant: lien
            .as_deref()
            .map_or(raccourci::Existant::Absent, |l| raccourci::existant(l, &exes)),
        raccourci: lien.map(|c| c.display().to_string()),
    }
}

/// Installe, puis relance l'appli depuis `programme\` et arrête ce lancement. En cas d'échec, rien n'a changé et
/// le message dit pourquoi.
// Sur un fil à part : la copie ne fige pas la fenêtre.
#[tauri::command(async)]
pub fn installer(app: AppHandle, etat: State<'_, Etat>, raccourci: bool) -> Result<(), String> {
    // Contrat : un exe élevé ne se lance pas depuis un dossier qu'un programme sans droits peut modifier. Une
    // appli lancée normalement n'a pas besoin des droits administrateur pour se ranger dans le dossier de données.
    if systeme::est_eleve() {
        return Err(
            "L'appli tourne avec les droits administrateur : l'installation se fait sans eux. Ferme-la et \
                    relance l'exe d'un simple double-clic."
                .to_owned(),
        );
    }
    let exe = std::env::current_exe().map_err(|e| format!("L'exe de l'appli est introuvable : {e}"))?;
    let installe = installation::installer(&exe, &Chemins::du_dossier(&etat.dossier)).map_err(|e| e.0)?;
    let mut bilan = format!(
        "version {} installée dans {}",
        identite::version(),
        etat.dossier.programme().display()
    );
    if raccourci {
        let exes = exes_de_l_appli(&etat);
        let exes: Vec<&Path> = exes.iter().map(PathBuf::as_path).collect();
        let lien = raccourci::chemin(&app);
        let existait = lien
            .as_deref()
            .is_some_and(|l| raccourci::existant(l, &exes) == raccourci::Existant::ANous);
        match lien.map(|lien| raccourci::creer_ou_mettre_a_jour(&lien, &installe, &exes)) {
            Some(Ok(())) if existait => bilan.push_str(", raccourci du Bureau mis à jour"),
            Some(Ok(())) => bilan.push_str(", raccourci du Bureau créé"),
            Some(Err(e)) if e.kind() == std::io::ErrorKind::AlreadyExists => {
                bilan.push_str(", raccourci du Bureau laissé tel quel (il pointe vers un autre programme)");
            }
            Some(Err(e)) => {
                bilan.push_str(", raccourci non créé");
                erreur_fatale::avertir(&format!(
                    "L'appli est installée, mais le raccourci du Bureau n'a pas pu être créé : {e}"
                ));
            }
            None => bilan.push_str(", raccourci non créé (Bureau introuvable)"),
        }
    }
    journal::consigner(&etat.dossier, "installation", &bilan);
    relancer(&app, &installe)
}

/// Lance l'exe installé et arrête celui-ci. La place d'exemplaire unique est rendue d'abord : sinon le nouveau
/// lancement la trouverait prise et s'arrêterait aussitôt.
pub fn relancer(app: &AppHandle, exe: &Path) -> Result<(), String> {
    tauri_plugin_single_instance::destroy(app);
    let mut commande = Command::new(exe);
    if let Some(dossier) = exe.parent() {
        commande.current_dir(dossier);
    }
    commande.spawn().map_err(|e| {
        format!(
            "L'appli est installée mais n'a pas pu être relancée : {e}. Lance-la depuis {}.",
            exe.display()
        )
    })?;
    app.exit(0);
    Ok(())
}

/// Ouvre l'appli sur place. « Ne plus me proposer » est enregistré dans les réglages.
#[tauri::command]
pub fn lancer_sans_installer(
    app: AppHandle,
    etat: State<'_, Etat>,
    ne_plus_proposer: bool,
) -> Result<(), String> {
    if ne_plus_proposer {
        let reglages = Reglages {
            proposer_installation: false,
            ..Reglages::charger(&etat.dossier)
        };
        reglages
            .enregistrer(&etat.dossier)
            .map_err(|e| format!("réglages non enregistrés : {e}"))?;
        journal::consigner(&etat.dossier, "réglages", "proposition d'installation coupée");
    }
    // La fenêtre de l'appli d'abord : fermer la dernière fenêtre arrêterait l'appli.
    ouvrir_principale(&app, true).map_err(|e| e.to_string())?;
    if let Some(fenetre) = app.get_webview_window(FENETRE_INSTALLATION) {
        let _ = fenetre.destroy();
    }
    Ok(())
}

/// Ce que l'installation demande à Windows.
#[cfg(windows)]
mod systeme {
    use windows::Win32::UI::Shell::IsUserAnAdmin;

    /// L'appli tourne-t-elle avec les droits administrateur ?
    #[allow(unsafe_code)]
    pub fn est_eleve() -> bool {
        // SAFETY : fonction sans paramètre, qui ne fait que lire le jeton du processus.
        unsafe { IsUserAnAdmin() }.as_bool()
    }
}

/// Hors de Windows (tests, outils) : pas de droits administrateur.
#[cfg(not(windows))]
mod systeme {
    pub fn est_eleve() -> bool {
        false
    }
}
