<!--
  Les paramètres de l'appli elle-même (démarrage, installation, mises à jour, affichage), à part des réglages de son
  travail. Chaque changement est enregistré tout de suite. Pas de bouton orange ici, donc aucun panneau accentué.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { lireDemarrageAuto, messageErreur, reglerDemarrageAuto } from '../api';
  import type { Identite, Reglages } from '../api';
  import DialogueDesinstallation from '../dialogues/Desinstallation.svelte';
  import ASavoir from '../composants/ASavoir.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import CarteChoix from '../composants/CarteChoix.svelte';
  import Panneau from '../composants/Panneau.svelte';

  interface Props {
    identite: Identite;
    reglages: Reglages;
    enregistrer: () => void;
    signaler: (texte: string, erreur?: boolean) => void;
  }

  let { identite, reglages = $bindable(), enregistrer, signaler }: Props = $props();
  let demarrageAuto = $state(false);
  let dialogueDesinstallation = $state(false);

  async function relireDemarrageAuto() {
    if (!identite.enFond) return;
    try {
      demarrageAuto = await lireDemarrageAuto();
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  }

  async function changerDemarrageAuto() {
    try {
      demarrageAuto = await reglerDemarrageAuto(demarrageAuto);
      signaler(demarrageAuto ? 'Démarrage avec Windows activé.' : 'Démarrage avec Windows désactivé.');
    } catch (e) {
      signaler(messageErreur(e), true);
      await relireDemarrageAuto();
    }
  }

  /** La valeur d'un champ nombre au moment où il prend le focus. */
  let avant = 0;

  function retenir(evenement: Event) {
    avant = (evenement.currentTarget as HTMLInputElement).valueAsNumber;
  }

  /** Un nombre vidé (null pour Svelte) n'est pas envoyé : le champ reprend sa valeur d'avant. */
  function enregistrerNombre(evenement: Event, cle: 'minimum' | 'maximum') {
    const champ = evenement.currentTarget as HTMLInputElement;
    if (Number.isNaN(champ.valueAsNumber)) {
      reglages.fond[cle] = avant;
      return;
    }
    enregistrer();
  }

  onMount(() => {
    void relireDemarrageAuto();
    // « Démarrer avec Windows » se change aussi dans Windows et dans le menu de l'icône : relu au retour.
    const auRetour = () => void relireDemarrageAuto();
    window.addEventListener('focus', auRetour);
    return () => window.removeEventListener('focus', auRetour);
  });
</script>

<h1>Paramètres</h1>
<p class="sous-titre">{identite.nom} elle-même : démarrage, installation, mises à jour, affichage.</p>
<ASavoir>
  {identite.enFond
    ? 'Chaque changement est enregistré tout de suite : dans reglages.json, et dans Windows pour le démarrage.'
    : 'Chaque changement est enregistré tout de suite dans reglages.json, dans le dossier de données.'}
</ASavoir>

{#if identite.enFond}
  <Panneau titre="Démarrage">
    <label class="ligne">
      <input type="checkbox" bind:checked={demarrageAuto} onchange={changerDemarrageAuto} />
      Démarrer avec Windows (dans l'icône de notification, sans ouvrir la fenêtre)
    </label>
  </Panneau>
{/if}

<Panneau titre="Installation">
  {#if identite.installable}
    <label class="ligne">
      <input type="checkbox" bind:checked={reglages.proposerInstallation} onchange={enregistrer} />
      Proposer l'installation quand l'exe est lancé hors de son dossier
    </label>
  {/if}
  <div class="ligne desinstaller">
    <Bouton onclick={() => (dialogueDesinstallation = true)}>Désinstaller</Bouton>
    <span class="aide">
      Retire {identite.enFond ? 'le démarrage avec Windows, ' : ''}le raccourci du Bureau et le programme installé. Tes
      données restent.
    </span>
  </div>
</Panneau>

<Panneau titre="Mises à jour">
  <label class="ligne">
    <input type="checkbox" bind:checked={reglages.chercherMisesAJour} onchange={enregistrer} />
    Chercher une nouvelle version au lancement (une requête vers GitHub ; rien n'est installé sans ton accord)
  </label>
</Panneau>

<Panneau titre="Fond animé">
  <label class="ligne">
    <input type="checkbox" bind:checked={reglages.fond.actif} onchange={enregistrer} />
    Afficher le fond animé (jamais si Windows demande moins d'animations)
  </label>

  <p class="etiquette">Vitesse</p>
  <div class="cartes">
    <CarteChoix groupe="vitesse" valeur="lente" titre="Lente" bind:choisie={reglages.fond.vitesse} onchange={enregistrer}>
      Un passage toutes les 14 s
    </CarteChoix>
    <CarteChoix groupe="vitesse" valeur="normale" titre="Normale" bind:choisie={reglages.fond.vitesse} onchange={enregistrer}>
      Un passage toutes les 7 s
    </CarteChoix>
    <CarteChoix groupe="vitesse" valeur="rapide" titre="Rapide" bind:choisie={reglages.fond.vitesse} onchange={enregistrer}>
      Un passage toutes les 3,5 s
    </CarteChoix>
  </div>

  <p class="etiquette">Éléments par passage</p>
  <div class="ligne">
    <label>
      Au moins
      <input
        type="number"
        min="1"
        max="12"
        bind:value={reglages.fond.minimum}
        onfocus={retenir}
        onchange={(e) => enregistrerNombre(e, 'minimum')}
      />
    </label>
    <label>
      au plus
      <input
        type="number"
        min="1"
        max="12"
        bind:value={reglages.fond.maximum}
        onfocus={retenir}
        onchange={(e) => enregistrerNombre(e, 'maximum')}
      />
    </label>
  </div>
</Panneau>

{#if dialogueDesinstallation}
  <DialogueDesinstallation
    nom={identite.nom}
    donnees={identite.dossier}
    installable={identite.installable}
    fermer={() => {
      dialogueDesinstallation = false;
      // La désinstallation a pu retirer le démarrage avec Windows : la case suit.
      void relireDemarrageAuto();
    }}
  />
{/if}

<style>
  .ligne {
    display: flex;
    flex-wrap: wrap;
    align-items: center;
    gap: 10px 16px;
  }

  .desinstaller {
    margin-top: 14px;
  }

  .aide {
    color: var(--texte-2);
  }

  .etiquette {
    margin: 18px 0 8px;
    color: var(--texte-2);
    font-size: 11px;
    font-weight: 600;
    letter-spacing: 0.14em;
    text-transform: uppercase;
  }

  .cartes {
    display: grid;
    grid-template-columns: repeat(3, minmax(0, 1fr));
    gap: 12px;
  }

  input[type='number'] {
    width: 4.5em;
  }
</style>
