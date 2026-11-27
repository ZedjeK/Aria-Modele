//! Les réglages de l'appli, enregistrés dans `reglages.json`. Jamais de secret ici : ils vont dans `secrets\`.
//!
//! Le fichier porte le numéro de son format, se lit champ par champ (un champ illisible prend sa valeur par défaut
//! sans emporter les autres) et n'est jamais réécrit sans copie s'il ne se relisait pas tel quel. L'appli range ses
//! propres réglages sous la clé `appli`.

use std::fs;

use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;

use crate::{Dossier, Erreur, donnees, journal, sauvegardes};

/// Vitesse du fond animé. Dans le fichier : « lente », « normale », « rapide ».
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Vitesse {
    Lente,
    #[default]
    Normale,
    Rapide,
}

/// Une valeur inconnue, ou qui n'est pas du texte, donne « normale » : un fichier modifié à la main ne casse rien.
impl<'de> Deserialize<'de> for Vitesse {
    fn deserialize<D: Deserializer<'de>>(entree: D) -> Result<Self, D::Error> {
        let valeur = serde_json::Value::deserialize(entree)?;
        Ok(
            match valeur.as_str().map(|s| s.trim().to_lowercase()).as_deref() {
                Some("lente") => Self::Lente,
                Some("rapide") => Self::Rapide,
                _ => Self::Normale,
            },
        )
    }
}

/// Le fond animé, réglages obligatoires de toute appli Aria : `fond.actif`, `fond.vitesse`, `fond.minimum` et
/// `fond.maximum` (nombre d'éléments tirés au hasard à chaque passage).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct ReglagesFond {
    pub actif: bool,
    pub vitesse: Vitesse,
    pub minimum: i64,
    pub maximum: i64,
}

impl ReglagesFond {
    /// Le moins d'éléments qu'un passage puisse tirer.
    pub const ELEMENTS_MIN: i64 = 1;
    /// Le plus d'éléments qu'un passage puisse tirer.
    pub const ELEMENTS_MAX: i64 = 12;

    /// Bornes ramenées entre 1 et 12, et remises dans l'ordre si le fichier les a inversées.
    #[must_use]
    pub fn assaini(self) -> Self {
        let bas = self.minimum.clamp(Self::ELEMENTS_MIN, Self::ELEMENTS_MAX);
        let haut = self.maximum.clamp(Self::ELEMENTS_MIN, Self::ELEMENTS_MAX);
        Self {
            minimum: bas.min(haut),
            maximum: bas.max(haut),
            ..self
        }
    }
}

impl Default for ReglagesFond {
    fn default() -> Self {
        Self {
            actif: true,
            vitesse: Vitesse::Normale,
            minimum: 2,
            maximum: 5,
        }
    }
}

/// Le numéro du format de `reglages.json`, écrit dans le fichier. Il monte quand un réglage change de sens ou de
/// forme ; [`Reglages::depuis_valeur`] reprend alors les anciens formats (migration).
pub const FORMAT: u64 = 1;

/// Les réglages de l'appli. Ceux du socle sont des champs ; ceux de l'appli vivent sous la clé `appli` du fichier
/// (voir [`Reglages::de_l_appli`]).
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Reglages {
    /// Le format du fichier : toujours [`FORMAT`] à l'écriture.
    pub format: u64,

    /// Fond animé ; absent du fichier, c'est celui par défaut.
    pub fond: ReglagesFond,

    /// Chercher une nouvelle version au lancement : une requête vers l'API GitHub. Décoché : plus aucune connexion
    /// pour ça.
    #[serde(rename = "chercherMisesAJour")]
    pub chercher_mises_a_jour: bool,

    /// L'assistant de premier lancement a été vu (terminé ou passé) : il ne s'ouvre plus de lui-même.
    #[serde(rename = "assistantVu")]
    pub assistant_vu: bool,

    /// Les réglages propres à l'appli, gardés tels quels : un objet JSON.
    pub appli: serde_json::Map<String, Value>,
}

impl Default for Reglages {
    fn default() -> Self {
        Self {
            format: FORMAT,
            fond: ReglagesFond::default(),
            chercher_mises_a_jour: true,
            assistant_vu: false,
            appli: serde_json::Map::new(),
        }
    }
}

/// Une lecture de réglages : ce qui a été retenu, et si le texte se relisait tel quel.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Lecture {
    pub reglages: Reglages,
    /// Faux si un champ a pris sa valeur par défaut faute d'être lisible, si le texte n'était pas un objet JSON, ou
    /// s'il vient d'un format plus récent : le réécrire perdrait quelque chose.
    pub intacte: bool,
}

