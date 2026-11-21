// Point d'entrée de l'interface : polices embarquées, thème Aria, puis la vue de la fenêtre : la fenêtre
// principale, le menu de l'icône de notification (fenêtre « menu », appli en arrière-plan) ou la proposition
// d'installation (fenêtre « installation », exe lancé hors de son dossier).
import '@fontsource/space-grotesk/latin-500.css';
import '@fontsource/space-grotesk/latin-600.css';
import '@fontsource/space-grotesk/latin-700.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import '@fontsource/ibm-plex-mono/latin-700.css';
import './lib/theme.css';

import { getCurrentWebview } from '@tauri-apps/api/webview';
import { mount } from 'svelte';
import App from './App.svelte';
import Installation from './Installation.svelte';
import MenuIcone from './MenuIcone.svelte';

const cible = document.getElementById('appli');
if (!cible) {
  throw new Error("L'élément #appli manque dans index.html.");
}

const vues = { menu: MenuIcone, installation: Installation } as const;
const etiquette = getCurrentWebview().label;
const vue = etiquette === 'menu' || etiquette === 'installation' ? vues[etiquette] : App;

export default mount(vue, { target: cible });
