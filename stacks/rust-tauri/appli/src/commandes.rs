//! Les commandes de l'appli que l'interface peut appeler, en plus de celles du socle. Chacune doit aussi figurer
//! dans build.rs et dans capabilities\principale.json, sinon l'interface ne peut pas l'appeler.
//!
//! Les erreurs remontent en texte français : l'interface les montre telles quelles. Jamais de secret dedans.

use aria_coeur::fenetre::{Etat, taches};
use aria_coeur::journal;
use tauri::{AppHandle, State};

use crate::disque::{self, Espace};

/// Exemple d'action à remplacer : la place sur le disque du dossier de données. Lecture seule. Elle passe par une
/// tâche du socle (`taches::commencer`) : pastille cyan et progression pendant qu'elle tourne, comme toute opération
/// qui peut durer.
// Sur un fil à part : une mesure lente ne fige pas la fenêtre.
#[tauri::command(async)]
pub fn mesurer_disque(app: AppHandle, etat: State<'_, Etat>) -> Result<Espace, String> {
    let tache = taches::commencer(&app, "Mesure du disque");
    tache.avancer(None, "lecture de la place libre");
    let espace = match disque::espace(etat.dossier.racine()) {
        Ok(espace) => espace,
        Err(e) => {
            let raison = format!("mesure impossible : {e}");
            tache.finir(Err(raison.clone()));
            return Err(raison);
        }
    };
    let bilan = format!("disque : {} octets libres sur {}", espace.libre, espace.total);
    journal::consigner(&etat.dossier, "mesure", &bilan);
    tache.finir(Ok(bilan));
    Ok(espace)
}
