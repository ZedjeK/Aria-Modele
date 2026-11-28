// Les commandes Rust du socle que l'interface peut appeler (crate aria-coeur, module fenetre), avec leurs types.
// Une commande ajoutée ici doit aussi l'être dans socle\coeur\src\fenetre\liste.rs et dans la capacité de la
// fenêtre qui l'appelle (capacite-principale.json, capacite-menu.json ou capacite-installation.json). Les commandes
// propres à une appli sont dans l'api.ts de son interface.
import { invoke } from '@tauri-apps/api/core';
import { listen, type UnlistenFn } from '@tauri-apps/api/event';

export type Vitesse = 'lente' | 'normale' | 'rapide';

export interface ReglagesFond {
  actif: boolean;
  vitesse: Vitesse;
  minimum: number;
  maximum: number;
}

export interface Reglages {
  /** Le numéro du format de reglages.json. */
  format: number;
  fond: ReglagesFond;
  chercherMisesAJour: boolean;
  /** L'assistant de premier lancement a été vu : il ne s'ouvre plus de lui-même. */
  assistantVu: boolean;
  /** Les réglages propres à l'appli, sous la clé « appli » du fichier. */
  appli: Record<string, unknown>;
}

export interface Identite {
  nom: string;
  version: string;
  promesse: string;
  enFond: boolean;
  dossier: string;
}

export const identite = (): Promise<Identite> => invoke<Identite>('identite');

export const lireReglages = (): Promise<Reglages> => invoke<Reglages>('lire_reglages');

/** Renvoie les réglages vraiment enregistrés : les bornes peuvent avoir été ramenées. */
export const enregistrerReglages = (reglages: Reglages): Promise<Reglages> =>
  invoke<Reglages>('enregistrer_reglages', { reglages });

/** Une opération du journal (journal.json). */
export interface Operation {
  /** RFC 3339, heure locale. */
  date: string;
  type: string;
  bilan: string;
}

/** Les dernières opérations, la plus récente d'abord. */
export const lireJournal = (): Promise<Operation[]> => invoke<Operation[]>('lire_journal');

/** Quitte l'appli, icône de notification comprise. */
export const quitter = (): Promise<void> => invoke<void>('quitter');

/**
 * Le diagnostic à coller dans un signalement de bug (version, build, Windows, WebView2, réglages, 50 dernières lignes
 * du journal) : aucun secret, chemin du profil remplacé par %USERPROFILE%, rien n'est envoyé.
 */
export const diagnostic = (): Promise<string> => invoke<string>('diagnostic');

/** « Démarrer avec Windows », relu dans le registre à chaque appel. */
export const lireDemarrageAuto = (): Promise<boolean> => invoke<boolean>('lire_demarrage_auto');

/** Coche ou décoche « Démarrer avec Windows » ; renvoie l'état relu ensuite. */
export const reglerDemarrageAuto = (actif: boolean): Promise<boolean> =>
  invoke<boolean>('regler_demarrage_auto', { actif });

/** Couleur du point d'une ligne « État » du menu de l'icône. */
export type Ton = 'repos' | 'actif' | 'attention' | 'information';

export interface LigneMenu {
  ton: Ton;
  texte: string;
}

export interface EtatMenu {
  nom: string;
  lignes: LigneMenu[];
  demarrageAuto: boolean;
  /** La version disponible : « Voir la mise à jour » n'apparaît qu'avec elle. */
  miseAJour: string | null;
}

export type ActionMenu = 'ouvrir' | 'parametres' | 'miseAJour' | 'quitter';

export const etatMenu = (): Promise<EtatMenu> => invoke<EtatMenu>('etat_menu');

/** Le menu se range, puis l'action se fait. */
export const actionMenu = (action: ActionMenu): Promise<void> => invoke<void>('action_menu', { action });

/** La fenêtre du menu prend la hauteur de son contenu. */
export const tailleMenu = (hauteur: number): Promise<void> => invoke<void>('taille_menu', { hauteur });

export interface InfosInstallation {
  nom: string;
  version: string;
  /** Le dossier programme\ du dossier de données. */
  dossier: string;
  versionInstallee: string | null;
  /** Le raccourci du Bureau qui serait créé ; null si le Bureau est introuvable. */
  raccourci: string | null;
  /** Ce qui est déjà à sa place : rien, un raccourci vers l'appli (mis à jour), ou vers un autre programme (gardé). */
  raccourciExistant: 'absent' | 'a-nous' | 'ailleurs';
}

export const infosInstallation = (): Promise<InfosInstallation> => invoke<InfosInstallation>('infos_installation');

/** Installe, puis l'appli se relance depuis son dossier et ce lancement se ferme : seul un échec revient ici. */
export const installer = (raccourci: boolean): Promise<void> => invoke<void>('installer', { raccourci });

