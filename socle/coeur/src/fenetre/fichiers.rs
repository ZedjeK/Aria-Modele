//! Module « Fichiers et dossiers », côté fenêtre (fonction `fichiers`) : les fenêtres « Ouvrir », « Enregistrer » et
//! « Choisir un dossier » de Windows, le glisser-déposer, les fichiers ouverts depuis l'Explorateur, et les
//! associations de types de fichiers réglées dans les Paramètres.
//!
//! **Chemins accordés** : un chemin choisi par l'utilisateur (fenêtre de Windows, fichier déposé, fichier ouvert par
//! l'Explorateur) est retenu. Une commande de l'appli qui lit ou écrit un fichier de l'utilisateur vérifie d'abord
//! [`accorde`] : l'interface ne peut pas lui faire toucher un fichier que l'utilisateur n'a pas désigné.
//!
//! ```ignore
//! #[tauri::command]
//! fn enregistrer(app: AppHandle, chemin: PathBuf, texte: String) -> Result<(), String> {
//!     if !fichiers::accorde(&app, &chemin) {
//!         return Err("fichier non choisi par l'utilisateur".into());
//!     }
//!     let dossier = &app.state::<Etat>().dossier;
//!     aria_coeur::fichiers::reecrire(dossier, &chemin, texte.as_bytes()).map_err(|e| e.to_string())?;
//!     Ok(())
//! }
//! ```

use std::collections::HashSet;
use std::path::{Path, PathBuf};
use std::sync::{Mutex, PoisonError};

use serde::Deserialize;
use tauri::{AppHandle, DragDropEvent, Emitter, Manager, State, WebviewWindow, Wry, ipc::Invoke};

use super::{Etat, FENETRE};
use crate::fichiers::associations::{self, Cles};
use crate::fichiers::{EtatAssociation, TypeFichier, chemins_ouverts, extension_valide};
use crate::installation::nom_exe;
use crate::{identite, journal};

/// Les commandes du module, présentes seulement avec la fonction `fichiers` (voir `liste.rs`).
pub fn commandes() -> impl Fn(Invoke<Wry>) -> bool + Send + Sync + 'static {
    tauri::generate_handler![
        choisir_fichiers,
        choisir_dossier,
        choisir_enregistrement,
        fichiers_ouverts,
        lire_associations,
        regler_association,
    ]
}

/// La capacité de la fenêtre principale pour ces commandes, ajoutée au lancement avec les autres.
pub const CAPACITE: &str = include_str!("capacite-fichiers.json");

/// L'état du module : les types déclarés par l'appli, les chemins accordés, les fichiers ouverts pas encore lus.
pub struct Fichiers {
    types: &'static [TypeFichier],
    accordes: Mutex<HashSet<PathBuf>>,
    ouverts: Mutex<Vec<PathBuf>>,
}

impl Fichiers {
    /// Les fichiers passés à l'exe au lancement sont accordés et gardés jusqu'à ce que l'interface les lise.
    pub fn new(types: &'static [TypeFichier], arguments: &[String]) -> Self {
        let ouverts = chemins_ouverts(arguments);
        Self {
            types: types
                .iter()
                .copied()
                .filter(|t| extension_valide(t.extension))
                .collect::<Vec<_>>()
                .leak(),
            accordes: Mutex::new(ouverts.iter().cloned().collect()),
            ouverts: Mutex::new(ouverts),
        }
    }

    fn accorder(&self, chemins: &[PathBuf]) {
        let mut accordes = self.accordes.lock().unwrap_or_else(PoisonError::into_inner);
        accordes.extend(chemins.iter().cloned());
    }
}

/// L'utilisateur a-t-il désigné ce chemin (fenêtre de Windows, dépôt, ouverture par l'Explorateur) ? Un fichier
/// d'un dossier choisi compte aussi.
pub fn accorde(app: &AppHandle, chemin: &Path) -> bool {
    let Some(fichiers) = app.try_state::<Fichiers>() else {
        return false;
    };
    let accordes = fichiers.accordes.lock().unwrap_or_else(PoisonError::into_inner);
    // Pas de `..` : un chemin accordé ne s'élargit pas à son voisin.
    let propre = !chemin
        .components()
        .any(|c| matches!(c, std::path::Component::ParentDir));
    propre && chemin.ancestors().any(|a| accordes.contains(a))
}

