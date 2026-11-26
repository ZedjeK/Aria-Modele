<!--
  Le journal de l'appli : ses dernières opérations, la plus récente en haut. Lecture seule. « Exporter le diagnostic »
  copie dans le presse-papiers le texte à coller dans un signalement de bug ; si le presse-papiers refuse, le texte
  s'affiche dans un dialogue, à copier à la main.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { diagnostic, lireJournal, messageErreur, type Operation } from '../lib/api';
  import { dateLisible } from '../lib/journal';
  import ASavoir from '../lib/composants/ASavoir.svelte';
  import Bouton from '../lib/composants/Bouton.svelte';
  import Dialogue from '../lib/composants/Dialogue.svelte';
  import Panneau from '../lib/composants/Panneau.svelte';

  interface Props {
    signaler: (texte: string, erreur?: boolean) => void;
  }

  let { signaler }: Props = $props();
  let operations = $state<Operation[] | null>(null);
  let texteDiagnostic = $state<string | null>(null);

  async function relire() {
    try {
      operations = await lireJournal();
    } catch (e) {
      signaler(messageErreur(e), true);
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

{#if texteDiagnostic !== null}
  <Dialogue titre="Diagnostic" fermer={() => (texteDiagnostic = null)}>
    <ASavoir>
      Le presse-papiers n'a pas pu être utilisé : clique dans le texte, sélectionne tout (Ctrl+A), copie-le (Ctrl+C) et
      colle-le dans ton signalement. Rien n'a été envoyé ; aucun secret n'y figure.
    </ASavoir>
    <textarea class="diagnostic" readonly rows="16" aria-label="Texte du diagnostic" value={texteDiagnostic}></textarea>
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
