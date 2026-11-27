import { describe, expect, it } from 'vitest';
import type { Avancement } from './api';
import { tachesEnCours } from './taches.svelte';

const tache = (id: number, etape: Avancement['etape']): Avancement => ({
  id,
  libelle: `tâche ${id}`,
  progression: null,
  detail: '',
  etape,
});

describe('tachesEnCours', () => {
  it('ne garde que les tâches en cours, la plus ancienne d’abord', () => {
    const liste = { 3: tache(3, 'en-cours'), 1: tache(1, 'en-cours'), 2: tache(2, 'finie'), 4: tache(4, 'echec') };
    expect(tachesEnCours(liste).map((t) => t.id)).toEqual([1, 3]);
  });
});
