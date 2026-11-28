//! La fenêtre du socle : le lancement de l'appli (dossier de données, WebView2, proposition d'installation), la
//! fenêtre principale, l'icône de notification, les mises à jour et la désinstallation, et les commandes que
//! l'interface du socle appelle.
//!
//! L'appli n'écrit que son métier : elle donne son identité, son contexte Tauri et ses propres commandes à
//! [`lancer`], qui fait le reste.
//!
//! ```ignore
//! fn main() {
//!     aria_coeur::fenetre::lancer(
//!         IDENTITE,
//!         Appli::new(tauri::generate_context!(), tauri::generate_handler![commandes::mesurer_disque]),
//!     );
//! }
//! ```

mod commandes;
pub mod demarrage_auto;
pub mod desinstallation;
pub mod erreur_fatale;
#[cfg(feature = "fichiers")]
pub mod fichiers;
pub mod icone;
pub mod installation;
mod liste;
pub mod mise_a_jour;
#[cfg(feature = "notifications")]
pub mod notifications;
pub mod raccourci;
pub mod taches;

use std::path::{Path, PathBuf};
use std::sync::atomic::AtomicBool;

use tauri::ipc::Invoke;
use tauri::window::Color;
use tauri::{
    AppHandle, Builder, Context, Manager, RunEvent, Theme, WebviewUrl, WebviewWindowBuilder, WindowEvent, Wry,
};

use crate::identite::{self, Identite};
use crate::installation::Lancement;
use crate::{Dossier, demarrage, journal, reprise};

pub use liste::COMMANDES;

/// L'étiquette de la fenêtre principale, citée par les capacités de l'appli et du socle.
pub const FENETRE: &str = "principale";

/// Ce que les commandes partagent. L'appli le lit dans ses propres commandes par `State<'_, Etat>`.
pub struct Etat {
    pub dossier: Dossier,
    /// Contrat : WebView2 range ses données dans le dossier de l'appli, jamais dans %LOCALAPPDATA%.
    pub cache_webview: PathBuf,
    /// Désinstallée, `programme\` reste à supprimer à la fermeture (voir `desinstallation::a_la_sortie`).
    pub desinstallee: AtomicBool,
    /// L'utilisateur a demandé d'effacer aussi ses données : fait à la fermeture, par la copie de fin.
    pub effacer_donnees: AtomicBool,
}

/// Le gestionnaire des commandes de l'appli, tel que `tauri::generate_handler!` le rend.
type Commandes = Box<dyn Fn(Invoke<Wry>) -> bool + Send + Sync + 'static>;

/// Ce que l'appli ajoute au constructeur Tauri (son état, un greffon).
type Preparation = Box<dyn FnOnce(Builder<Wry>) -> Builder<Wry>>;

/// La reprise des données d'une ancienne version : l'ancien dossier, et ce qui en reprend les données.
type Reprise = (PathBuf, Box<dyn FnOnce(&Path, &Dossier, &mut reprise::Bilan)>);

/// Ce que l'appli apporte au socle : son contexte Tauri, ses commandes et, au besoin, ses ajouts au constructeur.
pub struct Appli {
    contexte: Context<Wry>,
    commandes: Commandes,
    preparation: Option<Preparation>,
    reprise: Option<Reprise>,
    #[cfg(feature = "fichiers")]
    types_fichiers: &'static [crate::fichiers::TypeFichier],
}

impl Appli {
    /// `contexte` : `tauri::generate_context!()` ; `commandes` : `tauri::generate_handler![...]` des seules
    /// commandes de l'appli. Elles s'autorisent dans son build.rs et ses capacités, comme avant.
    pub fn new(
        contexte: Context<Wry>,
        commandes: impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static,
    ) -> Self {
        Self {
            contexte,
            commandes: Box::new(commandes),
            preparation: None,
            reprise: None,
            #[cfg(feature = "fichiers")]
            types_fichiers: &[],
        }
    }

    /// Les types de fichiers que l'appli sait ouvrir (fonction `fichiers`) : proposés dans les Paramètres, associés
    /// seulement si l'utilisateur coche la case, retirés à la désinstallation.
    #[cfg(feature = "fichiers")]
    #[must_use]
    pub fn types_de_fichiers(mut self, types: &'static [crate::fichiers::TypeFichier]) -> Self {
        self.types_fichiers = types;
        self
    }

    /// Ajoute au constructeur Tauri ce que l'appli gère elle-même : son état (`.manage`), un greffon.
    #[must_use]
    pub fn preparer(mut self, preparation: impl FnOnce(Builder<Wry>) -> Builder<Wry> + 'static) -> Self {
        self.preparation = Some(Box::new(preparation));
        self
    }

