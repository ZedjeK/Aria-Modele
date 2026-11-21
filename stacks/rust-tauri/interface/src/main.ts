// Point d'entrée de l'interface : polices embarquées, thème Aria, puis l'appli.
import '@fontsource/space-grotesk/latin-500.css';
import '@fontsource/space-grotesk/latin-600.css';
import '@fontsource/space-grotesk/latin-700.css';
import '@fontsource/ibm-plex-mono/latin-400.css';
import '@fontsource/ibm-plex-mono/latin-700.css';
import './lib/theme.css';

import { mount } from 'svelte';
import App from './App.svelte';

const cible = document.getElementById('appli');
if (!cible) {
  throw new Error("L'élément #appli manque dans index.html.");
}

export default mount(App, { target: cible });
