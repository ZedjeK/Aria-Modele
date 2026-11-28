/// Les commandes que la fenêtre du socle expose à l'interface. Une seule liste, lue par build.rs (qui la publie à
/// l'appli) et par `gestionnaire` (qui y aiguille les appels) : une commande ajoutée au socle s'ajoute ici et dans
/// `tauri::generate_handler!` de `fenetre::commandes_du_socle`, et les tests vérifient que les deux se suivent.
pub const COMMANDES: &[&str] = &[
    "identite",
    "lire_reglages",
    "enregistrer_reglages",
    "lire_journal",
    "quitter",
    "diagnostic",
    "lire_demarrage_auto",
    "regler_demarrage_auto",
    "etat_menu",
    "action_menu",
    "taille_menu",
    "infos_installation",
    "installer",
    "refuser_installation",
    "mise_a_jour_disponible",
    "voir_notes",
    "installer_mise_a_jour",
    "desinstaller",
    "fermer_apres_desinstallation",
    "lister_sauvegardes",
    "restaurer_sauvegarde",
    "comparer_sauvegarde",
    "bilan_reprise",
    "annuler_tache",
];

/// Les commandes du module « Fichiers et dossiers », présentes seulement avec la fonction `fichiers`.
// Lue par build.rs et, avec la fonction, par l'aiguillage des appels : sans elle, seulement par build.rs.
#[allow(dead_code)]
pub const COMMANDES_FICHIERS: &[&str] = &[
    "choisir_fichiers",
    "choisir_dossier",
    "choisir_enregistrement",
    "fichiers_ouverts",
    "lire_associations",
    "regler_association",
];

/// Les commandes du module « Notifications », présentes seulement avec la fonction `notifications`.
// Lue par build.rs et, avec la fonction, par l'aiguillage des appels.
#[allow(dead_code)]
pub const COMMANDES_NOTIFICATIONS: &[&str] = &["notifier"];

/// Les modules en option et leurs commandes : la variable `CARGO_FEATURE_*` de la fonction, puis la liste. build.rs
/// publie les commandes d'un module seulement si sa fonction est activée.
// Lue par build.rs et par les tests.
#[allow(dead_code)]
pub const MODULES: &[(&str, &[&str])] = &[
    ("CARGO_FEATURE_FICHIERS", COMMANDES_FICHIERS),
    ("CARGO_FEATURE_NOTIFICATIONS", COMMANDES_NOTIFICATIONS),
];
