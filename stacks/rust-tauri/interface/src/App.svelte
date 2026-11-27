<!--
  La fenêtre d'Aria {{NOM}} : le socle (Fenetre) apporte le menu latéral, le décor, la barre de titre, le bandeau
  de mise à jour, le Journal et les Paramètres ; l'appli donne son logo, ses familles de pages, ses pastilles d'état
  et ses pages.
-->
<script lang="ts">
  import { Fenetre, type Famille, type Pastille } from 'aria-interface';
  import logo from './logo.svg';
  import Accueil from './pages/Accueil.svelte';

  // Les familles des pages du travail de l'appli. À adapter.
  const familles: Famille[] = [{ titre: 'Travail', pages: [{ id: 'accueil', titre: 'Accueil' }] }];

  /** Part libre du disque, mesurée par la page d'accueil ; null tant que rien n'est mesuré. */
  let fraction = $state<number | null>(null);
  let mesure = $state('');

  const pastilles: Pastille[] = $derived([
    {
      nom: 'Disque',
      detail: fraction === null ? 'à tester' : `${Math.round(fraction * 100)} % libre`,
      ton: fraction === null ? 'repos' : fraction < 0.1 ? 'attention' : 'actif',
    },
  ]);
</script>

<Fenetre {logo} {familles} {pastilles}>
  {#snippet contenu(page, outils)}
    {#if page === 'accueil'}
      <Accueil identite={outils.identite} bind:fraction bind:mesure signaler={outils.signaler} />
    {/if}
  {/snippet}
</Fenetre>
