// Point d'entrée de l'interface : polices embarquées, thème Aria, puis la vue de la fenêtre : la fenêtre
// principale, ou le menu de l'icône de notification (fenêtre « menu », appli en arrière-plan seulement).
import '@fontsource/space-grotesk/latin-500.css';
import '@fontsource/space-grotesk/latin-600.css';
import '@fontsource/space-grotesk/latin-700.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import '@fontsource/ibm-plex-mono/latin-700.css';
import './lib/theme.css';

import { getCurrentWebview } from '@tauri-apps/api/webview';
import { mount } from 'svelte';
import App from './App.svelte';
import MenuIcone from './MenuIcone.svelte';

const cible = document.getElementById('appli');
if (!cible) {
  throw new Error("L'élément #appli manque dans index.html.");
}

export default getCurrentWebview().label === 'menu' ? mount(MenuIcone, { target: cible }) : mount(App, { target: cible });
