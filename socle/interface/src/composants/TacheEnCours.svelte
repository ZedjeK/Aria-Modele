<!--
  Une tâche longue dans le panneau qui l'a lancée : son nom, ce qui se fait, une barre de progression (indéterminée
  tant que la durée est inconnue) et « Annuler » en contour discret. Finie, elle montre son bilan.
-->
<script lang="ts">
  import { annulerTache, messageErreur, type Avancement } from '../api';
  import Bouton from './Bouton.svelte';

  interface Props {
    tache: Avancement;
    signaler?: ((texte: string, erreur?: boolean) => void) | undefined;
  }

  let { tache, signaler }: Props = $props();
  let demandee = $state(false);

  async function annuler() {
    demandee = true;
    try {
      await annulerTache(tache.id);
    } catch (e) {
      signaler?.(messageErreur(e), true);
    }
  }

  const pourcent = $derived(tache.progression === null ? null : Math.round(tache.progression * 100));
</script>

<div class="tache" class:echec={tache.etape === 'echec'} role="status" aria-live="polite">
  <div class="tete">
    <span class="libelle">{tache.libelle}</span>
    <span class="detail">
      {#if tache.etape === 'en-cours'}
        {demandee ? 'Annulation demandée…' : tache.detail}{pourcent === null ? '' : ` · ${pourcent} %`}
      {:else if tache.etape === 'annulee'}
        Annulée
      {:else}
        {tache.detail}
      {/if}
    </span>
  </div>
  <div
    class="barre"
    class:indeterminee={tache.etape === 'en-cours' && pourcent === null}
    role="progressbar"
    aria-label={tache.libelle}
    aria-valuemin="0"
    aria-valuemax="100"
    aria-valuenow={pourcent ?? undefined}
  >
    <i style:width={pourcent === null ? undefined : `${pourcent}%`}></i>
  </div>
  {#if tache.etape === 'en-cours'}
    <div class="actions">
      <Bouton genre="discret" petit desactive={demandee} onclick={() => void annuler()}>Annuler</Bouton>
    </div>
  {/if}
</div>

<style>
  .tache {
    display: flex;
    flex-direction: column;
    gap: 8px;
    margin-top: 12px;
  }

  .tete {
    display: flex;
    justify-content: space-between;
    gap: 12px;
  }

  .libelle {
    color: var(--cyan);
    font-weight: 600;
  }

  .detail {
    color: var(--texte-2);
  }

  .echec .detail {
    color: var(--rouge);
  }

  .barre {
    position: relative;
    height: 6px;
    overflow: hidden;
    background: var(--ligne);
  }

  .barre i {
    position: absolute;
    inset: 0 auto 0 0;
    background: var(--cyan);
  }

  .indeterminee i {
    width: 30%;
    animation: glisse 1.4s linear infinite;
  }

  @keyframes glisse {
    from {
      left: -30%;
    }
    to {
      left: 100%;
    }
  }

  .actions {
    display: flex;
    justify-content: flex-end;
  }
</style>
