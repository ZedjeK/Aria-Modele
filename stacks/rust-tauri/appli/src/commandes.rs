//! Les commandes de l'appli que l'interface peut appeler, en plus de celles du socle. Chacune doit aussi figurer
//! dans build.rs et dans capabilities\principale.json, sinon l'interface ne peut pas l'appeler.
//!
//! Les erreurs remontent en texte français : l'interface les montre telles quelles. Jamais de secret dedans.

use aria_coeur::fenetre::{Etat, icone};
use aria_coeur::journal;
use tauri::{AppHandle, State};

use crate::disque::{self, Espace};

/// Exemple d'action à remplacer : la place sur le disque du dossier de données. Lecture seule. Pendant l'action,
/// l'icône de notification porte la pastille cyan (l'appli agit).
#[tauri::command]
pub fn mesurer_disque(app: AppHandle, etat: State<'_, Etat>) -> Result<Espace, String> {
    icone::changer(&app, |icone| icone.en_cours = Some("mesure du disque".to_owned()));
    let mesure = disque::espace(etat.dossier.racine());
    icone::changer(&app, |icone| icone.en_cours = None);
    let espace = mesure.map_err(|e| format!("mesure impossible : {e}"))?;
    journal::consigner(
        &etat.dossier,
        "mesure",
        &format!("disque : {} octets libres sur {}", espace.libre, espace.total),
    );
    Ok(espace)
}
