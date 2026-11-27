// Les commandes Rust de l'appli que l'interface peut appeler (appli\src\commandes.rs), avec leurs types. Celles du
// socle viennent du paquet aria-interface. Une commande ajoutée ici doit aussi l'être dans appli\build.rs et dans
// appli\capabilities\principale.json.
import { invoke } from '@tauri-apps/api/core';

/** Place libre et taille du disque, en octets. */
export interface Espace {
  libre: number;
  total: number;
}

export const mesurerDisque = (): Promise<Espace> => invoke<Espace>('mesurer_disque');