    /// Reprend une fois les données d'une ancienne version de l'appli, rangées dans `ancien` (voir
    /// [`reprise::une_fois`]) : au premier lancement de l'appli installée, avant sa fenêtre.
    #[must_use]
    pub fn reprendre(
        mut self,
        ancien: impl Into<PathBuf>,
        reprendre: impl FnOnce(&Path, &Dossier, &mut reprise::Bilan) + 'static,
    ) -> Self {
        self.reprise = Some((ancien.into(), Box::new(reprendre)));
        self
    }
}

/// Lance l'appli et ne rend la main qu'à sa fermeture. Une erreur de démarrage est montrée dans une boîte de
/// dialogue de Windows, et l'exe s'arrête avec le code 1.
pub fn lancer(identite: Identite, appli: Appli) {
    let resultat = identite::definir(identite).and_then(|()| demarrer(appli));
    if let Err(message) = resultat {
        erreur_fatale::afficher(&message);
        std::process::exit(1);
    }
}

fn demarrer(appli: Appli) -> Result<(), String> {
    let dossier = Dossier::du_profil().map_err(|e| e.to_string())?;
    let arguments: Vec<String> = std::env::args().skip(1).collect();
    // Copie de l'exe qui finit une désinstallation : ni fenêtre ni Tauri ; elle vide programme\ et s'arrête.
    if let Some(processus) = crate::desinstallation::processus_a_attendre(&arguments) {
        let effacer = crate::desinstallation::effacer_demande(&arguments);
        desinstallation::finir(&dossier, processus, effacer);
        return Ok(());
    }
    dossier.preparer().map_err(|e| e.to_string())?;
    journal::consigner(
        &dossier,
        "lancement",
        &format!(
            "{} {} (socle {})",
            identite::nom(),
            identite::version(),
            identite::VERSION_SOCLE
        ),
    );
    // Restes d'un remplacement précédent, maintenant que l'ancienne version est arrêtée.
    crate::installation::nettoyer(&dossier.programme());
    // La copie qui a fini une désinstallation précédente : elle ne pouvait pas se supprimer elle-même.
    crate::desinstallation::nettoyer_aide(&dossier.cache());
    // Sans WebView2, la fenêtre ne peut pas s'afficher : l'appli le dit et propose la page de Microsoft.
    if !webview2_present() {
        journal::consigner(&dossier, "lancement", WEBVIEW2_ABSENT);
        erreur_fatale::webview2_absent();
        std::process::exit(1);
    }
    // Lancé à l'ouverture de session : l'icône seule, la fenêtre reste rangée jusqu'à ce qu'on l'ouvre.
    let silencieux = identite::en_fond() && demarrage::est_silencieux(&arguments);
    // D'où vient l'exe (décision 0008) : installée, zip décompressé à installer, aperçu du zip, ou compilé à la main.
    let exe = std::env::current_exe().map_err(|e| format!("l'exe de l'appli est introuvable : {e}"))?;
    let temporaire = std::env::temp_dir();
    let proposer = match crate::installation::lancement(&exe, &dossier, Some(&temporaire)) {
        Lancement::ADecompresser => {
            journal::consigner(&dossier, "lancement", DEPUIS_L_APERCU);
            erreur_fatale::decompresser_d_abord();
            return Ok(());
        }
        Lancement::AInstaller => !silencieux,
        Lancement::Installee | Lancement::Developpement => false,
    };
    let cache_webview = dossier.cache_webview();
    let Appli {
        contexte,
        commandes,
        preparation,
        reprise: a_reprendre,
        #[cfg(feature = "fichiers")]
        types_fichiers,
    } = appli;
    // La reprise se fait dans l'appli installée, pas pendant la proposition d'installation.
    if let Some((ancien, reprendre)) = a_reprendre.filter(|_| !proposer)
        && let Err(e) = reprise::une_fois(&dossier, &ancien, reprendre)
    {
        journal::consigner(
            &dossier,
            "reprise",
            &format!("reportée au prochain lancement : {e}"),
        );
    }

    let mut constructeur = Builder::default()
        // Toujours le premier greffon : relancer l'exe ramène la fenêtre existante au lieu d'en ouvrir une autre.
        // Un second lancement silencieux (ouverture de session) ne fait pas surgir la fenêtre.
        .plugin(tauri_plugin_single_instance::init(|app, arguments, _dossier| {
            if !demarrage::est_silencieux(arguments.iter().skip(1)) {
                ramener(app);
            }
            #[cfg(feature = "fichiers")]
            fichiers::a_la_relance(app, &arguments);
        }))
        .manage(Etat {
            dossier,
            cache_webview,
            desinstallee: AtomicBool::new(false),
            effacer_donnees: AtomicBool::new(false),
        })
        .manage(icone::Presence::default())
        .manage(taches::Taches::default())
        .manage(mise_a_jour::Disponible::default());
    #[cfg(feature = "fichiers")]
    {
        constructeur = constructeur.manage(fichiers::Fichiers::new(types_fichiers, &arguments));
    }
    if let Some(preparation) = preparation {
        constructeur = preparation(constructeur);
    }
    let du_socle = commandes_du_socle();
    let modules = commandes_des_modules();
    constructeur
        // Un appel va au socle si sa commande est dans COMMANDES, à un module activé si elle est dans sa liste,
        // sinon à l'appli.
        .invoke_handler(move |appel: Invoke<Wry>| {
            let commande = appel.message.command();
            if COMMANDES.contains(&commande) {
                return du_socle(appel);
            }
            match modules.iter().find(|(liste, _)| liste.contains(&commande)) {
                Some((_, module)) => module(appel),
                None => commandes(appel),
            }
        })
        .setup(move |app| {
            autoriser(app.handle())?;
            if proposer {
                installation::ouvrir_fenetre(app.handle())?;
            } else {
                ouvrir_principale(app.handle(), !silencieux)?;
            }
            Ok(())
        })
        .build(contexte)
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

/// Les commandes du socle, aiguillées par leur nom. La liste suit `COMMANDES` (voir le test).
fn commandes_du_socle() -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        commandes::identite,
        commandes::lire_reglages,
        commandes::enregistrer_reglages,
        commandes::lire_journal,
        commandes::quitter,
        commandes::diagnostic,
        demarrage_auto::lire_demarrage_auto,
        demarrage_auto::regler_demarrage_auto,
        icone::etat_menu,
        icone::action_menu,
        icone::taille_menu,
        installation::infos_installation,
        installation::installer,
        installation::refuser_installation,
        mise_a_jour::mise_a_jour_disponible,
        mise_a_jour::voir_notes,
        mise_a_jour::installer_mise_a_jour,
        desinstallation::desinstaller,
        desinstallation::fermer_apres_desinstallation,
        commandes::lister_sauvegardes,
        commandes::restaurer_sauvegarde,
        commandes::comparer_sauvegarde,
        commandes::bilan_reprise,
        taches::annuler_tache,
    ]
}

