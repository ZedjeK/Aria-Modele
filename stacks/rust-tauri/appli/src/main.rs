//! Aria {{NOM}} : l'exe Windows. Le socle (crate aria-coeur, fonction `fenetre`) prépare le dossier de données,
//! propose l'installation, ouvre la fenêtre Tauri et tient tout le contrat ; l'appli lui donne son identité et ses
//! propres commandes, listées dans build.rs.

// Pas de console derrière la fenêtre, sauf en débogage.
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod commandes;
mod disque;

use aria_coeur::fenetre::{self, Appli};
use aria_coeur::identite::Identite;

/// La promesse de l'appli, en une phrase.
// Laissée sur une ligne quelle que soit sa longueur : init.ps1 y écrit une promesse de 200 caractères au plus, et
// rustfmt couperait la ligne au-delà de la largeur fixée par rustfmt.toml (issue #57).
#[rustfmt::skip]
const PROMESSE: &str = "{{PROMESSE}}";

/// Ce que l'appli dit d'elle-même, rempli par init.ps1.
const IDENTITE: Identite = Identite {
    nom: "{{NOM}}",
    promesse: PROMESSE,
    // La version de `[workspace.package]` dans Cargo.toml : celle de l'appli, pas celle du socle.
    version: env!("CARGO_PKG_VERSION"),
    en_fond: {{EN_FOND}},
    installable: {{INSTALLABLE}},
};

fn main() {
    fenetre::lancer(
        IDENTITE,
        Appli::new(
            tauri::generate_context!(),
            tauri::generate_handler![commandes::mesurer_disque],
        ),
    );
}
