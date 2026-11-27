<!--
  La désinstallation, posée sur la page des Paramètres : ce qui sera retiré, puis le bilan étape par étape. Les
  données restent, et le dialogue dit où elles sont.
-->
<script lang="ts">
  import { desinstaller, fermerApresDesinstallation, messageErreur } from '../api';
  import type { BilanDesinstallation } from '../api';
  import ASavoir from '../composants/ASavoir.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Dialogue from '../composants/Dialogue.svelte';
  import Panneau from '../composants/Panneau.svelte';

  interface Props {
    nom: string;
    /** Le dossier de données, gardé. */
    donnees: string;
    /** L'appli peut s'être installée dans programme\. */
    installable: boolean;
    fermer: () => void;
  }

  let { nom, donnees, installable, fermer }: Props = $props();
  let enCours = $state(false);
  let bilan = $state<BilanDesinstallation | null>(null);
  let etat = $state('');
  let erreur = $state(false);

  function signaler(texte: string, estErreur = false) {
    etat = texte;
    erreur = estErreur;
  }

  async function lancer() {
    enCours = true;
    signaler('Désinstallation en cours…');
    try {
      const fait = await desinstaller();
      bilan = fait;
      const ratees = fait.etapes.filter((e) => !e.reussie).length;
      if (ratees > 0) signaler('Une partie reste à faire à la main : le détail est ci-dessus.', true);
      else if (fait.aLaFermeture) signaler('programme\\ sera supprimé quand l\'appli se fermera.');
      else signaler('Désinstallation faite.');
    } catch (e) {
      signaler(messageErreur(e), true);
    } finally {
      enCours = false;
    }
  }

  async function fermerAppli() {
    enCours = true;
    try {
      await fermerApresDesinstallation();
    } catch (e) {
      signaler(messageErreur(e), true);
      enCours = false;
    }
  }
</script>

<Dialogue titre={`Désinstaller ${nom}`} fermer={() => !enCours && fermer()}>
  {#if bilan === null}
    <ASavoir>
      Tes données restent dans <span class="code">{donnees}</span> : réglages, journal, sauvegardes et secrets. Les
      supprimer est à toi, une fois l'appli fermée.
    </ASavoir>

    <Panneau titre="Ce qui sera retiré" accentue>
      <ul>
        <li>Le démarrage avec Windows.</li>
        <li>Le raccourci du Bureau, s'il pointe vers cette appli.</li>
        {#if installable}
          <li>
            Le programme installé (<span class="code">programme\</span>), s'il existe : une fois l'appli fermée si
            elle tourne depuis là.
          </li>
        {/if}
      </ul>
    </Panneau>
  {:else}
    <ASavoir>
      Tes données sont toujours dans <span class="code">{bilan.donnees}</span>. Pour tout effacer, supprime ce dossier
      une fois l'appli fermée.
    </ASavoir>

    <Panneau titre="Bilan" accentue={bilan.aLaFermeture}>
      <ul>
        {#each bilan.etapes as e (e.etape)}
          <li class:ratee={!e.reussie}>
            <strong>{e.etape}</strong> : {e.bilan}
          </li>
        {/each}
      </ul>
    </Panneau>
  {/if}

  {#snippet pied()}
    <p class="etat" class:erreur role="status">{etat}</p>
    {#if bilan === null}
      <Bouton genre="discret" desactive={enCours} onclick={fermer}>Annuler</Bouton>
      <Bouton genre="principal" desactive={enCours} onclick={lancer}>Désinstaller</Bouton>
    {:else if bilan.aLaFermeture}
      <Bouton genre="discret" desactive={enCours} onclick={fermer}>Plus tard</Bouton>
      <Bouton genre="principal" desactive={enCours} onclick={fermerAppli}>Fermer l'appli</Bouton>
    {:else}
      <Bouton genre="discret" onclick={fermer}>Fermer</Bouton>
    {/if}
  {/snippet}
</Dialogue>

<style>
  ul {
    margin: 0;
    padding-left: 1.2em;
  }

  li + li {
    margin-top: 6px;
  }

  li.ratee {
    color: var(--rouge);
  }

  .code {
    font-family: var(--police-code);
    overflow-wrap: anywhere;
  }

  .etat {
    flex: 1;
    min-height: 1.5em;
    color: var(--texte-2);
  }

  .etat.erreur {
    color: var(--rouge);
  }
</style>