/// Les commandes des modules activés, avec leur liste.
type Module = (&'static [&'static str], Commandes);

fn commandes_des_modules() -> Vec<Module> {
    vec![
        #[cfg(feature = "fichiers")]
        (liste::COMMANDES_FICHIERS, Box::new(fichiers::commandes())),
        #[cfg(feature = "notifications")]
        (
            liste::COMMANDES_NOTIFICATIONS,
            Box::new(notifications::commandes()),
        ),
    ]
}

/// La capacité du module « Fichiers et dossiers », ajoutée seulement avec sa fonction.
#[cfg_attr(not(feature = "fichiers"), allow(dead_code))]
const CAPACITE_FICHIERS: &str = include_str!("capacite-fichiers.json");

/// La capacité du module « Notifications », ajoutée seulement avec sa fonction.
#[cfg_attr(not(feature = "notifications"), allow(dead_code))]
const CAPACITE_NOTIFICATIONS: &str = include_str!("capacite-notifications.json");

/// Chaque module, sa liste de commandes et sa capacité (pour les tests).
#[cfg(test)]
const MODULES_ET_CAPACITES: [(&[&str], &str); 2] = [
    (liste::COMMANDES_FICHIERS, CAPACITE_FICHIERS),
    (liste::COMMANDES_NOTIFICATIONS, CAPACITE_NOTIFICATIONS),
];

/// Les capacités des fenêtres du socle : la fenêtre principale (commandes du socle, barre de titre), la proposition
/// d'installation et le menu de l'icône. Ajoutées au lancement, à côté de celles de l'appli (`capabilities\`), qui
/// n'autorisent que ses propres commandes.
const CAPACITES: [&str; 3] = [
    include_str!("capacite-principale.json"),
    include_str!("capacite-installation.json"),
    include_str!("capacite-menu.json"),
];

fn autoriser(app: &AppHandle) -> tauri::Result<()> {
    for capacite in CAPACITES {
        app.add_capability(capacite)?;
    }
    #[cfg(feature = "fichiers")]
    app.add_capability(CAPACITE_FICHIERS)?;
    #[cfg(feature = "notifications")]
    app.add_capability(CAPACITE_NOTIFICATIONS)?;
    Ok(())
}

