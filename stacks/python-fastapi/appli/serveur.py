"""Le serveur local : l'interface web (appli\\statique) et son API, sur 127.0.0.1 seulement.

Chaque appli ajoute ses routes ici. Les lectures sont des GET ; tout ce qui modifie quelque chose est un POST ou un PUT.
"""

import threading
from collections.abc import Callable
from pathlib import Path

from fastapi import Body, FastAPI, HTTPException
from fastapi.responses import FileResponse
from fastapi.staticfiles import StaticFiles

from appli import donnees, fond, identite, mises_a_jour, securite


class MisesAJour:
    """Ce que l'appli sait d'une nouvelle version : la recherche tourne dans un fil, l'interface lit le résultat."""

    def __init__(self):
        self._verrou = threading.Lock()
        self.trouvee: mises_a_jour.MiseAJour | None = None
        self.en_cours = False  # une recherche ou une installation
        self.cherchee = False  # au moins une recherche a abouti (avec ou sans nouvelle version)

    def chercher(self) -> None:
        with self._verrou:
            if self.en_cours:
                return
            self.en_cours = True
        trouvee = None
        try:
            trouvee = mises_a_jour.chercher()  # ne lève jamais : sans réseau, rien n'est trouvé
        finally:
            with self._verrou:
                self.trouvee, self.cherchee, self.en_cours = trouvee, True, False

    def en_dict(self) -> dict:
        with self._verrou:
            trouvee = self.trouvee
            return {"actuelle": identite.version(), "cherchee": self.cherchee, "enCours": self.en_cours,
                    "possible": mises_a_jour.exe_en_cours() is not None,
                    "disponible": {"version": trouvee.version, "page": trouvee.page} if trouvee else None}

    def prendre(self) -> bool:
        """Réserve l'installation : faux si une recherche ou une autre installation est en cours."""
        with self._verrou:
            if self.en_cours:
                return False
            self.en_cours = True
            return True

    def rendre(self) -> None:
        with self._verrou:
            self.en_cours = False


def creer_app(jeton: str, port: int, racine: Path | None = None, arreter: Callable[[], None] | None = None,
              chercher_au_lancement: bool = False,
              empeche_la_mise_a_jour: Callable[[], str | None] | None = None) -> FastAPI:
    """`racine` : le dossier de données (celui de l'utilisateur par défaut) ; `arreter` : ce que fait « Quitter » ;
    `chercher_au_lancement` : chercher une nouvelle version dès maintenant, dans un fil ; `empeche_la_mise_a_jour` :
    la raison, pour l'utilisateur, qui interdit d'installer en ce moment (une opération en cours), ou None."""
    app = FastAPI(title=identite.NOM, version=identite.version(), docs_url=None, redoc_url=None, openapi_url=None)
    securite.installer(app, port, jeton)
    maj = MisesAJour()
    if chercher_au_lancement:
        threading.Thread(target=maj.chercher, daemon=True).start()

    @app.get("/api/etat")
    def etat():
        return {"nom": identite.NOM, "exe": identite.EXE, "version": identite.version(),
                "en_fond": identite.EN_FOND, "installable": identite.INSTALLABLE}

    @app.get("/api/reglages")
    def lire_reglages():
        return donnees.charger_reglages(racine).en_dict()

    @app.put("/api/reglages")
    def ecrire_reglages(brut: dict = Body(...)):
        nouveaux = donnees.Reglages.depuis_dict(brut)
        fichier = (Path(racine) if racine is not None else identite.dossier_donnees()) / donnees.FICHIER_REGLAGES
        try:
            if fichier.is_file():
                donnees.sauvegarder(fichier, racine)  # sans copie, pas de modification
            donnees.enregistrer_reglages(nouveaux, racine)
        except OSError as erreur:
            raise HTTPException(500, f"Réglages non enregistrés : {erreur.strerror or erreur}") from erreur
        donnees.noter("réglages", "Réglages enregistrés.", racine)
        return nouveaux.en_dict()

    @app.get("/api/fond")
    def lire_fond():
        """Ce que le navigateur a besoin de savoir pour dessiner le fond : les règles sont dans appli.fond."""
        reglage = donnees.charger_reglages(racine).fond
        return {"actif": reglage.actif, "duree": fond.duree_passage(reglage), "minimum": reglage.minimum,
                "maximum": reglage.maximum}

    @app.post("/api/quitter")
    def quitter():
        if arreter is None:
            raise HTTPException(409, "Cette appli ne sait pas s'arrêter seule.")
        donnees.noter("arrêt", "Arrêt demandé depuis l'interface.", racine)
        arreter()
        return {"arret": True}

    @app.get("/api/mise-a-jour")
    def etat_mise_a_jour():
        return maj.en_dict()

    @app.post("/api/mise-a-jour/chercher")
    def chercher_mise_a_jour():
        maj.chercher()
        return maj.en_dict()

    @app.post("/api/mise-a-jour/installer")
    def installer_mise_a_jour():
        """Le clic de l'utilisateur sur « Installer la mise à jour » : rien ne se télécharge sans lui."""
        trouvee, exe = maj.trouvee, mises_a_jour.exe_en_cours()
        if trouvee is None:
            raise HTTPException(409, "Aucune nouvelle version n'est connue : lance d'abord une recherche.")
        if exe is None:
            raise HTTPException(409, "Lancée depuis les sources, l'appli ne se met pas à jour : seul l'exe le fait.")
        if arreter is None:
            raise HTTPException(409, "Cette appli ne sait pas s'arrêter seule : elle ne peut pas se relancer.")
        raison = empeche_la_mise_a_jour() if empeche_la_mise_a_jour else None
        if raison:
            raise HTTPException(409, raison)
        if not maj.prendre():
            raise HTTPException(409, "Une recherche ou une installation est déjà en cours.")
        try:
            archive = mises_a_jour.telecharger(trouvee, mises_a_jour.dossier_travail(racine))
            resultat = mises_a_jour.installer(archive, exe, mises_a_jour.dossier_versions(racine),
                                              mises_a_jour.dossier_travail(racine))
            if not resultat.fait:
                raise mises_a_jour.ErreurMiseAJour(resultat.erreur)
        except (mises_a_jour.ErreurMiseAJour, OSError) as erreur:
            maj.rendre()
            donnees.noter("mise à jour", f"Version {trouvee.version} non installée : {erreur}", racine)
            raise HTTPException(500, str(erreur)) from erreur
        donnees.noter("mise à jour", f"Version {trouvee.version} installée à la place de la {identite.version()} ; "
                      f"la {identite.version()} est gardée dans {donnees.DOSSIER_SAUVEGARDES}\\"
                      f"{mises_a_jour.DOSSIER_VERSIONS}.", racine)
        try:
            mises_a_jour.relancer(resultat.exe)
        except OSError as erreur:
            donnees.noter("mise à jour", f"La version {trouvee.version} ne s'est pas relancée : {erreur}", racine)
            raise HTTPException(500, f"La version {trouvee.version} est installée, mais elle n'a pas pu se relancer "
                                     f"({erreur}) : quitte l'appli, puis relance son exe.") from erreur
        arreter()  # la nouvelle version attend la fin de celle-ci pour démarrer
        return {"installee": trouvee.version}

    @app.get("/logo.svg", include_in_schema=False)
    def logo():
        return FileResponse(identite.racine() / "docs" / "logo.svg", media_type="image/svg+xml")

    app.mount("/", StaticFiles(directory=identite.racine() / "appli" / "statique", html=True), name="statique")
    return app
