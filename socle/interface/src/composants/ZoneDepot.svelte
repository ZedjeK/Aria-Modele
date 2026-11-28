<!--
  Une zone où déposer des fichiers (module « Fichiers et dossiers »). Pendant le glisser, un cadre cyan pointillé dit
  qu'on peut lâcher ; au dépôt, `deposer` reçoit les chemins, déjà accordés aux commandes de l'appli. Une seule zone
  par page : Windows dépose sur toute la fenêtre.
-->
<script lang="ts">
  import { onMount, type Snippet } from 'svelte';
  import { getCurrentWebview } from '@tauri-apps/api/webview';
  import { ecouterFichiers } from '../api';

  interface Props {
    /** Ce que dit la zone pendant le glisser (« Dépose tes notes ici »). */
    texte: string;
    deposer: (chemins: string[]) => void;
    children?: Snippet;
  }

  let { texte, deposer, children }: Props = $props();
  let survol = $state(false);

  onMount(() => {
    const fins: Array<() => void> = [];
    void ecouterFichiers('fichiers-deposes', (chemins) => {
      survol = false;
      if (chemins.length > 0) deposer(chemins);
    }).then((fin) => fins.push(fin));
    void getCurrentWebview()
      .onDragDropEvent((e) => {
        if (e.payload.type === 'enter' || e.payload.type === 'over') survol = true;
        else if (e.payload.type === 'leave') survol = false;
      })
      .then((fin) => fins.push(fin));
    return () => fins.forEach((fin) => fin());
  });
</script>

<div class="zone" class:survol>
  {@render children?.()}
  {#if survol}
    <div class="voile" aria-live="polite">{texte}</div>
  {/if}
</div>

<style>
  .zone {
    position: relative;
  }

  .voile {
    position: absolute;
    inset: 0;
    display: grid;
    place-items: center;
    border: 2px dashed var(--cyan);
    background: color-mix(in srgb, var(--fond) 82%, transparent);
    color: var(--cyan);
    font-weight: 600;
    pointer-events: none;
  }
</style>
