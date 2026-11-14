namespace Aria.Core;

/// <summary>
/// Nom de l'appli, de son exe et de son dossier de données : un seul endroit pour toute l'appli.
/// </summary>
public static class AppIdentity
{
    /// <summary>Nom affiché.</summary>
    public const string Name = "Aria {{NOM}}";

    /// <summary>Nom de l'exe, sans extension.</summary>
    public const string ExeName = "Aria-{{NOM}}";

    /// <summary>Nom du dossier de données, dans le profil de l'utilisateur.</summary>
    public const string DataFolderName = ".Aria-{{NOM}}";

    /// <summary>
    /// L'appli sert en arrière-plan : icône de notification, la croix range la fenêtre dans l'icône, démarrage
    /// automatique proposé (désactivé par défaut). Faux : ni icône ni démarrage automatique, la croix quitte.
    /// Réglé par init.ps1 d'après la fiche projet ; une propriété plutôt qu'une constante, pour que le code de
    /// l'autre cas compile sans avertissement.
    /// </summary>
    public static bool RunsInBackground { get; } = {{EN_FOND}};

    /// <summary>
    /// Dossier de données : données, réglages, journal, sauvegardes et cache, tout est là.
    /// Jamais dans %APPDATA%, %LOCALAPPDATA% ni %TEMP%.
    /// </summary>
    public static string DataFolder =>
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.UserProfile), DataFolderName);

    /// <summary>Version X.Y.Z, celle de Directory.Build.props.</summary>
    public static string Version =>
        typeof(AppIdentity).Assembly.GetName().Version?.ToString(3) ?? "0.0.0";
}