/// La ligne du journal quand l'exe est lancé depuis l'aperçu du zip.
const DEPUIS_L_APERCU: &str = "lancé depuis l'aperçu du zip : décompresser d'abord";

/// La ligne du journal quand WebView2 manque.
const WEBVIEW2_ABSENT: &str = "WebView2 absent : l'appli ne peut pas s'ouvrir";

/// WebView2 est-il installé ? Par la méthode que documente Microsoft, `GetAvailableCoreWebView2BrowserVersionString`
/// (appelée par `tauri::webview_version`) : sans moteur, pas de version.
fn webview2_present() -> bool {
    tauri::webview_version().is_ok_and(|version| !version.trim().is_empty())
}

/// Ouvre la fenêtre de l'appli et, pour une appli qui sert en arrière-plan, son icône de notification.
///
/// # Errors
/// La fenêtre ou l'icône n'ont pas pu être créées.
pub fn ouvrir_principale(app: &AppHandle, visible: bool) -> tauri::Result<()> {
    let cache_webview = app.state::<Etat>().cache_webview.clone();
    let fenetre = WebviewWindowBuilder::new(app, FENETRE, WebviewUrl::default())
        .title(identite::nom())
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
    #[cfg(feature = "fichiers")]
    fichiers::suivre_depots(&fenetre);
    if identite::en_fond() {
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
pub fn ramener(app: &AppHandle) {
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

#[cfg(test)]
mod tests {
    use super::*;

    /// Les commandes de `generate_handler!`, lues dans ce fichier.
    fn commandes_du_gestionnaire() -> Vec<String> {
        let source = include_str!("mod.rs");
        // La liste du gestionnaire, pas l'exemple de la documentation : celle de `commandes_du_socle`.
        let fonction = source.find("fn commandes_du_socle").unwrap();
        let marque = "generate_handler![";
        let debut = fonction + source[fonction..].find(marque).unwrap() + marque.len();
        let fin = debut + source[debut..].find(']').unwrap();
        source[debut..fin]
            .split(',')
            .filter_map(|c| c.trim().rsplit("::").next())
            .filter(|c| !c.is_empty())
            .map(str::to_owned)
            .collect()
    }

    #[test]
    fn le_gestionnaire_suit_la_liste() {
        let mut gestionnaire = commandes_du_gestionnaire();
        let mut liste: Vec<String> = COMMANDES.iter().map(|c| (*c).to_owned()).collect();
        gestionnaire.sort();
        liste.sort();
        assert_eq!(gestionnaire, liste);
    }

    /// Les commandes citées par une capacité, hors `core:`.
    fn citees(capacite: &str) -> Vec<String> {
        let json: serde_json::Value = serde_json::from_str(capacite).unwrap();
        json["permissions"]
            .as_array()
            .unwrap()
            .iter()
            .map(|p| p.as_str().unwrap())
            .filter(|p| !p.starts_with("core:"))
            .map(|p| p.strip_prefix("allow-").unwrap().replace('-', "_"))
            .collect()
    }

    #[test]
    fn les_capacites_ne_citent_que_des_commandes_du_socle() {
        for capacite in CAPACITES {
            for commande in citees(capacite) {
                assert!(COMMANDES.contains(&commande.as_str()), "{commande}");
            }
        }
    }

    #[test]
    fn chaque_module_a_sa_capacite_complete() {
        assert_eq!(MODULES_ET_CAPACITES.len(), liste::MODULES.len());
        for (commandes, capacite) in MODULES_ET_CAPACITES {
            let mut vues = citees(capacite);
            let mut attendues: Vec<String> = commandes.iter().map(|c| (*c).to_owned()).collect();
            vues.sort();
            attendues.sort();
            assert_eq!(vues, attendues);
        }
        // Aucune commande d'un module dans la liste toujours publiée, ni dans deux modules.
        let mut vues = std::collections::HashSet::new();
        for (_, commandes) in liste::MODULES {
            for commande in *commandes {
                assert!(!COMMANDES.contains(commande), "{commande}");
                assert!(vues.insert(*commande), "{commande}");
            }
        }
    }

    #[test]
    fn les_capacites_se_lisent() {
        for capacite in CAPACITES
            .iter()
            .chain(MODULES_ET_CAPACITES.iter().map(|(_, c)| c))
        {
            assert!(
                capacite
                    .parse::<tauri::utils::acl::capability::CapabilityFile>()
                    .is_ok()
            );
        }
    }
}
