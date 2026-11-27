<!--
  Le journal de l'appli : ses dernières opérations, la plus récente en haut. Lecture seule. « Exporter le diagnostic »
  copie dans le presse-papiers le texte à coller dans un signalement de bug ; si le presse-papiers refuse, le texte
  s'affiche dans un dialogue, à copier à la main.
-->
<script lang="ts">
  import { onMount, tick } from 'svelte';
  import {
    diagnostic,
    lireJournal,
    listerSauvegardes,
    messageErreur,
    restaurerSauvegarde,
    type Operation,
    type Sauvegarde,
  } from '../api';
  import { octetsLisibles } from '../jauge';
  import { dateLisible } from '../journal';
  import ASavoir from '../composants/ASavoir.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Dialogue from '../composants/Dialogue.svelte';
  import Panneau from '../composants/Panneau.svelte';

  interface Props {
    signaler: (texte: string, erreur?: boolean) => void;
  }

  let { signaler }: Props = $props();
  let operations = $state<Operation[] | null>(null);
  let sauvegardes = $state<Sauvegarde[] | null>(null);
  let aRestaurer = $state<Sauvegarde | null>(null);
  let restauration = $state(false);
  let texteDiagnostic = $state<string | null>(null);
  let zoneDiagnostic = $state<HTMLTextAreaElement | null>(null);

  // Le texte est prêt à copier dès l'ouverture : focus et tout sélectionné, après le focus que pose le dialogue.
  $effect(() => {
    const zone = zoneDiagnostic;
    if (zone === null) return;
    void tick().then(() => {
      zone.focus();
      zone.select();
    });
  });

  async function relire() {
    try {
      [operations, sauvegardes] = await Promise.all([lireJournal(), listerSauvegardes()]);
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  }

  async function restaurer(sauvegarde: Sauvegarde) {
    restauration = true;
    try {
      const fichier = await restaurerSauvegarde(sauvegarde.nom);
      signaler(`${fichier} restauré ; son état d'avant est gardé dans les sauvegardes.`);
      aRestaurer = null;
      await relire();
    } catch (e) {
      signaler(messageErreur(e), true);
    } finally {
      restauration = false;
    }
  }

  async function exporterDiagnostic() {
    let texte: string;
    try {
      texte = await diagnostic();
    } catch (e) {
      signaler(messageErreur(e), true);
      return;
    }
    try {
      await navigator.clipboard.writeText(texte);
      signaler('Diagnostic copié : relis-le, puis colle-le dans ton signalement. Rien n\'a été envoyé.');
    } catch {
      // Presse-papiers refusé : le texte s'affiche, à copier à la main.
      texteDiagnostic = texte;
    }
  }

  onMount(() => void relire());
</script>

<h1>Journal</h1>
<p class="sous-titre">Ce que l'appli a fait, dans l'ordre.</p>
<ASavoir>Les 500 dernières opérations sont gardées dans journal.json, dans le dossier de données.</ASavoir>

<Panneau titre="Dernières opérations">
  {#if operations === null}
    <p class="discret">Lecture…</p>
  {:else if operations.length === 0}
    <p class="discret">Rien pour l'instant.</p>
  {:else}
    <table>
      <thead><tr><th>Quand</th><th>Quoi</th><th>Bilan</th></tr></thead>
      <tbody>
        {#each operations as operation, i (i)}
          <tr>
            <td class="code">{dateLisible(operation.date)}</td>
            <td>{operation.type}</td>
            <td>{operation.bilan}</td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
  <div class="actions">
    <Bouton onclick={() => void relire()}>Relire</Bouton>
    <Bouton onclick={() => void exporterDiagnostic()}>Exporter le diagnostic</Bouton>
  </div>
</Panneau>

<Panneau titre="Sauvegardes">
  <p class="discret">
    Une copie datée de chaque fichier, faite avant de le modifier. Restaurer sauvegarde d'abord l'état actuel : rien
    n'est perdu.
  </p>
  {#if sauvegardes === null}
    <p class="discret">Lecture…</p>
  {:else if sauvegardes.length === 0}
    <p class="discret">Aucune sauvegarde pour l'instant.</p>
  {:else}
    <table>
      <thead><tr><th>Copie</th><th>Fichier d'origine</th><th>Taille</th><th></th></tr></thead>
      <tbody>
        {#each sauvegardes as sauvegarde (sauvegarde.nom)}
          <tr>
            <td class="code">{sauvegarde.nom}</td>
            <td class="code">{sauvegarde.origine ?? 'inconnu : à restaurer à la main'}</td>
            <td>{octetsLisibles(sauvegarde.taille)}</td>
            <td>
              {#if sauvegarde.origine}
                <Bouton petit onclick={() => (aRestaurer = sauvegarde)}>Restaurer</Bouton>
              {/if}
            </td>
          </tr>
        {/each}
      </tbody>
    </table>
  {/if}
</Panneau>

{#if aRestaurer}
  <Dialogue titre="Restaurer une sauvegarde" fermer={() => !restauration && (aRestaurer = null)}>
    <ASavoir>L'état actuel du fichier est d'abord sauvegardé : tu pourras revenir en arrière.</ASavoir>
    <Panneau titre="Ce qui sera remis" accentue>
      <p class="code">{aRestaurer.origine}</p>
      <p class="discret">depuis la copie <span class="code">{aRestaurer.nom}</span></p>
    </Panneau>
    {#snippet pied()}
      <Bouton genre="discret" desactive={restauration} onclick={() => (aRestaurer = null)}>Annuler</Bouton>
      <Bouton genre="principal" desactive={restauration} onclick={() => aRestaurer && void restaurer(aRestaurer)}>
        Restaurer
      </Bouton>
    {/snippet}
  </Dialogue>
{/if}

{#if texteDiagnostic !== null}
  <Dialogue titre="Diagnostic" fermer={() => (texteDiagnostic = null)}>
    <ASavoir>
      Le presse-papiers n'a pas pu être utilisé : le texte est déjà sélectionné, copie-le (Ctrl+C) et colle-le dans ton
      signalement. Rien n'a été envoyé ; aucun secret n'y figure.
    </ASavoir>
    <textarea
      class="diagnostic"
      readonly
      rows="16"
      aria-label="Texte du diagnostic"
      value={texteDiagnostic}
      bind:this={zoneDiagnostic}
    ></textarea>
    {#snippet pied()}
      <Bouton genre="discret" onclick={() => (texteDiagnostic = null)}>Fermer</Bouton>
    {/snippet}
  </Dialogue>
{/if}

<style>
  table {
    width: 100%;
    border-collapse: collapse;
    font-size: 13px;
  }

  th {
    padding: 6px 10px 6px 0;
    border-bottom: 1px solid var(--ligne);
    color: var(--texte-2);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.12em;
    text-align: left;
    text-transform: uppercase;
  }

  td {
    padding: 6px 10px 6px 0;
    border-bottom: 1px solid color-mix(in srgb, var(--ligne) 50%, transparent);
    vertical-align: top;
  }

  td.code {
    color: var(--texte-2);
    white-space: nowrap;
  }

  .actions {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: 16px;
  }

  .diagnostic {
    box-sizing: border-box;
    width: 100%;
    margin-top: 12px;
    padding: 10px;
    border: 1px solid var(--ligne);
    background: var(--fond);
    color: var(--texte);
    font: 12px var(--police-code);
    resize: vertical;
  }
</style>
