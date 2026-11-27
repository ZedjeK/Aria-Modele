<!--
  L'assistant de premier lancement : il présente l'appli et règle l'essentiel, puis ne s'ouvre plus de lui-même ; il
  se rouvre depuis les Paramètres. Les étapes en encoches non cliquables, cochées une fois faites ; « À savoir » pour
  le commentaire de l'étape ; un panneau accentué par étape ; le bouton orange sur « Suivant », puis « Terminer ».
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    bilanReprise,
    lireDemarrageAuto,
    messageErreur,
    reglerDemarrageAuto,
    type BilanReprise,
    type Identite,
    type Reglages,
  } from '../api';
  import ASavoir from '../composants/ASavoir.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Dialogue from '../composants/Dialogue.svelte';
  import Panneau from '../composants/Panneau.svelte';

  interface Props {
    identite: Identite;
    reglages: Reglages;
    enregistrer: () => Promise<void>;
    fermer: () => void;
    signaler: (texte: string, erreur?: boolean) => void;
  }

  let { identite, reglages = $bindable(), enregistrer, fermer, signaler }: Props = $props();

  let reprise = $state<BilanReprise | null>(null);
  let demarrageAuto = $state(false);
  let rang = $state(0);

  type Etape = { id: string; titre: string; commentaire: string };
  const etapes: Etape[] = $derived([
    {
      id: 'bienvenue',
      titre: 'Bienvenue',
      commentaire: identite.promesse,
    },
    {
      id: 'reglages',
      titre: 'Réglages',
      commentaire: 'Tout se change ensuite dans les Paramètres. Rien ne part de ton PC sans que tu le saches.',
    },
    ...(reprise?.copie
      ? [
          {
            id: 'reprise',
            titre: 'Tes données',
            commentaire:
              "L'appli a repris les données de son ancienne version depuis une copie : l'ancien dossier n'a pas été modifié.",
          },
        ]
      : []),
    {
      id: 'pret',
      titre: 'Prêt',
      commentaire: 'Cet assistant se rouvre depuis les Paramètres.',
    },
  ]);
  const etape = $derived(etapes[Math.min(rang, etapes.length - 1)]);
  const derniere = $derived(rang >= etapes.length - 1);

  onMount(() => {
    void (async () => {
      try {
        reprise = await bilanReprise();
        if (identite.enFond) demarrageAuto = await lireDemarrageAuto();
      } catch (e) {
        signaler(messageErreur(e), true);
      }
    })();
  });

  async function changerDemarrageAuto() {
    try {
      demarrageAuto = await reglerDemarrageAuto(demarrageAuto);
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  }

  async function terminer() {
    reglages.assistantVu = true;
    await enregistrer();
    fermer();
  }
</script>

<Dialogue titre={`Premiers pas avec ${identite.nom}`} fermer={() => void terminer()}>
  <ol class="etapes" aria-label="Étapes">
    {#each etapes as e, i (e.id)}
      <li class="encoche" class:active={i === rang} class:faite={i < rang} aria-current={i === rang ? 'step' : undefined}>
        <span>{#if i < rang}<b aria-hidden="true">✓</b>{/if}{e.titre}</span>
      </li>
    {/each}
  </ol>

  {#if etape}<ASavoir>{etape.commentaire}</ASavoir>{/if}

  {#if etape?.id === 'bienvenue'}
    <Panneau titre="Où vit l'appli" accentue>
      <p>Tout ce qu'elle garde (réglages, journal, sauvegardes) est dans ton dossier de données, et nulle part ailleurs :</p>
      <p class="code discret">{identite.dossier}</p>
      <p>La page « Journal » dit ce qu'elle a fait ; « Paramètres » règle l'appli elle-même.</p>
    </Panneau>
  {:else if etape?.id === 'reglages'}
    <Panneau titre="L'essentiel" accentue>
      <label class="ligne">
        <input type="checkbox" bind:checked={reglages.chercherMisesAJour} onchange={() => void enregistrer()} />
        Chercher une nouvelle version au lancement (une requête vers GitHub ; rien n'est installé sans ton accord)
      </label>
      {#if identite.enFond}
        <label class="ligne">
          <input type="checkbox" bind:checked={demarrageAuto} onchange={() => void changerDemarrageAuto()} />
          Démarrer avec Windows (dans l'icône de notification)
        </label>
      {/if}
      <label class="ligne">
        <input type="checkbox" bind:checked={reglages.fond.actif} onchange={() => void enregistrer()} />
        Afficher le fond animé
      </label>
    </Panneau>
  {:else if etape?.id === 'reprise' && reprise}
    <Panneau titre="Ce qui a été repris" accentue>
      {#if reprise.repris.length > 0}
        <ul>{#each reprise.repris as ligne, i (i)}<li>{ligne}</li>{/each}</ul>
      {:else}
        <p class="discret">Rien.</p>
      {/if}
      {#if reprise.nonRepris.length > 0}
        <p class="etiquette">Non repris</p>
        <ul class="non-repris">{#each reprise.nonRepris as ligne, i (i)}<li>{ligne}</li>{/each}</ul>
      {/if}
      <p class="discret">La copie de l'ancien dossier est gardée : <span class="code">{reprise.copie}</span></p>
    </Panneau>
  {:else}
    <Panneau titre="C'est prêt" accentue>
      <p>{identite.nom} est installée et réglée. Bon travail !</p>
    </Panneau>
  {/if}

  {#snippet pied()}
    <Bouton genre="discret" onclick={() => void terminer()}>Passer</Bouton>
    {#if derniere}
      <Bouton genre="principal" onclick={() => void terminer()}>Terminer</Bouton>
    {:else}
      <Bouton genre="principal" onclick={() => (rang += 1)}>Suivant</Bouton>
    {/if}
  {/snippet}
</Dialogue>

<style>
  .etapes {
    display: flex;
    gap: 6px;
    margin: 0 0 4px;
    padding: 0;
    list-style: none;
    border-bottom: 1px solid var(--ligne);
  }

  .encoche {
    --encoche: polygon(0 0, calc(100% - 10px) 0, 100% 10px, 100% 100%, 0 100%);
    padding: 1px;
    clip-path: var(--encoche);
    background: var(--ligne);
    color: var(--texte-2);
    font: 700 12px var(--police);
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .encoche span {
    display: block;
    padding: 8px 20px 10px 14px;
    clip-path: var(--encoche);
    background: var(--panneau);
  }

  .encoche b {
    margin-right: 6px;
    color: var(--cyan);
  }

  .active {
    background: var(--cyan);
    color: var(--cyan);
  }

  .faite {
    color: var(--texte);
  }

  .ligne {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
  }

  p {
    margin: 0 0 8px;
  }

  .etiquette {
    margin-top: 12px;
    color: var(--texte-2);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .non-repris {
    color: var(--ambre);
  }
</style>
