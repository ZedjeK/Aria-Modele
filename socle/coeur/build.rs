// Publie la liste des commandes de la fenêtre du socle au build.rs de l'appli, qui les autorise avec les siennes
// (tauri_build::AppManifest) : DEP_ARIA_COEUR_COMMANDES, des noms séparés par des virgules.
include!("src/fenetre/liste.rs");

fn main() {
    println!("cargo::rerun-if-changed=src/fenetre/liste.rs");
    println!("cargo::metadata=commandes={}", COMMANDES.join(","));
}
