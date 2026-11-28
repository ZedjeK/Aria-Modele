// Publie la liste des commandes de la fenêtre du socle au build.rs de l'appli, qui les autorise avec les siennes
// (tauri_build::AppManifest) : DEP_ARIA_COEUR_COMMANDES, des noms séparés par des virgules. Les commandes d'un module
// en option n'y sont que si sa fonction est activée.
include!("src/fenetre/liste.rs");

fn main() {
    println!("cargo::rerun-if-changed=src/fenetre/liste.rs");
    let mut commandes = COMMANDES.to_vec();
    for (fonction, du_module) in MODULES {
        if std::env::var_os(fonction).is_some() {
            commandes.extend_from_slice(du_module);
        }
    }
    println!("cargo::metadata=commandes={}", commandes.join(","));
}