/** Installation refusée : l'appli se ferme. */
export const refuserInstallation = (): Promise<void> => invoke<void>('refuser_installation');

export interface InfosMiseAJour {
  nom: string;
  actuelle: string;
  version: string;
  /** Taille du zip, en octets. */
  taille: number;
}

/** La mise à jour trouvée au lancement ; null s'il n'y en a pas (ou pas encore). */
export const miseAJourDisponible = (): Promise<InfosMiseAJour | null> =>
  invoke<InfosMiseAJour | null>('mise_a_jour_disponible');

/** Ouvre la page de la release dans le navigateur. */
export const voirNotes = (): Promise<void> => invoke<void>('voir_notes');

/** Télécharge, vérifie et installe, puis l'appli se relance : seul un échec revient ici. */
export const installerMiseAJour = (): Promise<void> => invoke<void>('installer_mise_a_jour');

/** Une étape de la désinstallation : ce qu'elle retire, si c'est fait, et le détail. */
export interface EtapeDesinstallation {
  etape: string;
  reussie: boolean;
  bilan: string;
}

export interface BilanDesinstallation {
  etapes: EtapeDesinstallation[];
  /** Le dossier de données, gardé. */
  donnees: string;
  /** programme\ sera supprimé une fois l'appli fermée par fermerApresDesinstallation. */
  aLaFermeture: boolean;
}

/**
 * Retire le démarrage automatique, le raccourci du Bureau s'il pointe vers l'appli, et programme\ (tout de suite, ou
 * une fois l'appli fermée). Les données restent.
 */
export const desinstaller = (): Promise<BilanDesinstallation> => invoke<BilanDesinstallation>('desinstaller');

/** Ferme l'appli ; si elle tourne depuis programme\, une copie de l'exe le supprime ensuite. */
export const fermerApresDesinstallation = (): Promise<void> => invoke<void>('fermer_apres_desinstallation');

/** Une copie du dossier de sauvegardes. */
export interface Sauvegarde {
  nom: string;
  /** Le fichier d'où elle vient ; null si l'appli ne le sait pas (copie d'une ancienne version). */
  origine: string | null;
  taille: number;
}

/** Les sauvegardes, la plus récente d'abord. */
export const listerSauvegardes = (): Promise<Sauvegarde[]> => invoke<Sauvegarde[]>('lister_sauvegardes');

/** Sauvegarde l'état actuel, puis remet la copie à sa place ; renvoie le fichier restauré. */
export const restaurerSauvegarde = (nom: string): Promise<string> => invoke<string>('restaurer_sauvegarde', { nom });

/** Un champ qui diffère ; null : absent de ce côté. Les champs qui évoquent un secret sont masqués. */
export interface Difference {
  champ: string;
  avant: string | null;
  apres: string | null;
}

/** Ce qu'une restauration changerait : `avant` est l'état actuel, `apres` celui de la copie. */
export interface Comparaison {
  /** La copie et l'état actuel se lisent en JSON : la liste est complète. */
  json: boolean;
  differences: Difference[];
}

export const comparerSauvegarde = (nom: string): Promise<Comparaison> =>
  invoke<Comparaison>('comparer_sauvegarde', { nom });

/** Ce que la reprise des données d'une ancienne version a fait. */
export interface BilanReprise {
  ancien: string;
  copie: string;
  repris: string[];
  nonRepris: string[];
}

export const bilanReprise = (): Promise<BilanReprise | null> => invoke<BilanReprise | null>('bilan_reprise');

/** Une tâche longue, telle que l'appli la décrit (événement « tache »). */
export interface Avancement {
  id: number;
  libelle: string;
  /** Entre 0 et 1 ; null tant que la durée est inconnue. */
  progression: number | null;
  detail: string;
  etape: 'en-cours' | 'finie' | 'annulee' | 'echec';
}

/** Demande l'annulation d'une tâche : elle s'arrête à son prochain pas. */
export const annulerTache = (id: number): Promise<boolean> => invoke<boolean>('annuler_tache', { id });

/**
 * Événements envoyés par l'appli : « etat-icone » (le menu relit son état), « aller-a » (page ou dialogue à
 * montrer : « parametres », « mise-a-jour »), « mise-a-jour » (version disponible), « tache » (une tâche longue
 * avance), « restauree » (un fichier a été restauré).
 */
export const ecouter = <T>(
  evenement: 'etat-icone' | 'aller-a' | 'mise-a-jour' | 'tache' | 'restauree',
  rappel: (charge: T) => void,
): Promise<UnlistenFn> =>
  listen<T>(evenement, (e) => rappel(e.payload));

/** Le message d'une erreur renvoyée par une commande Rust (du texte) ou par le navigateur. */
export function messageErreur(erreur: unknown): string {
  if (typeof erreur === 'string') return erreur;
  if (erreur instanceof Error) return erreur.message;
  return 'erreur inconnue';
}
