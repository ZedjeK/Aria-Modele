// Ce que l'appli donne à la fenêtre du socle (Fenetre.svelte), et ce qu'elle reçoit pour ses pages.
import type { Identite, Reglages } from './api';

/** Une famille de pages du menu latéral : son titre, puis ses pages. */
export interface Famille {
  titre: string;
  pages: { id: string; titre: string }[];
}

/** Ce que reçoit chaque page de l'appli. */
export interface Outils {
  /** L'identité de l'appli ; null tant qu'elle n'est pas lue. */
  readonly identite: Identite | null;
  /** Les réglages communs ; null tant qu'ils ne sont pas lus. */
  readonly reglages: Reglages | null;
  /** Écrit dans la ligne d'état, en rouge pour une erreur. */
  signaler: (texte: string, erreur?: boolean) => void;
}
