<!--
  La fenêtre principale d'une appli Aria : menu latéral à gauche (marque, familles de pages, Quitter, version), et à
  droite la zone des pages sur son décor (nuit profonde, grille fine, halos, fond animé) : barre de titre avec le
  chemin et l'état en direct, bandeau de mise à jour, la page, puis la ligne d'état. L'appli donne son logo, ses
  familles de pages, ses pastilles et le contenu de ses pages ; le Journal et les Paramètres viennent du socle.
-->
<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import {
    ecouter,
    enregistrerReglages,
    identite as lireIdentite,
    lireReglages,
    messageErreur,
    miseAJourDisponible,
    quitter,
    type Identite,
    type InfosMiseAJour,
    type Reglages,
  } from './api';
  import type { Famille, Outils } from './fenetre';
  import type { Pastille } from './menu';
  import Bandeau from './composants/Bandeau.svelte';
  import BarreTitre from './composants/BarreTitre.svelte';
  import Bouton from './composants/Bouton.svelte';
  import EtatDirect from './composants/EtatDirect.svelte';
  import FondAnime from './composants/FondAnime.svelte';
  import MenuLateral from './composants/MenuLateral.svelte';
  import DialogueMiseAJour from './dialogues/MiseAJour.svelte';
  import Journal from './pages/Journal.svelte';
  import Parametres from './pages/Parametres.svelte';

  interface Props {
    /** Le logo de l'appli : `import logo from './logo.svg'`. */
    logo: string;
    /** Les familles des pages du travail de l'appli ; la famille « Appli » (Journal, Paramètres) vient du socle. */
    familles: Famille[];
    /** Les pastilles d'état de l'appli, avant celle des mises à jour, qui vient du socle. */
    pastilles?: Pastille[];
    /** La page de l'appli à montrer, d'après son identifiant. */
    contenu: Snippet<[string, Outils]>;
  }

  let { logo, familles, pastilles = [], contenu }: Props = $props();

  /** Les pages de l'appli elle-même, à part du travail de l'appli. */
  const familleAppli: Famille = {
    titre: 'Appli',
    pages: [
      { id: 'journal', titre: 'Journal' },
      { id: 'parametres', titre: 'Paramètres' },
    ],
  };
  const toutes = $derived([...familles, familleAppli]);

  let identite = $state<Identite | null>(null);
  let reglages = $state<Reglages | null>(null);
  // svelte-ignore state_referenced_locally
  let page = $state<string>(familles[0]?.pages[0]?.id ?? 'journal');
  let miseAJour = $state<InfosMiseAJour | null>(null);
  let bandeauRange = $state(false);
  let dialogueMiseAJour = $state(false);
  let etat = $state('');
  let erreur = $state(false);

  const nomCourt = $derived(identite?.nom.replace(/^Aria\s+/, '') ?? '');
  const famille = $derived(toutes.find((f) => f.pages.some((p) => p.id === page)));
  const titrePage = $derived(famille?.pages.find((p) => p.id === page)?.titre ?? '');
  const toutesPastilles: Pastille[] = $derived([
    ...pastilles,
    {
      nom: 'Mises à jour',
      detail: miseAJour
        ? `${miseAJour.version} disponible`
        : reglages?.chercherMisesAJour === false
          ? 'désactivées'
          : 'aucune connue',
      ton: miseAJour ? 'information' : 'repos',
    },
  ]);

  function signaler(texte: string, estErreur = false) {
    etat = texte;
    erreur = estErreur;
  }

  const outils: Outils = {
    get identite() {
      return identite;
    },
    get reglages() {
      return reglages;
    },
    signaler,
  };

  onMount(() => {
    void (async () => {
      try {
        [identite, reglages] = await Promise.all([lireIdentite(), lireReglages()]);
        miseAJour ??= await miseAJourDisponible();
      } catch (e) {
        signaler(`Démarrage incomplet : ${messageErreur(e)}`, true);
      }
    })();
    // « Paramètres » et « Voir la mise à jour » dans le menu de l'icône.
    const arretAllerA = ecouter<string>('aller-a', (cible) => {
      if (cible === 'parametres') page = 'parametres';
      if (cible === 'mise-a-jour' && miseAJour) dialogueMiseAJour = true;
    });
    // Une version plus récente trouvée au lancement (la recherche ne bloque pas l'appli).
    const arretMiseAJour = ecouter<string>('mise-a-jour', () => {
      void miseAJourDisponible().then((infos) => (miseAJour = infos));
    });
    return () => {
      void arretAllerA.then((f) => f());
      void arretMiseAJour.then((f) => f());
    };
  });

  async function enregistrer() {
    if (!reglages) return;
    try {
      reglages = await enregistrerReglages($state.snapshot(reglages));
      signaler('Réglages enregistrés.');
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  }
</script>

<div class="fenetre">
  <MenuLateral
    {logo}
    nom={nomCourt}
    version={identite?.version ?? ''}
    familles={toutes}
    bind:page
    quitter={identite?.enFond ? () => void quitter() : undefined}
  />

  <main class="zone">
    <!-- Décor : nuit profonde, grille fine, halo bleu au centre, halo orange en bas à droite. -->
    <div class="decor" aria-hidden="true"></div>
    {#if reglages}
      <FondAnime reglages={reglages.fond} />
    {/if}

    <BarreTitre>
      <span class="chemin" data-tauri-drag-region>{famille?.titre ?? ''} <b>/ {titrePage}</b></span>
      <EtatDirect pastilles={toutesPastilles} />
    </BarreTitre>

    {#if miseAJour && !bandeauRange}
      <Bandeau
        etiquette="Mise à jour disponible"
        texte={`La version ${miseAJour.version} est publiée (en cours : ${miseAJour.actuelle}).`}
      >
        <Bouton genre="discret" petit onclick={() => (bandeauRange = true)}>Plus tard</Bouton>
        <Bouton petit onclick={() => (dialogueMiseAJour = true)}>Voir</Bouton>
      </Bandeau>
    {/if}

    <div class="page">
      {#if page === 'journal'}
        <Journal {signaler} />
      {:else if page === 'parametres'}
        {#if identite && reglages}
          <Parametres {identite} bind:reglages {enregistrer} {signaler} />
        {/if}
      {:else}
        {@render contenu(page, outils)}
      {/if}

      <p class="etat" class:erreur role="status">{etat}</p>
    </div>
  </main>
</div>

{#if dialogueMiseAJour && miseAJour}
  <DialogueMiseAJour infos={miseAJour} fermer={() => (dialogueMiseAJour = false)} />
{/if}

<style>
  .fenetre {
    display: grid;
    grid-template-columns: 220px minmax(0, 1fr);
    height: 100%;
  }

  .zone {
    position: relative;
    display: flex;
    flex-direction: column;
    min-width: 0;
    overflow: hidden;
    background: linear-gradient(180deg, var(--fond) 0%, #061026 55%, var(--fond) 100%);
  }

  .decor {
    position: absolute;
    inset: 0;
    pointer-events: none;
    background:
      radial-gradient(ellipse 550px 450px at 50% 50%, rgb(92 139 255 / 18%), transparent 70%),
      radial-gradient(ellipse 450px 350px at 100% 100%, rgb(255 138 31 / 9%), transparent 70%),
      linear-gradient(rgb(125 249 255 / 6%) 1px, transparent 1px) 0 0 / 40px 40px,
      linear-gradient(90deg, rgb(125 249 255 / 6%) 1px, transparent 1px) 0 0 / 40px 40px;
  }

  .chemin {
    flex: 1;
    color: var(--texte-2);
    font-size: 11px;
    font-weight: 500;
    letter-spacing: 0.16em;
    text-transform: uppercase;
  }

  .chemin b {
    color: var(--cyan);
    font-weight: 500;
  }

  .page {
    position: relative;
    z-index: 1;
    display: flex;
    flex: 1;
    flex-direction: column;
    gap: 16px;
    width: 100%;
    max-width: 1100px;
    margin: 0 auto;
    overflow-y: auto;
    padding: 18px 28px 28px;
  }

  .page :global(h1) {
    margin: 0;
    font-size: 26px;
    font-weight: 600;
    letter-spacing: 0.02em;
  }

  .page :global(.sous-titre) {
    margin: -10px 0 0;
    color: var(--texte-2);
  }

  .page :global(.discret) {
    color: var(--texte-2);
  }

  .etat {
    min-height: 1.5em;
    margin: auto 0 0;
    color: var(--texte-2);
  }

  .etat.erreur {
    color: var(--rouge);
  }
</style>
