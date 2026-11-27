// Contrat, « Accessibilité » : le texte et les éléments qui agissent gardent un contraste d'au moins 4,5 pour 1
// avec leur fond. Les couleurs sont lues dans theme.css, la seule source de la palette.
import { describe, expect, it } from 'vitest';
import theme from './theme.css?raw';

function couleur(nom: string): string {
  const trouvee = new RegExp(`--${nom}:\\s*(#[0-9a-fA-F]{6})`).exec(theme);
  if (!trouvee?.[1]) throw new Error(`--${nom} absente de theme.css`);
  return trouvee[1];
}

/** Luminance relative (WCAG 2). */
function luminance(hexa: string): number {
  const canaux = [1, 3, 5].map((i) => parseInt(hexa.slice(i, i + 2), 16) / 255);
  const [r = 0, v = 0, b = 0] = canaux.map((c) => (c <= 0.04045 ? c / 12.92 : ((c + 0.055) / 1.055) ** 2.4));
  return 0.2126 * r + 0.7152 * v + 0.0722 * b;
}

function contraste(a: string, b: string): number {
  const [clair, sombre] = [luminance(a), luminance(b)].sort((x, y) => y - x);
  return ((clair ?? 0) + 0.05) / ((sombre ?? 0) + 0.05);
}

// Texte et éléments qui agissent, sur chaque fond où ils se posent.
const paires: [texte: string, fond: string][] = [
  ['texte', 'fond'],
  ['texte', 'panneau'],
  ['texte-2', 'fond'],
  ['texte-2', 'panneau'],
  ['cyan', 'fond'],
  ['cyan', 'panneau'],
  ['bleu', 'panneau'],
  ['rouge', 'panneau'],
  ['ambre', 'panneau'],
  ['orange', 'panneau'],
  // Le texte du bouton principal, sur l'orange ; celui d'une carte choisie, sur le cyan.
  ['fond', 'orange'],
  ['fond', 'cyan'],
];

describe('contraste de la palette', () => {
  for (const [texte, fond] of paires) {
    it(`${texte} sur ${fond} : au moins 4,5 pour 1`, () => {
      expect(contraste(couleur(texte), couleur(fond))).toBeGreaterThanOrEqual(4.5);
    });
  }
});
