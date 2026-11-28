// aria-interface : l'interface commune des applis Aria (docs/CONTRAT.md du dépôt modèle, « Identité visuelle »).
//
// L'appli monte sa vue par `demarrer`, la pose sur `Fenetre`, et compose ses pages avec les composants ci-dessous.
// Sa version est celle du dépôt modèle, la même que celle de la crate aria-coeur.
export { demarrer } from './demarrer';
export type { Famille, Outils } from './fenetre';
export * from './api';
export { NON_MESURE, octetsLisibles, segmentsRemplis, texteJauge } from './jauge';
export { dateLisible } from './journal';
export { couleurTon, type Pastille } from './menu';

export { default as Fenetre } from './Fenetre.svelte';
export { default as ASavoir } from './composants/ASavoir.svelte';
export { default as Bandeau } from './composants/Bandeau.svelte';
export { default as Bouton } from './composants/Bouton.svelte';
export { default as CarteChoix } from './composants/CarteChoix.svelte';
export { default as Dialogue } from './composants/Dialogue.svelte';
export { default as Jauge } from './composants/Jauge.svelte';
export { default as Panneau } from './composants/Panneau.svelte';
export { default as SousOnglets } from './composants/SousOnglets.svelte';
export { default as TacheEnCours } from './composants/TacheEnCours.svelte';
export { default as ZoneDepot } from './composants/ZoneDepot.svelte';
export { taches, tachesEnCours } from './taches.svelte';