/// Lit un champ ; sa valeur par défaut s'il manque, et `intacte` passe à faux s'il est illisible.
fn champ<T: DeserializeOwned>(
    objet: &serde_json::Map<String, Value>,
    cle: &str,
    defaut: T,
    intacte: &mut bool,
) -> T {
    match objet.get(cle) {
        None => defaut,
        Some(valeur) => serde_json::from_value(valeur.clone()).unwrap_or_else(|_| {
            *intacte = false;
            defaut
        }),
    }
}

impl ReglagesFond {
    /// Lit le fond champ par champ (voir [`Reglages::depuis_valeur`]).
    fn depuis_valeur(valeur: &Value, intacte: &mut bool) -> Self {
        let defaut = Self::default();
        let Some(objet) = valeur.as_object() else {
            *intacte = false;
            return defaut;
        };
        Self {
            actif: champ(objet, "actif", defaut.actif, intacte),
            // Vitesse se lit toujours (une valeur inconnue donne « normale ») : elle ne rend pas le fichier abîmé.
            vitesse: champ(objet, "vitesse", defaut.vitesse, intacte),
            minimum: champ(objet, "minimum", defaut.minimum, intacte),
            maximum: champ(objet, "maximum", defaut.maximum, intacte),
        }
    }
}

impl Reglages {
    /// Valeurs ramenées dans leurs bornes.
    #[must_use]
    pub fn assaini(self) -> Self {
        Self {
            format: FORMAT,
            fond: self.fond.assaini(),
            ..self
        }
    }

    /// Lit des réglages champ par champ : un champ illisible prend sa valeur par défaut sans emporter les autres.
    pub fn depuis_valeur(valeur: &Value) -> Lecture {
        let mut intacte = true;
        let Some(objet) = valeur.as_object() else {
            return Lecture {
                reglages: Self::default(),
                intacte: false,
            };
        };
        let defaut = Self::default();
        // Format absent : un fichier écrit avant les numéros de format, qui se lit comme le format 1.
        let format: u64 = champ(objet, "format", 1, &mut intacte);
        if format > FORMAT {
            // Écrit par une version plus récente de l'appli : lu au mieux, mais pas réécrit sans sauvegarde.
            intacte = false;
        }
        let reglages = Self {
            format: FORMAT,
            fond: objet.get("fond").map_or(defaut.fond, |fond| {
                ReglagesFond::depuis_valeur(fond, &mut intacte)
            }),
            chercher_mises_a_jour: champ(
                objet,
                "chercherMisesAJour",
                defaut.chercher_mises_a_jour,
                &mut intacte,
            ),
            assistant_vu: champ(objet, "assistantVu", defaut.assistant_vu, &mut intacte),
            appli: champ(objet, "appli", defaut.appli, &mut intacte),
        };
        Lecture {
            reglages: reglages.assaini(),
            intacte,
        }
    }

    /// Lit un texte JSON (voir [`Reglages::depuis_valeur`]).
    pub fn lire_json(texte: &str) -> Lecture {
        serde_json::from_str::<Value>(texte).map_or_else(
            |_| Lecture {
                reglages: Self::default(),
                intacte: false,
            },
            |valeur| Self::depuis_valeur(&valeur),
        )
    }

    /// Réglages lus dans un texte JSON ; un champ illisible prend sa valeur par défaut.
    pub fn depuis_json(texte: &str) -> Self {
        Self::lire_json(texte).reglages
    }