/// Un second lancement avec des fichiers (double-clic dans l'Explorateur, appli déjà ouverte) : accordés et
/// annoncés à l'interface.
pub fn a_la_relance(app: &AppHandle, arguments: &[String]) {
    let ouverts = chemins_ouverts(arguments.iter().skip(1));
    if ouverts.is_empty() {
        return;
    }
    if let Some(fichiers) = app.try_state::<Fichiers>() {
        fichiers.accorder(&ouverts);
    }
    let _ = app.emit_to(FENETRE, "fichiers-ouverts", chemins_en_texte(&ouverts));
}

/// Branche le glisser-déposer sur la fenêtre principale : les fichiers déposés sont accordés et annoncés à
/// l'interface (`fichiers-deposes`).
pub fn suivre_depots(fenetre: &WebviewWindow) {
    let app = fenetre.app_handle().clone();
    fenetre.on_window_event(move |evenement| {
        if let tauri::WindowEvent::DragDrop(DragDropEvent::Drop { paths, .. }) = evenement {
            if let Some(fichiers) = app.try_state::<Fichiers>() {
                fichiers.accorder(paths);
            }
            let _ = app.emit_to(FENETRE, "fichiers-deposes", chemins_en_texte(paths));
        }
    });
}

/// Retire toutes les associations de l'appli (désinstallation).
///
/// # Errors
/// Le registre qui ne s'écrit pas.
pub fn retirer_associations() -> Result<String, String> {
    match associations::dissocier_tout(&Cles::utilisateur(), identite::identite().nom) {
        Ok(retirees) if retirees.is_empty() => Ok("aucune".to_owned()),
        Ok(retirees) => Ok(format!("retirées : .{}", retirees.join(", ."))),
        Err(e) => Err(format!("pas retirées : {e}")),
    }
}

fn chemins_en_texte(chemins: &[PathBuf]) -> Vec<String> {
    chemins.iter().map(|c| c.display().to_string()).collect()
}

