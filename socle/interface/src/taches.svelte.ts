// Les tâches longues de l'appli, suivies par l'événement « tache » : une seule écoute pour toute la fenêtre. Une
// tâche finie reste visible quelques secondes, le temps de lire son bilan.
import { ecouter, type Avancement } from './api';

/** Les tâches connues, par identifiant. */
export const taches = $state<Record<number, Avancement>>({});

/** Le temps pendant lequel une tâche terminée reste affichée. */
const AFFICHAGE_APRES_FIN = 6000;

let ecoute: Promise<() => void> | null = null;

/** Commence à suivre les tâches (une seule fois par fenêtre). */
export function suivreTaches(): void {
  ecoute ??= ecouter<Avancement>('tache', (avancement) => {
    taches[avancement.id] = avancement;
    if (avancement.etape !== 'en-cours') {
      setTimeout(() => {
        if (taches[avancement.id]?.etape !== 'en-cours') delete taches[avancement.id];
      }, AFFICHAGE_APRES_FIN);
    }
  });
}

/** Les tâches en cours, la plus ancienne d'abord. */
export function tachesEnCours(liste: Record<number, Avancement>): Avancement[] {
  return Object.values(liste)
    .filter((t) => t.etape === 'en-cours')
    .sort((a, b) => a.id - b.id);
}
