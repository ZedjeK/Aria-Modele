"use strict";

// --- Appels à l'API : le cookie du jeton part tout seul avec chaque requête de la même origine ---
async function api(chemin, options) {
  const reponse = await fetch(chemin, options);
  if (!reponse.ok) {
    let detail = "";
    try { detail = (await reponse.json()).detail; } catch (_) { /* pas de JSON */ }
    throw new Error(detail || `Erreur ${reponse.status}`);
  }
  return reponse.json();
}

// --- Le fond animé : règles d'appli/fond.py ---
// Coupé : fond.actif faux, ou le système demande moins d'animations. En pause : la page est cachée.
// Motif du squelette : des points cyan et bleus traversent la zone de gauche à droite, à une hauteur et un départ
// tirés au hasard à chaque passage. À remplacer par le motif de l'appli (il dit ce qu'elle fait).
const Fond = (() => {
  const toile = document.getElementById("fond");
  const dessin = toile.getContext("2d");
  const couleurs = ["#7df9ff", "#5c8bff"];
  const reduit = window.matchMedia("(prefers-reduced-motion: reduce)");
  let reglage = { actif: true, duree: 7, minimum: 2, maximum: 5 };
  let points = [];
  let debut = 0;
  let cadre = 0;

  function taille() {
    toile.width = toile.clientWidth * devicePixelRatio;
    toile.height = toile.clientHeight * devicePixelRatio;
    dessin.setTransform(devicePixelRatio, 0, 0, devicePixelRatio, 0, 0);
  }

  function nouveauPassage(maintenant) {
    const nombre = reglage.minimum + Math.floor(Math.random() * (reglage.maximum - reglage.minimum + 1));
    points = Array.from({ length: nombre }, () => {
      const trajet = reglage.duree * (0.6 + 0.4 * Math.random());
      return {
        y: 24 + Math.random() * Math.max(0, toile.clientHeight - 48),
        rayon: 2.5 + Math.random() * 2.5,
        couleur: couleurs[Math.floor(Math.random() * couleurs.length)],
        depart: (reglage.duree - trajet) * Math.random(),
        trajet,
      };
    });
    debut = maintenant;
  }

  function image(maintenant) {
    cadre = 0;
    if (!reglage.actif || reduit.matches || document.hidden) return;
    const ecoule = (maintenant - debut) / 1000;
    if (ecoule >= reglage.duree) nouveauPassage(maintenant);
    dessin.clearRect(0, 0, toile.clientWidth, toile.clientHeight);
    for (const p of points) {
      const t = (ecoule - p.depart) / p.trajet;
      if (t < 0 || t > 1) continue;
      const x = -p.rayon + t * (toile.clientWidth + 2 * p.rayon);
      dessin.globalAlpha = 0.8 * Math.min(1, t * 6, (1 - t) * 6);
      dessin.shadowColor = p.couleur;
      dessin.shadowBlur = 12;
      dessin.fillStyle = p.couleur;
      dessin.beginPath();
      dessin.arc(x, p.y, p.rayon, 0, 2 * Math.PI);
      dessin.fill();
    }
    cadre = requestAnimationFrame(image);
  }

  function relancer() {
    cancelAnimationFrame(cadre);
    dessin.clearRect(0, 0, toile.clientWidth, toile.clientHeight);
    if (reglage.actif && !reduit.matches && !document.hidden) {
      nouveauPassage(performance.now());
      cadre = requestAnimationFrame(image);
    }
  }

  window.addEventListener("resize", () => { taille(); relancer(); });
  document.addEventListener("visibilitychange", relancer);  // pause quand la page est cachée, reprise au retour
  reduit.addEventListener("change", relancer);
  taille();

  return { configurer(nouveau) { reglage = nouveau; relancer(); } };
})();

// --- Réglages ---
const champ = (id) => document.getElementById(id);
const message = champ("message");
let reglagesLus = {};  // ce que la page ne montre pas repart tel quel : un PUT remplace tout le fichier

function afficher(reglages) {
  reglagesLus = reglages;
  champ("fond-actif").checked = reglages.fond.actif;
  champ("fond-vitesse").value = reglages.fond.vitesse;
  champ("fond-minimum").value = reglages.fond.minimum;
  champ("fond-maximum").value = reglages.fond.maximum;
  champ("chercher-maj").checked = reglages.chercherMisesAJour;
}

