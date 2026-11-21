<!--
  La fenêtre de mise à jour. Fenêtre de dialogue : titre en cyan, « À savoir », le panneau sur lequel agit le bouton
  orange (accentué), puis la ligne d'état et les boutons, l'orange en dernier.
-->
<script lang="ts">
  import { getCurrentWindow } from '@tauri-apps/api/window';
  import { onMount } from 'svelte';
  import { installerMiseAJour, messageErreur, miseAJourDisponible, voirNotes } from './lib/api';
  import type { InfosMiseAJour } from './lib/api';
  import { octetsLisibles } from './lib/jauge';
  import ASavoir from './lib/composants/ASavoir.svelte';
  import Bouton from './lib/composants/Bouton.svelte';
  import Panneau from './lib/composants/Panneau.svelte';

  let infos = $state<InfosMiseAJour | null>(null);
  let enCours = $state(false);
  let etat = $state('');
  let erreur = $state(false);

  function signaler(texte: string, estErreur = false) {
    etat = texte;
    erreur = estErreur;
  }

  onMount(async () => {
    try {
      infos = await miseAJourDisponible();
      if (!infos) signaler('Aucune mise à jour connue.', true);
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  });

  async function installer() {
    enCours = true;
    signaler('Téléchargement et vérification…');
    try {
      await installerMiseAJour();
    } catch (e) {
      signaler(messageErreur(e), true);
      enCours = false;
    }
  }

  async function notes() {
    try {
      await voirNotes();
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  }
</script>

<div class="dialogue">
  <h1>{infos ? `Mettre à jour ${infos.nom}` : 'Mise à jour'}</h1>
  <ASavoir>
    La nouvelle version est téléchargée depuis la release GitHub de l'appli, vérifiée (taille et empreinte SHA-256),
    puis installée là où l'appli tourne. La version en cours est gardée dans les sauvegardes, et l'appli se relance.
  </ASavoir>

  <Panneau titre="Nouvelle version" accentue>
    {#if infos}
      <p><strong class="code">{infos.version}</strong> <span class="discret">(en cours : {infos.actuelle})</span></p>
      <p class="discret">Téléchargement : {octetsLisibles(infos.taille)}</p>
      <div class="lien">
        <Bouton genre="discret" onclick={notes}>Voir les notes de version</Bouton>
      </div>
    {/if}
  </Panneau>

  <p class="etat" class:erreur role="status">{etat}</p>

  <div class="boutons">
    <Bouton genre="discret" desactive={enCours} onclick={() => void getCurrentWindow().close()}>Plus tard</Bouton>
    <Bouton genre="principal" desactive={enCours || !infos} onclick={installer}>Installer la mise à jour</Bouton>
  </div>
</div>

<style>
  .dialogue {
    display: flex;
    flex-direction: column;
    gap: 16px;
    height: 100%;
    padding: 24px;
    overflow-y: auto;
  }

  h1 {
    margin: 0;
    color: var(--cyan);
    font-size: 20px;
    font-weight: 600;
  }

  p {
    margin: 0;
  }

  .discret {
    margin-top: 6px;
    color: var(--texte-2);
  }

  .lien {
    margin-top: 12px;
  }

  .etat {
    min-height: 1.5em;
    color: var(--texte-2);
  }

  .etat.erreur {
    color: var(--rouge);
  }

  .boutons {
    display: flex;
    justify-content: flex-end;
    gap: 12px;
    margin-top: auto;
  }
</style>
