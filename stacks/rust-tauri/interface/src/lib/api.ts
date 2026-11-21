// Les commandes Rust que l'interface peut appeler (appli\src\commandes.rs), avec leurs types.
// Une commande ajoutée ici doit aussi l'être dans appli\build.rs et appli\capabilities\principale.json.
import { invoke } from '@tauri-apps/api/core';

export type Vitesse = 'lente' | 'normale' | 'rapide';

export interface ReglagesFond {
  actif: boolean;
  vitesse: Vitesse;
  minimum: number;
  maximum: number;
}

export interface Reglages {
  fond: ReglagesFond;
  proposerInstallation: boolean;
  chercherMisesAJour: boolean;
}

export interface Identite {
  nom: string;
  version: string;
  promesse: string;
  enFond: boolean;
  installable: boolean;
  dossier: string;
}

/** Place libre et taille du disque, en octets. */
export interface Espace {
  libre: number;
  total: number;
}

export const identite = (): Promise<Identite> => invoke<Identite>('identite');

export const lireReglages = (): Promise<Reglages> => invoke<Reglages>('lire_reglages');

/** Renvoie les réglages vraiment enregistrés : les bornes peuvent avoir été ramenées. */
export const enregistrerReglages = (reglages: Reglages): Promise<Reglages> =>
  invoke<Reglages>('enregistrer_reglages', { reglages });

export const mesurerDisque = (): Promise<Espace> => invoke<Espace>('mesurer_disque');

/** Le message d'une erreur renvoyée par une commande Rust (du texte) ou par le navigateur. */
export function messageErreur(erreur: unknown): string {
  if (typeof erreur === 'string') return erreur;
  if (erreur instanceof Error) return erreur.message;
  return 'erreur inconnue';
}