/// Un filtre de la fenêtre de Windows : un nom et ses extensions (sans point).
#[derive(Debug, Clone, Deserialize)]
pub struct Filtre {
    nom: String,
    extensions: Vec<String>,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionsOuvrir {
    titre: Option<String>,
    filtres: Vec<Filtre>,
    plusieurs: bool,
}

#[derive(Debug, Clone, Default, Deserialize)]
#[serde(rename_all = "camelCase", default)]
pub struct OptionsEnregistrer {
    titre: Option<String>,
    nom_propose: Option<String>,
    filtres: Vec<Filtre>,
}

/// Les filtres au format de Windows (`Documents` → `*.md;*.txt`), extensions vérifiées.
fn specs(filtres: &[Filtre]) -> Result<Vec<(String, String)>, String> {
    filtres
        .iter()
        .map(|f| {
            if f.extensions.is_empty() {
                return Err(format!("filtre « {} » sans extension", f.nom));
            }
            if let Some(mauvaise) = f.extensions.iter().find(|e| !extension_valide(e)) {
                return Err(format!("extension refusée : « {mauvaise} »"));
            }
            let motif = f
                .extensions
                .iter()
                .map(|e| format!("*.{e}"))
                .collect::<Vec<_>>()
                .join(";");
            Ok((f.nom.clone(), motif))
        })
        .collect()
}

/// La fenêtre principale, propriétaire des fenêtres de Windows (elles restent devant elle).
fn proprietaire(app: &AppHandle) -> Result<isize, String> {
    let fenetre = app
        .get_webview_window(FENETRE)
        .ok_or_else(|| "la fenêtre de l'appli est introuvable".to_owned())?;
    let poignee = fenetre.hwnd().map_err(|e| e.to_string())?;
    Ok(poignee.0 as isize)
}

/// Lance une fenêtre de Windows sur un fil à part (COM y est préparé pour elle seule) et accorde ce qui est choisi.
fn choisir(app: &AppHandle, demande: dialogues::Demande) -> Result<Vec<String>, String> {
    let fil = std::thread::spawn(move || dialogues::montrer(&demande));
    let choisis = fil
        .join()
        .map_err(|_| "la fenêtre de Windows s'est arrêtée".to_owned())??;
    if let Some(fichiers) = app.try_state::<Fichiers>() {
        fichiers.accorder(&choisis);
    }
    Ok(chemins_en_texte(&choisis))
}

/// La fenêtre « Ouvrir » de Windows ; une liste vide si l'utilisateur annule.
#[tauri::command(async)]
pub fn choisir_fichiers(app: AppHandle, options: OptionsOuvrir) -> Result<Vec<String>, String> {
    let demande = dialogues::Demande {
        proprietaire: proprietaire(&app)?,
        genre: dialogues::Genre::Ouvrir {
            plusieurs: options.plusieurs,
        },
        titre: options.titre.unwrap_or_else(|| "Ouvrir".to_owned()),
        filtres: specs(&options.filtres)?,
        nom_propose: None,
    };
    choisir(&app, demande)
}

/// La fenêtre « Choisir un dossier » de Windows ; `None` si l'utilisateur annule.
#[tauri::command(async)]
pub fn choisir_dossier(app: AppHandle, titre: Option<String>) -> Result<Option<String>, String> {
    let demande = dialogues::Demande {
        proprietaire: proprietaire(&app)?,
        genre: dialogues::Genre::Dossier,
        titre: titre.unwrap_or_else(|| "Choisir un dossier".to_owned()),
        filtres: Vec::new(),
        nom_propose: None,
    };
    Ok(choisir(&app, demande)?.into_iter().next())
}

/// La fenêtre « Enregistrer sous » de Windows ; `None` si l'utilisateur annule. Rien n'est écrit : l'appli écrit
/// ensuite par [`crate::fichiers::reecrire`], qui copie d'abord un fichier existant.
#[tauri::command(async)]
pub fn choisir_enregistrement(app: AppHandle, options: OptionsEnregistrer) -> Result<Option<String>, String> {
    let demande = dialogues::Demande {
        proprietaire: proprietaire(&app)?,
        genre: dialogues::Genre::Enregistrer,
        titre: options.titre.unwrap_or_else(|| "Enregistrer sous".to_owned()),
        filtres: specs(&options.filtres)?,
        nom_propose: options.nom_propose,
    };
    Ok(choisir(&app, demande)?.into_iter().next())
}

/// Les fichiers passés à l'exe au lancement, une seule fois : l'interface les lit quand elle est prête.
#[tauri::command]
pub fn fichiers_ouverts(fichiers: State<'_, Fichiers>) -> Vec<String> {
    let mut ouverts = fichiers.ouverts.lock().unwrap_or_else(PoisonError::into_inner);
    chemins_en_texte(&std::mem::take(&mut *ouverts))
}

/// Les types de fichiers que l'appli déclare, et leur association.
#[tauri::command]
pub fn lire_associations(fichiers: State<'_, Fichiers>) -> Vec<EtatAssociation> {
    associations::etat(&Cles::utilisateur(), identite::identite().nom, fichiers.types)
}

/// Associe ou dissocie un type déclaré, à la demande de l'utilisateur. L'association vise l'exe installé : une
/// appli qui n'est pas installée ne s'associe pas.
#[tauri::command]
pub fn regler_association(
    etat: State<'_, Etat>,
    fichiers: State<'_, Fichiers>,
    extension: String,
    active: bool,
) -> Result<Vec<EtatAssociation>, String> {
    let type_fichier = fichiers
        .types
        .iter()
        .find(|t| t.extension == extension)
        .ok_or_else(|| format!("« .{extension} » n'est pas un type que l'appli déclare"))?;
    let cles = Cles::utilisateur();
    let nom = identite::identite().nom;
    if active {
        let exe = etat.dossier.programme().join(nom_exe());
        if !exe.is_file() {
            return Err(
                "l'appli n'est pas installée : elle ne peut pas encore s'associer à des fichiers".to_owned(),
            );
        }
        associations::associer(&cles, nom, type_fichier, &exe)
            .map_err(|e| format!("association impossible : {e}"))?;
        journal::consigner(
            &etat.dossier,
            "fichiers",
            &format!(".{extension} associé à l'appli"),
        );
    } else {
        associations::dissocier(&cles, nom, &extension)
            .map_err(|e| format!("dissociation impossible : {e}"))?;
        journal::consigner(
            &etat.dossier,
            "fichiers",
            &format!(".{extension} dissocié de l'appli"),
        );
    }
    Ok(associations::etat(&cles, nom, fichiers.types))
}

/// Les fenêtres de Windows (`IFileOpenDialog`, `IFileSaveDialog`).
mod dialogues {
    use std::path::PathBuf;

