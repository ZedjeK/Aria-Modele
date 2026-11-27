// Point d'entrée de l'interface d'Aria {{NOM}} : le socle (aria-interface) charge les polices et le thème, puis montre
// la vue de la fenêtre : celle de l'appli (App.svelte) dans la fenêtre principale, la sienne dans les autres.
import { demarrer } from 'aria-interface';
import App from './App.svelte';

export default demarrer(App);
