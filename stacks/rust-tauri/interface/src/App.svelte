<!--
  La fenêtre d'Aria {{NOM}} : menu latéral à gauche, zone principale à droite avec ses sous-onglets.
  Page d'exemple à remplacer par celle de l'appli, en gardant les règles de l'identité Aria (aria-template-app).
-->
<script lang="ts">
  import { onMount } from 'svelte';
  import {
    ecouter,
    enregistrerReglages,
    identite as lireIdentite,
    lireDemarrageAuto,
    lireReglages,
    mesurerDisque,
    messageErreur,
    reglerDemarrageAuto,
    type Identite,
    type Reglages,
  } from './lib/api';
  import { octetsLisibles } from './lib/jauge';
  import ASavoir from './lib/composants/ASavoir.svelte';
  import Bouton from './lib/composants/Bouton.svelte';
  import CarteChoix from './lib/composants/CarteChoix.svelte';
  import FondAnime from './lib/composants/FondAnime.svelte';
  import Jauge from './lib/composants/Jauge.svelte';
  import Panneau from './lib/composants/Panneau.svelte';
  import SousOnglets from './lib/composants/SousOnglets.svelte';

  const onglets = [
    { id: 'apercu', titre: 'Aperçu' },
    { id: 'reglages', titre: 'Réglages' },
  ];

  let identite = $state<Identite | null>(null);
  const commentaires: Record<string, string> = $derived({
    apercu: "Exemple à remplacer : une action de l'appli et ce qu'elle produit. Rien n'est modifié sans ton accord.",
    reglages: identite?.enFond
      ? 'Chaque changement est enregistré tout de suite : dans reglages.json, et dans Windows pour le démarrage.'
      : 'Chaque changement est enregistré tout de suite dans reglages.json, dans le dossier de données.',
  });
  let reglages = $state<Reglages | null>(null);
  let onglet = $state('apercu');
  let fraction = $state<number | null>(null);
  let mesure = $state('');
  let mesureEnCours = $state(false);
  let etat = $state('');
  let erreur = $state(false);
  let demarrageAuto = $state(false);

  const nomCourt = $derived(identite?.nom.replace(/^Aria\s+/, '') ?? '');

  function signaler(texte: string, estErreur = false) {
    etat = texte;
    erreur = estErreur;
  }

  onMount(() => {
    void (async () => {
      try {
        [identite, reglages] = await Promise.all([lireIdentite(), lireReglages()]);
        await relireDemarrageAuto();
      } catch (e) {
        signaler(`Démarrage incomplet : ${messageErreur(e)}`, true);
      }
    })();
    // « Démarrer avec Windows » se change aussi dans Windows et dans le menu de l'icône : relu au retour.
    const auRetour = () => void relireDemarrageAuto();
    window.addEventListener('focus', auRetour);
    // « Réglages » dans le menu de l'icône.
    const arret = ecouter<string>('aller-a', (page) => {
      if (page === 'reglages') onglet = 'reglages';
    });
    return () => {
      window.removeEventListener('focus', auRetour);
      void arret.then((f) => f());
    };
  });

  async function relireDemarrageAuto() {
    if (identite?.enFond) demarrageAuto = await lireDemarrageAuto();
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

  async function mesurer() {
    mesureEnCours = true;
    try {
      const espace = await mesurerDisque();
      fraction = espace.total > 0 ? espace.libre / espace.total : 0;
      mesure = `${octetsLisibles(espace.libre)} libres sur ${octetsLisibles(espace.total)}`;
      signaler('Mesure faite, rien n’a été écrit.');
    } catch (e) {
      signaler(messageErreur(e), true);
    } finally {
      mesureEnCours = false;
    }
  }

  async function enregistrer() {
    if (!reglages) return;
    try {
      reglages = await enregistrerReglages($state.snapshot(reglages));
      signaler('Réglages enregistrés.');
    } catch (e) {
      signaler(messageErreur(e), true);
    }
  }
</script>

<div class="fenetre">
  <!-- Menu latéral : opaque, le fond animé ne passe jamais dessous. -->
  <nav class="menu" aria-label="Pages">
    <div class="marque">
      <strong>Aria</strong>
      <span class="code">{nomCourt}</span>
    </div>
    <ul>
      <li><a href="#accueil" class="actif" aria-current="page">Accueil</a></li>
    </ul>
    <div class="pied code">
      {#if identite}<span>v{identite.version}</span>{/if}
      <span>tout en local</span>
    </div>
  </nav>

  <main class="zone">
    {#if reglages}
      <FondAnime reglages={reglages.fond} />
    {/if}

    <div class="contenu" id="accueil">
      <h1>Accueil</h1>
      {#if identite}<p class="sous-titre">{identite.nom} : {identite.promesse}</p>{/if}

      <SousOnglets {onglets} bind:actif={onglet} />
      <ASavoir>{commentaires[onglet]}</ASavoir>

      {#if onglet === 'apercu'}
        <div class="pile">
          <Panneau titre="Exemple d'action" accentue>
            <p class="discret">
              Exemple à remplacer : le bouton orange mesure la place libre sur le disque de ton dossier de données.
              Lecture seule, rien n'est écrit.
            </p>
            <Jauge etiquette="Disque" {fraction} mesure={mesure || undefined} />
            <div class="actions">
              <Bouton genre="principal" desactive={mesureEnCours} onclick={mesurer}>Mesurer le disque</Bouton>
            </div>
          </Panneau>

          <Panneau titre="Dossier de données">
            <p>Tout ce que l'appli garde est ici : réglages, journal, sauvegardes, cache.</p>
            {#if identite}<p class="code discret">{identite.dossier}</p>{/if}
          </Panneau>
        </div>
      {:else if reglages}
        <div class="pile">
          {#if identite?.enFond}
            <Panneau titre="Démarrage">
              <label class="ligne">
                <input type="checkbox" bind:checked={demarrageAuto} onchange={changerDemarrageAuto} />
                Démarrer avec Windows (dans l'icône de notification, sans ouvrir la fenêtre)
              </label>
            </Panneau>
          {/if}

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
              <label>Au moins <input type="number" min="1" max="12" bind:value={reglages.fond.minimum} onchange={enregistrer} /></label>
              <label>au plus <input type="number" min="1" max="12" bind:value={reglages.fond.maximum} onchange={enregistrer} /></label>
            </div>
          </Panneau>
        </div>
      {/if}

      <p class="etat" class:erreur role="status">{etat}</p>
    </div>
  </main>
</div>

<style>
  .fenetre {
    display: grid;
    grid-template-columns: 220px 1fr;
    height: 100%;
  }

  .menu {
    display: flex;
    flex-direction: column;
    gap: 18px;
    padding: 22px 12px 18px 20px;
    background: var(--panneau);
    border-right: 1px solid var(--ligne);
  }

  .marque {
    display: flex;
    flex-direction: column;
  }

  .marque strong {
    font-size: 15px;
    letter-spacing: 0.08em;
    text-transform: uppercase;
  }

  .marque span {
    font-size: 11px;
    color: var(--cyan);
    text-transform: uppercase;
  }

  ul {
    flex: 1;
    margin: 0;
    padding: 0;
    list-style: none;
  }

  ul a {
    display: block;
    padding: 8px 10px;
    border-left: 3px solid transparent;
    color: var(--texte-2);
    font-size: 13px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-decoration: none;
    text-transform: uppercase;
  }

  ul a.actif,
  ul a:hover {
    color: var(--cyan);
    border-left-color: var(--cyan);
  }

  .pied {
    display: flex;
    flex-direction: column;
    font-size: 11px;
    color: var(--texte-2);
  }

  .zone {
    position: relative;
    overflow: auto;
  }

  .contenu {
    position: relative;
    z-index: 1;
    display: flex;
    flex-direction: column;
    gap: 14px;
    max-width: 860px;
    padding: 26px 32px 32px;
  }

  h1 {
    margin: 0;
    font-size: 26px;
    font-weight: 700;
  }

  .sous-titre {
    margin: 0 0 6px;
    color: var(--texte-2);
  }

  .pile {
    display: flex;
    flex-direction: column;
    gap: 16px;
  }

  .discret {
    color: var(--texte-2);
  }

  .code {
    font-family: var(--police-code);
    font-size: 12px;
  }

  .actions {
    margin-top: 18px;
  }

  .etiquette {
    margin: 18px 0 8px;
    font-size: 12px;
    font-weight: 700;
    letter-spacing: 0.08em;
    text-transform: uppercase;
    color: var(--texte-2);
  }

  .cartes {
    display: grid;
    grid-template-columns: repeat(3, 1fr);
    gap: 12px;
  }

  .ligne {
    display: flex;
    gap: 16px;
    align-items: center;
  }

  .etat {
    min-height: 1.5em;
    margin: 0;
    color: var(--texte-2);
  }

  .etat.erreur {
    color: var(--rouge);
  }
</style>