    pub enum Genre {
        Ouvrir { plusieurs: bool },
        Dossier,
        Enregistrer,
    }

    pub struct Demande {
        /// La poignée de la fenêtre principale (`HWND`), passée d'un fil à l'autre sous forme de nombre.
        pub proprietaire: isize,
        pub genre: Genre,
        pub titre: String,
        /// Nom affiché et motif (`*.md;*.txt`).
        pub filtres: Vec<(String, String)>,
        pub nom_propose: Option<String>,
    }

    #[cfg(windows)]
    pub fn montrer(demande: &Demande) -> Result<Vec<PathBuf>, String> {
        windows_seul::montrer(demande).map_err(|e| format!("fenêtre de Windows : {e}"))
    }

    #[cfg(not(windows))]
    pub fn montrer(_demande: &Demande) -> Result<Vec<PathBuf>, String> {
        Err("fenêtres de fichiers : Windows seulement".to_owned())
    }

    #[cfg(windows)]
    mod windows_seul {
        use std::path::PathBuf;

        use windows::Win32::Foundation::{ERROR_CANCELLED, HWND};
        use windows::Win32::System::Com::{
            CLSCTX_INPROC_SERVER, COINIT_APARTMENTTHREADED, CoCreateInstance, CoInitializeEx, CoTaskMemFree,
            CoUninitialize,
        };
        use windows::Win32::UI::Shell::Common::COMDLG_FILTERSPEC;
        use windows::Win32::UI::Shell::{
            FOS_ALLOWMULTISELECT, FOS_FILEMUSTEXIST, FOS_FORCEFILESYSTEM, FOS_OVERWRITEPROMPT,
            FOS_PATHMUSTEXIST, FOS_PICKFOLDERS, FileOpenDialog, FileSaveDialog, IFileOpenDialog,
            IFileSaveDialog, IShellItem, SIGDN_FILESYSPATH,
        };
        use windows::core::{HSTRING, PCWSTR};

        use super::{Demande, Genre};

        /// `Show` rend ERROR_CANCELLED quand l'utilisateur ferme la fenêtre sans choisir.
        fn annulee(resultat: windows::core::Result<()>) -> windows::core::Result<bool> {
            match resultat {
                Ok(()) => Ok(false),
                Err(e) if e.code() == ERROR_CANCELLED.to_hresult() => Ok(true),
                Err(e) => Err(e),
            }
        }

        #[allow(unsafe_code)]
        fn chemin_de(element: &IShellItem) -> windows::core::Result<PathBuf> {
            // SAFETY : GetDisplayName rend une chaîne allouée par COM, lue puis libérée une seule fois ici.
            unsafe {
                let texte = element.GetDisplayName(SIGDN_FILESYSPATH)?;
                let chemin = texte.to_string();
                CoTaskMemFree(Some(texte.0 as *const core::ffi::c_void));
                Ok(PathBuf::from(chemin.map_err(|_| {
                    windows::core::Error::from(ERROR_CANCELLED.to_hresult())
                })?))
            }
        }