    /// Le texte JSON du fichier, indenté pour rester lisible à la main.
    ///
    /// # Errors
    /// Seulement si la sérialisation échoue, ce qui n'arrive pas avec ces types.
    pub fn vers_json(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.clone().assaini())
    }

    /// Lit le fichier ; un fichier absent donne les réglages par défaut, et compte comme intact.
    pub fn lire(dossier: &Dossier) -> Lecture {
        match fs::read_to_string(dossier.reglages()) {
            Ok(texte) => Self::lire_json(&texte),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Lecture {
                reglages: Self::default(),
                intacte: true,
            },
            Err(_) => Lecture {
                reglages: Self::default(),
                intacte: false,
            },
        }
    }

    /// Réglages enregistrés ; un champ absent ou illisible prend sa valeur par défaut.
    pub fn charger(dossier: &Dossier) -> Self {
        Self::lire(dossier).reglages
    }

    /// Enregistre les réglages, assainis, sans jamais laisser un fichier à moitié écrit. Si le fichier en place ne
    /// se relisait pas tel quel (champ illisible, format plus récent), il est d'abord sauvegardé, et le journal le
    /// dit : rien ne se perd sans copie.
    ///
    /// # Errors
    /// [`Erreur::Fichier`] si la sauvegarde ou l'écriture échoue ; l'ancien fichier reste alors intact.
    pub fn enregistrer(&self, dossier: &Dossier) -> Result<(), Erreur> {
        let chemin = dossier.reglages();
        if chemin.is_file() && !Self::lire(dossier).intacte {
            let copie = sauvegardes::sauvegarder(dossier, &chemin)?;
            journal::consigner(
                dossier,
                "réglages",
                &format!(
                    "reglages.json ne se relisait pas tel quel : copie gardée dans {}",
                    copie.display()
                ),
            );
        }
        let texte = self
            .vers_json()
            .map_err(|e| Erreur::fichier("préparation de", &chemin, e.into()))?;
        donnees::ecrire_entier(&chemin, texte.as_bytes())
    }

    /// Les réglages de l'appli sous sa clé `appli`, dans son propre type ; ses valeurs par défaut si la clé est
    /// absente ou ne se lit pas dans ce type.
    pub fn de_l_appli<T: DeserializeOwned + Default>(&self) -> T {
        serde_json::from_value(Value::Object(self.appli.clone())).unwrap_or_default()
    }

    /// Remplace les réglages de l'appli. Ils doivent former un objet JSON.
    ///
    /// # Errors
    /// Le type ne donne pas un objet JSON (un nombre, une liste…).
    pub fn regler_l_appli<T: Serialize>(&mut self, valeur: &T) -> Result<(), String> {
        match serde_json::to_value(valeur) {
            Ok(Value::Object(objet)) => {
                self.appli = objet;
                Ok(())
            }
            Ok(_) => Err("les réglages de l'appli doivent former un objet JSON".to_owned()),
            Err(e) => Err(format!("réglages de l'appli illisibles : {e}")),
        }
    }
}

/// Lu depuis l'interface comme depuis le fichier : champ par champ, un champ illisible (un nombre vidé, par
/// exemple) prend sa valeur par défaut au lieu de faire refuser tout l'enregistrement.
impl<'de> Deserialize<'de> for Reglages {
    fn deserialize<D: Deserializer<'de>>(entree: D) -> Result<Self, D::Error> {
        let valeur = Value::deserialize(entree)?;
        Ok(Self::depuis_valeur(&valeur).reglages)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn un_fichier_absent_donne_les_reglages_par_defaut() {
        let temp = tempfile::tempdir().unwrap();
        assert_eq!(Reglages::charger(&Dossier::a(temp.path())), Reglages::default());
    }

    #[test]
    fn un_fichier_illisible_donne_les_reglages_par_defaut() {
        assert_eq!(Reglages::depuis_json("{ pas du json"), Reglages::default());
        assert_eq!(Reglages::depuis_json("[]"), Reglages::default());
    }