champ("formulaire").addEventListener("submit", async (evenement) => {
  evenement.preventDefault();
  message.className = "";
  try {
    const enregistres = await api("/api/reglages", {
      method: "PUT",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify({
        ...reglagesLus,
        fond: {
          actif: champ("fond-actif").checked,
          vitesse: champ("fond-vitesse").value,
          minimum: Number(champ("fond-minimum").value),
          maximum: Number(champ("fond-maximum").value),
        },
        chercherMisesAJour: champ("chercher-maj").checked,
      }),
    });
    afficher(enregistres);
    Fond.configurer(await api("/api/fond"));
    message.textContent = "Réglages enregistrés.";
  } catch (erreur) {
    message.className = "erreur";
    message.textContent = erreur.message;
  }
});

champ("quitter").addEventListener("click", async () => {
  try {
    await api("/api/quitter", { method: "POST" });
    document.body.innerHTML = "<p class='arret'>L'appli est arrêtée. Tu peux fermer cet onglet.</p>";
  } catch (erreur) {
    message.className = "erreur";
    message.textContent = erreur.message;
  }
});

// --- Mises à jour : appli/mises_a_jour.py ; rien ne s'installe sans le clic sur « Installer la mise à jour » ---
const MiseAJour = (() => {
  const bandeau = champ("maj");
  const etatTexte = champ("maj-etat");
  let essais = 0;

  function montrer(etat) {
    const nouvelle = etat.disponible;
    bandeau.hidden = !nouvelle;
    if (nouvelle) {
      champ("maj-titre").textContent = `Version ${nouvelle.version} disponible.`;
      champ("maj-detail").textContent = etat.possible
        ? `Tu as la ${etat.actuelle}. La mise à jour garde ta version actuelle de côté, puis relance l'appli.`
        : `Tu as la ${etat.actuelle}. Lancée depuis les sources, l'appli ne se met pas à jour : seul l'exe le fait.`;
      champ("maj-installer").hidden = !etat.possible;
      champ("maj-page").href = nouvelle.page;
    }
    etatTexte.className = "";
    if (etat.enCours) etatTexte.textContent = "Recherche en cours…";
    else if (!etat.cherchee) etatTexte.textContent = "";
    else etatTexte.textContent = nouvelle ? `Version ${nouvelle.version} disponible.` : `Tu as la dernière version (${etat.actuelle}).`;
  }

  // La recherche du lancement tourne côté serveur : on relit son résultat quelques fois
  async function suivre() {
    try {
      const etat = await api("/api/mise-a-jour");
      montrer(etat);
      if (etat.enCours && essais++ < 20) setTimeout(suivre, 2000);
    } catch (_) { /* l'interface reste utilisable sans cette information */ }
  }

  champ("maj-chercher").addEventListener("click", async () => {
    const bouton = champ("maj-chercher");
    bouton.disabled = true;
    etatTexte.className = "";
    etatTexte.textContent = "Recherche en cours…";
    try {
      montrer(await api("/api/mise-a-jour/chercher", { method: "POST" }));
    } catch (erreur) {
      etatTexte.className = "erreur";
      etatTexte.textContent = erreur.message;
    } finally {
      bouton.disabled = false;
    }
  });

  champ("maj-installer").addEventListener("click", async () => {
    const bouton = champ("maj-installer");
    const texte = champ("maj-message");
    bouton.disabled = true;
    texte.className = "";
    texte.textContent = "Téléchargement et vérification de la nouvelle version…";
    try {
      const fait = await api("/api/mise-a-jour/installer", { method: "POST" });
      document.body.innerHTML = "<p class='arret'></p>";
      document.querySelector(".arret").textContent =
        `La version ${fait.installee} est installée et se relance : elle s'ouvrira dans un nouvel onglet. ` +
        "Tu peux fermer celui-ci.";
    } catch (erreur) {
      texte.className = "erreur";
      texte.textContent = erreur.message;
      bouton.disabled = false;
    }
  });

  return { suivre };
})();

// --- Menu : la section affichée suit l'ancre ---
function section() {
  const cible = location.hash || "#accueil";
  for (const lien of document.querySelectorAll(".menu nav a")) lien.classList.toggle("actif", lien.hash === cible);
  for (const panneau of document.querySelectorAll(".panneau")) panneau.hidden = `#${panneau.id}` !== cible;
}
window.addEventListener("hashchange", section);
section();

// --- Démarrage ---
(async () => {
  try {
    const etat = await api("/api/etat");
    champ("version").textContent = `Version ${etat.version}`;
    afficher(await api("/api/reglages"));
    Fond.configurer(await api("/api/fond"));
  } catch (erreur) {
    message.className = "erreur";
    message.textContent = erreur.message;
  }
  MiseAJour.suivre();
})();
