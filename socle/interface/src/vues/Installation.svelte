<!--
  La proposition d'installation, montrée au lancement d'un exe hors de son dossier programme\. Fenêtre de dialogue :
  titre en cyan, « À savoir », le panneau sur lequel agit le bouton orange (accentué), puis la ligne d'état et les
  boutons, l'orange en dernier.
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import { infosInstallation, installer, messageErreur, refuserInstallation } from '../api';
  import type { InfosInstallation } from '../api';
  import ASavoir from '../composants/ASavoir.svelte';
  import Bouton from '../composants/Bouton.svelte';
  import Panneau from '../composants/Panneau.svelte';

  let infos = $state<InfosInstallation | null>(null);
  let raccourci = $state(false);
  let enCours = $state(false);
  let etat = $state('');
  let erreur = $state(false);

  onMount(async () => {
    try {
      infos = await infosInstallation();
    } catch (e) {
      etat = messageErreur(e);
      erreur = true;
    }
  });

  async function agir(action: () => Promise<void>, pendant: string) {
    enCours = true;
    etat = pendant;
    erreur = false;
    try {
      await action();
    } catch (e) {
      etat = messageErreur(e);
      erreur = true;
      enCours = false;
    }
  }
</script>

<div class="dialogue">
  <h1>{infos ? `Installer ${infos.nom}` : 'Installer'}</h1>
  <ASavoir>
    L'appli s'installe dans ton dossier de données avant de s'ouvrir, et se lance ensuite de là. Rien n'est installé
    ailleurs, et rien ne change dans Windows. Annuler ferme l'appli sans rien installer.
  </ASavoir>

  <Panneau titre="Dossier d'installation" accentue>
    {#if infos}
      <p class="code">{infos.dossier}</p>
      {#if infos.versionInstallee}
        <p class="discret">
          La version {infos.versionInstallee} déjà installée sera remplacée par la {infos.version}, et gardée dans les
          sauvegardes.
        </p>
      {/if}
    {/if}
  </Panneau>

  <div class="options">
    <label class="ligne">
      <input
        type="checkbox"
        bind:checked={raccourci}
        disabled={!infos?.raccourci || infos.raccourciExistant === 'ailleurs'}
      />
      {infos?.raccourciExistant === 'a-nous'
        ? 'Mettre à jour le raccourci du Bureau'
        : 'Créer un raccourci sur le Bureau'}
    </label>
    {#if infos?.raccourci}<p class="aide code">{infos.raccourci}</p>{/if}
    {#if infos?.raccourciExistant === 'ailleurs'}
      <p class="aide">Un raccourci du même nom pointe vers un autre programme : il reste tel quel.</p>
    {/if}
  </div>

  <p class="etat" class:erreur role="status">{etat}</p>

  <div class="boutons">
    <Bouton genre="discret" desactive={enCours} onclick={() => agir(refuserInstallation, 'Fermeture…')}>Annuler</Bouton>
    <Bouton
      genre="principal"
      desactive={enCours || !infos}
      onclick={() => agir(() => installer(raccourci), 'Installation…')}
    >
      Installer
    </Bouton>
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
    margin-top: 8px;
    color: var(--texte-2);
  }

  .options {
    display: flex;
    flex-direction: column;
    gap: 4px;
  }

  .ligne {
    display: flex;
    align-items: center;
    gap: 10px;
    margin-top: 8px;
  }

  .aide {
    margin-left: 28px;
    color: var(--texte-2);
    font-size: 12px;
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
