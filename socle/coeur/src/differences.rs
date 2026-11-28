//! Les différences entre deux documents JSON, champ par champ : ce qu'un enregistrement des réglages a changé, ou ce
//! qu'une restauration remettrait. Les champs dont le nom évoque un secret sont masqués : les secrets n'ont rien à
//! faire dans ces fichiers (contrat), mais une erreur d'appli ne doit pas les afficher pour autant.

use serde::Serialize;
use serde_json::Value;

/// Un champ qui diffère. `None` : absent de ce côté.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct Difference {
    /// Le chemin du champ, ses clés séparées par des points (`fond.vitesse`).
    pub champ: String,
    pub avant: Option<String>,
    pub apres: Option<String>,
}

/// Ce qui remplace la valeur d'un champ qui pourrait être secret.
pub const MASQUE: &str = "(masqué)";

/// Un nom de champ qui évoque un secret.
fn sensible(cle: &str) -> bool {
    let cle = cle.to_lowercase();
    [
        "secret",
        "jeton",
        "token",
        "motdepasse",
        "mot_de_passe",
        "password",
        "cle",
        "key",
    ]
    .iter()
    .any(|mot| cle.contains(mot))
}

/// Une valeur lisible : le texte sans guillemets, le reste en JSON compact.
fn lisible(valeur: &Value) -> String {
    match valeur {
        Value::String(texte) => texte.clone(),
        autre => autre.to_string(),
    }
}

fn parcourir(
    chemin: &str,
    avant: Option<&Value>,
    apres: Option<&Value>,
    masquer: bool,
    sortie: &mut Vec<Difference>,
) {
    match (avant, apres) {
        (Some(Value::Object(a)), Some(Value::Object(b))) => {
            let mut cles: Vec<&String> = a.keys().chain(b.keys()).collect();
            cles.sort();
            cles.dedup();
            for cle in cles {
                let sous = if chemin.is_empty() {
                    cle.clone()
                } else {
                    format!("{chemin}.{cle}")
                };
                parcourir(&sous, a.get(cle), b.get(cle), masquer || sensible(cle), sortie);
            }
        }
        (a, b) if a == b => {}
        (a, b) => {
            let montrer = |v: Option<&Value>| v.map(|v| if masquer { MASQUE.to_owned() } else { lisible(v) });
            sortie.push(Difference {
                champ: chemin.to_owned(),
                avant: montrer(a),
                apres: montrer(b),
            });
        }
    }
}

/// Les champs qui diffèrent de `avant` à `apres`, dans l'ordre de leurs chemins. Un objet est comparé clé par clé ;
/// une liste ou une valeur simple, en entier.
pub fn entre(avant: &Value, apres: &Value) -> Vec<Difference> {
    let mut sortie = Vec::new();
    parcourir("", Some(avant), Some(apres), false, &mut sortie);
    sortie
}

/// Une ligne pour le journal : au plus `max` champs, puis leur nombre restant.
pub fn resumer(differences: &[Difference], max: usize) -> String {
    let montrer = |v: &Option<String>| v.clone().unwrap_or_else(|| "(absent)".to_owned());
    let mut morceaux: Vec<String> = differences
        .iter()
        .take(max)
        .map(|d| format!("{} : {} → {}", d.champ, montrer(&d.avant), montrer(&d.apres)))
        .collect();
    if differences.len() > max {
        morceaux.push(format!("et {} autre(s)", differences.len() - max));
    }
    morceaux.join(" ; ")
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn seuls_les_champs_changes_sont_rendus() {
        let avant = json!({ "fond": { "vitesse": "normale", "actif": true }, "format": 1, "appli": {} });
        let apres =
            json!({ "fond": { "vitesse": "rapide", "actif": true }, "format": 1, "appli": { "seuil": 3 } });
        let d = entre(&avant, &apres);
        assert_eq!(
            d,
            [
                Difference {
                    champ: "appli.seuil".to_owned(),
                    avant: None,
                    apres: Some("3".to_owned())
                },
                Difference {
                    champ: "fond.vitesse".to_owned(),
                    avant: Some("normale".to_owned()),
                    apres: Some("rapide".to_owned())
                },
            ]
        );
        assert_eq!(resumer(&d, 1), "appli.seuil : (absent) → 3 ; et 1 autre(s)");
        assert!(entre(&avant, &avant).is_empty());
    }

    #[test]
    fn un_champ_qui_evoque_un_secret_est_masque() {
        let d = entre(
            &json!({ "appli": { "jetonGithub": "a" } }),
            &json!({ "appli": { "jetonGithub": "b" } }),
        );
        assert_eq!(d[0].avant.as_deref(), Some(MASQUE));
        assert_eq!(d[0].apres.as_deref(), Some(MASQUE));
        let d = entre(
            &json!({ "acces": { "nom": "x" } }),
            &json!({ "acces": { "nom": "y" } }),
        );
        assert_eq!(d[0].apres.as_deref(), Some("y"));
        // Tout ce qui est sous une clé sensible est masqué.
        let d = entre(
            &json!({ "secrets": { "a": 1 } }),
            &json!({ "secrets": { "a": 2 } }),
        );
        assert_eq!(d[0].apres.as_deref(), Some(MASQUE));
    }

    #[test]
    fn des_types_differents_se_comparent_en_entier() {
        let d = entre(&json!({ "a": [1, 2] }), &json!({ "a": "x" }));
        assert_eq!(d[0].avant.as_deref(), Some("[1,2]"));
        assert_eq!(d[0].apres.as_deref(), Some("x"));
    }
}
