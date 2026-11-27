// Prépare la compilation Tauri : icône et version dans l'exe, et une autorisation par commande. Une commande absente
// de ces listes n'est pas appelable depuis l'interface, même si le code Rust l'expose.
//
// Les commandes du socle viennent du socle lui-même (DEP_ARIA_COEUR_COMMANDES, publiée par son build.rs) : l'appli
// ne liste que les siennes.

/// Les commandes de l'appli (main.rs, `generate_handler!`).
const COMMANDES_APPLI: &[&str] = &["mesurer_disque"];

fn main() {
    let socle = std::env::var("DEP_ARIA_COEUR_COMMANDES").unwrap_or_default();
    let mut commandes: Vec<&'static str> = socle
        .split(',')
        .filter(|c| !c.is_empty())
        .map(|c| &*c.to_owned().leak())
        .collect();
    if commandes.is_empty() {
        // Un script de build signale son échec en s'arrêtant : cargo affiche le message.
        panic!("le socle (aria-coeur) n'a pas publié ses commandes");
    }
    commandes.extend_from_slice(COMMANDES_APPLI);
    let commandes = tauri_build::AppManifest::new().commands(commandes.leak());
    let attributs = tauri_build::Attributes::new().app_manifest(commandes);
    if let Err(erreur) = tauri_build::try_build(attributs) {
        panic!("préparation de Tauri impossible : {erreur:#}");
    }
}
