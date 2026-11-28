<!--
  La désinstallation, posée sur la page des Paramètres : ce qui sera retiré, puis le bilan étape par étape. Les
  données restent, et le dialogue dit où elles sont, sauf si l'utilisateur coche « Supprimer aussi mes données ».
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
    /** Le dossier de données. */
    donnees: string;
    /** L'appli sert en arrière-plan : elle a pu s'inscrire au démarrage de Windows. */
    enFond: boolean;
    /** Le module « Fichiers et dossiers » est activé : l'appli a pu s'associer à des types de fichiers. */
    associations?: boolean;
    fermer: () => void;
  }

  let { nom, donnees, enFond, associations = false, fermer }: Props = $props();
  let enCours = $state(false);
  let bilan = $state<BilanDesinstallation | null>(null);
  let etat = $state('');
  let erreur = $state(false);
  /** Décochée d'office : effacer les données ne se défait pas. */
  let effacerDonnees = $state(false);

  function signaler(texte: string, estErreur = false) {
    etat = texte;
    erreur = estErreur;
  }

  async function lancer() {
    enCours = true;
    signaler('Désinstallation en cours…');
    try {
      const fait = await desinstaller(effacerDonnees);
      bilan = fait;
      const ratees = fait.etapes.filter((e) => !e.reussie).length;
      if (ratees > 0) signaler('Une partie reste à faire à la main : le détail est ci-dessus.', true);
      else if (fait.donneesEffacees) signaler('Tes données seront effacées quand l\'appli se fermera.');
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
      Tes données sont dans <span class="code">{donnees}</span> : réglages, journal, sauvegardes et secrets. Elles
      restent, sauf si tu coches la case ci-dessous.
    </ASavoir>

    <Panneau titre="Ce qui sera retiré" accentue>
      <ul>
        {#if enFond}<li>Le démarrage avec Windows.</li>{/if}
        {#if associations}<li>Les types de fichiers associés à l'appli.</li>{/if}
        <li>Le raccourci du Bureau, s'il pointe vers cette appli.</li>
        <li>
          Le programme installé (<span class="code">programme\</span>) : une fois l'appli fermée si elle tourne depuis
          là.
        </li>
        {#if effacerDonnees}
          <li class="alerte">Tout le dossier de données, une fois l'appli fermée.</li>
        {/if}
      </ul>
    </Panneau>

    <Panneau titre="Données">
      <label class="ligne">
        <input type="checkbox" bind:checked={effacerDonnees} disabled={enCours} />
        Supprimer aussi mes données
      </label>
      <p class="aide">
        Réglages, journal, sauvegardes et secrets sont effacés une fois l'appli fermée, et ne se récupèrent pas. Les
        autres applis Aria et leur dossier commun <span class="code">.Aria</span> ne sont pas touchés.
      </p>
    </Panneau>
  {:else}
    <ASavoir>
      {#if bilan.donneesEffacees}
        Tes données (<span class="code">{bilan.donnees}</span>) seront effacées une fois l'appli fermée. Si le dossier
        est encore là ensuite, supprime-le à la main.
      {:else}
        Tes données sont toujours dans <span class="code">{bilan.donnees}</span>. Pour tout effacer, supprime ce dossier
        une fois l'appli fermée.
      {/if}
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

  .ligne {
    display: flex;
    align-items: center;
    gap: 10px;
  }

  li.alerte {
    color: var(--orange);
  }

  .aide {
    margin: 6px 0 0;
    color: var(--texte-2);
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