        #[allow(unsafe_code)]
        pub fn montrer(demande: &Demande) -> windows::core::Result<Vec<PathBuf>> {
            // Les chaînes vivent jusqu'à la fin de la fenêtre : les filtres pointent dessus.
            let textes: Vec<(HSTRING, HSTRING)> = demande
                .filtres
                .iter()
                .map(|(nom, motif)| (HSTRING::from(nom.as_str()), HSTRING::from(motif.as_str())))
                .collect();
            let filtres: Vec<COMDLG_FILTERSPEC> = textes
                .iter()
                .map(|(nom, motif)| COMDLG_FILTERSPEC {
                    pszName: PCWSTR(nom.as_ptr()),
                    pszSpec: PCWSTR(motif.as_ptr()),
                })
                .collect();
            let titre = HSTRING::from(demande.titre.as_str());
            let proprietaire = HWND(demande.proprietaire as *mut core::ffi::c_void);
            // SAFETY : COM est préparé puis libéré sur ce fil, lancé pour cette seule fenêtre ; les objets COM sont
            // relâchés avant CoUninitialize, et les chaînes passées restent vivantes pendant tous les appels.
            unsafe {
                CoInitializeEx(None, COINIT_APARTMENTTHREADED).ok()?;
                let resultat = (|| -> windows::core::Result<Vec<PathBuf>> {
                    if let Genre::Enregistrer = demande.genre {
                        let fenetre: IFileSaveDialog =
                            CoCreateInstance(&FileSaveDialog, None, CLSCTX_INPROC_SERVER)?;
                        fenetre.SetOptions(
                            fenetre.GetOptions()?
                                | FOS_FORCEFILESYSTEM
                                | FOS_PATHMUSTEXIST
                                | FOS_OVERWRITEPROMPT,
                        )?;
                        fenetre.SetTitle(&titre)?;
                        if !filtres.is_empty() {
                            fenetre.SetFileTypes(&filtres)?;
                            // L'extension du premier filtre, ajoutée si l'utilisateur n'en tape pas.
                            if let Some(premiere) = demande.filtres[0].1.split(';').next() {
                                fenetre
                                    .SetDefaultExtension(&HSTRING::from(premiere.trim_start_matches("*.")))?;
                            }
                        }
                        if let Some(nom) = &demande.nom_propose {
                            fenetre.SetFileName(&HSTRING::from(nom.as_str()))?;
                        }
                        if annulee(fenetre.Show(Some(proprietaire)))? {
                            return Ok(Vec::new());
                        }
                        return Ok(vec![chemin_de(&fenetre.GetResult()?)?]);
                    }
                    let fenetre: IFileOpenDialog =
                        CoCreateInstance(&FileOpenDialog, None, CLSCTX_INPROC_SERVER)?;
                    let mut options = fenetre.GetOptions()? | FOS_FORCEFILESYSTEM | FOS_PATHMUSTEXIST;
                    match demande.genre {
                        Genre::Dossier => options |= FOS_PICKFOLDERS,
                        Genre::Ouvrir { plusieurs } => {
                            options |= FOS_FILEMUSTEXIST;
                            if plusieurs {
                                options |= FOS_ALLOWMULTISELECT;
                            }
                        }
                        Genre::Enregistrer => {}
                    }
                    fenetre.SetOptions(options)?;
                    fenetre.SetTitle(&titre)?;
                    if !filtres.is_empty() {
                        fenetre.SetFileTypes(&filtres)?;
                    }
                    if annulee(fenetre.Show(Some(proprietaire)))? {
                        return Ok(Vec::new());
                    }
                    let elements = fenetre.GetResults()?;
                    (0..elements.GetCount()?)
                        .map(|i| chemin_de(&elements.GetItemAt(i)?))
                        .collect()
                })();
                CoUninitialize();
                resultat
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn filtre(nom: &str, extensions: &[&str]) -> Filtre {
        Filtre {
            nom: nom.to_owned(),
            extensions: extensions.iter().map(|e| (*e).to_owned()).collect(),
        }
    }

    #[test]
    fn les_filtres_se_traduisent() {
        let vus = specs(&[filtre("Documents", &["md", "txt"])]).unwrap();
        assert_eq!(vus, [("Documents".to_owned(), "*.md;*.txt".to_owned())]);
        assert!(specs(&[filtre("Tout", &["*"])]).is_err());
        assert!(specs(&[filtre("Vide", &[])]).is_err());
    }

    #[test]
    fn les_types_mal_declares_sont_ecartes() {
        static TYPES: [TypeFichier; 2] = [
            TypeFichier {
                extension: "md",
                description: "Markdown",
            },
            TypeFichier {
                extension: ".exe",
                description: "Mauvais",
            },
        ];
        let fichiers = Fichiers::new(&TYPES, &[]);
        assert_eq!(fichiers.types.len(), 1);
    }

    #[test]
    fn le_gestionnaire_suit_la_liste() {
        let source = include_str!("fichiers.rs");
        let debut = source.find("generate_handler![").unwrap() + "generate_handler![".len();
        let fin = debut + source[debut..].find(']').unwrap();
        let mut gestionnaire: Vec<&str> = source[debut..fin]
            .split(',')
            .map(str::trim)
            .filter(|c| !c.is_empty())
            .collect();
        let mut liste = super::super::liste::COMMANDES_FICHIERS.to_vec();
        gestionnaire.sort_unstable();
        liste.sort_unstable();
        assert_eq!(gestionnaire, liste);
    }
}
