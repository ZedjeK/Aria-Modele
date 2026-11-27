//! Les tâches longues : une opération qui peut durer montre sa progression, s'annule, et ne fige jamais la fenêtre
//! (contrat, « Tâches longues »). L'appli lance son travail sur un fil à part et le suit par une [`Tache`] :
//!
//! ```ignore
//! let tache = taches::commencer(&app, "Analyse des fichiers");
//! for (i, fichier) in fichiers.iter().enumerate() {
//!     if tache.annulee() {
//!         return tache.finir(Err("annulée".to_owned()));
//!     }
//!     tache.avancer(Some(i as f64 / fichiers.len() as f64), fichier);
//! }
//! tache.finir(Ok("12 fichiers analysés".to_owned()));
//! ```
//!
//! Chaque changement part vers l'interface par l'événement « tache » ; tant qu'une tâche tourne, l'icône de
//! notification porte la pastille cyan.

use std::collections::BTreeMap;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, PoisonError};

use serde::Serialize;
use tauri::{AppHandle, Emitter, Manager, State};

use super::icone;

/// L'événement envoyé à l'interface à chaque changement d'une tâche.
pub const EVENEMENT: &str = "tache";

/// Où en est une tâche.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum Etape {
    EnCours,
    Finie,
    Annulee,
    Echec,
}

/// Ce que l'interface reçoit d'une tâche.
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Avancement {
    pub id: u64,
    pub libelle: String,
    /// Entre 0 et 1 ; `None` tant que la durée est inconnue (barre indéterminée).
    pub progression: Option<f64>,
    pub detail: String,
    pub etape: Etape,
}

/// Les tâches en cours, partagées entre les commandes.
#[derive(Default)]
pub struct Taches {
    suivante: AtomicU64,
    en_cours: Mutex<BTreeMap<u64, (String, Arc<AtomicBool>)>>,
}

impl Taches {
    fn libelles(&self) -> Vec<String> {
        self.en_cours
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .values()
            .map(|(libelle, _)| libelle.clone())
            .collect()
    }
}

/// Une tâche en cours. La lâcher sans [`Tache::finir`] la note finie.
pub struct Tache {
    app: AppHandle,
    id: u64,
    libelle: String,
    annulee: Arc<AtomicBool>,
    finie: bool,
}

/// L'icône dit ce qui tourne : la pastille cyan tant qu'une tâche est en cours.
fn montrer_dans_l_icone(app: &AppHandle) {
    let libelles = app.state::<Taches>().libelles();
    icone::changer(app, |etat| {
        etat.en_cours = (!libelles.is_empty()).then(|| libelles.join(", "));
    });
}

/// Commence une tâche et la montre.
pub fn commencer(app: &AppHandle, libelle: &str) -> Tache {
    let taches = app.state::<Taches>();
    let id = taches.suivante.fetch_add(1, Ordering::SeqCst) + 1;
    let annulee = Arc::new(AtomicBool::new(false));
    taches
        .en_cours
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .insert(id, (libelle.to_owned(), Arc::clone(&annulee)));
    let tache = Tache {
        app: app.clone(),
        id,
        libelle: libelle.to_owned(),
        annulee,
        finie: false,
    };
    tache.envoyer(None, "", Etape::EnCours);
    montrer_dans_l_icone(app);
    tache
}

impl Tache {
    fn envoyer(&self, progression: Option<f64>, detail: &str, etape: Etape) {
        let _ = self.app.emit(
            EVENEMENT,
            Avancement {
                id: self.id,
                libelle: self.libelle.clone(),
                progression: progression.map(|p| p.clamp(0.0, 1.0)),
                detail: detail.to_owned(),
                etape,
            },
        );
    }

    /// L'identifiant de la tâche, que l'interface passe à `annuler_tache`.
    pub fn id(&self) -> u64 {
        self.id
    }

    /// Montre la progression (entre 0 et 1, ou `None` si elle est inconnue) et ce qui se fait.
    pub fn avancer(&self, progression: Option<f64>, detail: &str) {
        self.envoyer(progression, detail, Etape::EnCours);
    }

    /// L'utilisateur a-t-il demandé l'annulation ? À lire entre deux pas : le travail s'arrête proprement.
    pub fn annulee(&self) -> bool {
        self.annulee.load(Ordering::SeqCst)
    }

    /// Termine la tâche : réussie (avec son bilan), annulée ou en échec (avec la raison).
    pub fn finir(mut self, issue: Result<String, String>) {
        self.terminer(issue);
    }

    fn terminer(&mut self, issue: Result<String, String>) {
        if self.finie {
            return;
        }
        self.finie = true;
        self.app
            .state::<Taches>()
            .en_cours
            .lock()
            .unwrap_or_else(PoisonError::into_inner)
            .remove(&self.id);
        let (etape, detail) = match issue {
            Ok(bilan) => (Etape::Finie, bilan),
            Err(_) if self.annulee() => (Etape::Annulee, "annulée".to_owned()),
            Err(raison) => (Etape::Echec, raison),
        };
        self.envoyer((etape == Etape::Finie).then_some(1.0), &detail, etape);
        montrer_dans_l_icone(&self.app);
    }
}

impl Drop for Tache {
    fn drop(&mut self) {
        self.terminer(Ok(String::new()));
    }
}

/// Demande l'annulation d'une tâche ; le travail s'arrête à son prochain pas.
#[tauri::command]
pub fn annuler_tache(taches: State<'_, Taches>, id: u64) -> bool {
    taches
        .en_cours
        .lock()
        .unwrap_or_else(PoisonError::into_inner)
        .get(&id)
        .map(|(_, annulee)| annulee.store(true, Ordering::SeqCst))
        .is_some()
}
