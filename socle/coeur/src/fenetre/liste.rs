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