    #[test]
    fn enregistrer_puis_charger_rend_les_memes_reglages() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path());
        let reglages = Reglages {
            fond: ReglagesFond {
                actif: false,
                vitesse: Vitesse::Rapide,
                minimum: 3,
                maximum: 9,
            },
            chercher_mises_a_jour: false,
            ..Reglages::default()
        };
        reglages.enregistrer(&dossier).unwrap();
        assert_eq!(Reglages::charger(&dossier), reglages);
    }

    #[test]
    fn le_fichier_garde_les_noms_francais() {
        let texte = Reglages::default().vers_json().unwrap();
        for cle in [
            "\"fond\"",
            "\"actif\"",
            "\"vitesse\": \"normale\"",
            "\"chercherMisesAJour\"",
            "\"format\": 1",
            "\"assistantVu\": false",
            "\"appli\": {}",
        ] {
            assert!(texte.contains(cle), "{cle} absent de {texte}");
        }
    }

    #[test]
    fn les_bornes_du_fond_sont_ramenees_et_remises_dans_l_ordre() {
        let r = Reglages::depuis_json(r#"{ "fond": { "minimum": 40, "maximum": -3 } }"#);
        assert_eq!((r.fond.minimum, r.fond.maximum), (1, 12));
        let r = Reglages::depuis_json(r#"{ "fond": { "minimum": 8, "maximum": 4 } }"#);
        assert_eq!((r.fond.minimum, r.fond.maximum), (4, 8));
    }

    #[test]
    fn une_vitesse_inconnue_donne_normale() {
        for valeur in ["\"turbo\"", "12", "null", "\" RAPIDE \""] {
            let texte = String::from(r#"{ "fond": { "vitesse": "#) + valeur + " } }";
            let r = Reglages::depuis_json(&texte);
            let attendue = if valeur.contains("RAPIDE") {
                Vitesse::Rapide
            } else {
                Vitesse::Normale
            };
            assert_eq!(r.fond.vitesse, attendue, "{valeur}");
        }
    }

    #[test]
    fn un_champ_manquant_prend_sa_valeur_par_defaut() {
        let r = Reglages::depuis_json(r#"{ "chercherMisesAJour": false }"#);
        assert!(!r.chercher_mises_a_jour);
        assert_eq!(r.fond, ReglagesFond::default());
        let r = Reglages::depuis_json(r#"{ "fond": { "actif": false } }"#);
        assert!(r.chercher_mises_a_jour);
    }

    #[test]
    fn un_champ_illisible_n_emporte_pas_les_autres() {
        let lecture = Reglages::lire_json(
            r#"{ "chercherMisesAJour": "non", "fond": { "actif": false, "minimum": "x", "maximum": 9 } }"#,
        );
        assert!(!lecture.intacte);
        assert!(lecture.reglages.chercher_mises_a_jour, "valeur par défaut");
        assert!(!lecture.reglages.fond.actif, "gardé");
        assert_eq!(lecture.reglages.fond.maximum, 9, "gardé");
        assert_eq!(lecture.reglages.fond.minimum, ReglagesFond::default().minimum);
    }

    #[test]
    fn un_fichier_bien_forme_est_intact() {
        let texte = Reglages::default().vers_json().unwrap();
        assert!(Reglages::lire_json(&texte).intacte);
        // Sans numéro de format : un fichier d'avant les formats, intact lui aussi.
        assert!(Reglages::lire_json(r#"{ "chercherMisesAJour": true }"#).intacte);
    }

    #[test]
    fn un_format_plus_recent_n_est_pas_intact() {
        let lecture = Reglages::lire_json(r#"{ "format": 99, "chercherMisesAJour": false }"#);
        assert!(!lecture.intacte);
        assert!(!lecture.reglages.chercher_mises_a_jour);
    }

    #[test]
    fn un_fichier_abime_est_sauvegarde_avant_d_etre_reecrit() {
        let temp = tempfile::tempdir().unwrap();
        let dossier = Dossier::a(temp.path());
        dossier.preparer().unwrap();
        fs::write(dossier.reglages(), r#"{ "chercherMisesAJour": "oui" }"#).unwrap();
        Reglages::default().enregistrer(&dossier).unwrap();
        let copies: Vec<_> = fs::read_dir(dossier.sauvegardes())
            .unwrap()
            .filter_map(Result::ok)
            .filter(|e| e.file_name().to_string_lossy().ends_with("reglages.json"))
            .collect();
        assert_eq!(copies.len(), 1);
        let garde = fs::read_to_string(copies[0].path()).unwrap();
        assert!(garde.contains("\"oui\""));
        assert!(Reglages::lire(&dossier).intacte);
        // Un fichier intact n'est pas recopié à chaque enregistrement.
        Reglages::default().enregistrer(&dossier).unwrap();
        assert_eq!(sauvegardes::lister(&dossier).len(), 1);
    }

    #[test]
    fn les_reglages_de_l_appli_vont_et_viennent() {
        #[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
        struct DeLAppli {
            seuil: u32,
        }
        let mut reglages = Reglages::default();
        assert_eq!(reglages.de_l_appli::<DeLAppli>(), DeLAppli::default());
        reglages.regler_l_appli(&DeLAppli { seuil: 7 }).unwrap();
        let relu = Reglages::depuis_json(&reglages.vers_json().unwrap());
        assert_eq!(relu.de_l_appli::<DeLAppli>(), DeLAppli { seuil: 7 });
        assert!(reglages.regler_l_appli(&3).is_err());
    }

    #[test]
    fn l_interface_peut_envoyer_un_nombre_vide() {
        let reglages: Reglages =
            serde_json::from_str(r#"{ "fond": { "minimum": null, "maximum": 6 } }"#).unwrap();
        assert_eq!(reglages.fond.minimum, ReglagesFond::default().minimum);
        assert_eq!(reglages.fond.maximum, 6);
    }
}
