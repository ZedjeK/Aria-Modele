// Démarre l'interface d'une appli Aria : polices embarquées, thème, puis la vue de la fenêtre. La fenêtre principale
// montre la vue de l'appli (son App.svelte, posée sur Fenetre.svelte) ; le menu de l'icône (fenêtre « menu ») et la
// proposition d'installation (fenêtre « installation ») sont ceux du socle.
import '@fontsource/space-grotesk/latin-500.css';
import '@fontsource/space-grotesk/latin-600.css';
import '@fontsource/space-grotesk/latin-700.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import '@fontsource/ibm-plex-mono/latin-700.css';
import './theme.css';

import { getCurrentWebview } from '@tauri-apps/api/webview';
import { mount, type Component } from 'svelte';
import Installation from './vues/Installation.svelte';
import MenuIcone from './vues/MenuIcone.svelte';

/** Monte la vue de la fenêtre dans l'élément #appli d'index.html. */
export function demarrer(appli: Component): ReturnType<typeof mount> {
  const cible = document.getElementById('appli');
  if (!cible) {
    throw new Error("L'élément #appli manque dans index.html.");
  }
  const vues: Record<string, Component> = { menu: MenuIcone, installation: Installation };
  const vue = vues[getCurrentWebview().label] ?? appli;
  return mount(vue, { target: cible });
}
