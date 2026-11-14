namespace Aria.Core.Data;

/// <summary>
/// Le dossier de données, %USERPROFILE%\.Aria-{{NOM}}, et ce qu'il contient. Tout ce que l'appli écrit va là :
/// rien dans %APPDATA%, %LOCALAPPDATA% ni %TEMP%.
/// </summary>
public static class DataFolder
{
    /// <summary>Les réglages.</summary>
    public const string SettingsFile = "reglages.json";

    /// <summary>Les 500 dernières opérations.</summary>
    public const string JournalFile = "journal.json";

    /// <summary>Une copie datée de chaque fichier avant qu'il soit modifié.</summary>
    public const string BackupsFolder = "sauvegardes";

    /// <summary>Le cache de l'interface.</summary>
    public const string CacheFolder = "cache";

    /// <summary>Les secrets, chiffrés par DPAPI pour le compte de l'utilisateur.</summary>
    public const string SecretsFolder = "secrets";

    /// <summary>Le dossier de données de l'appli.</summary>
    public static string Root => AppIdentity.DataFolder;

    /// <summary>Crée le dossier et ses sous-dossiers s'ils manquent. Lève une IOException si c'est impossible.</summary>
    public static string Ensure() => Ensure(Root);

    internal static string Ensure(string root)
    {
        Directory.CreateDirectory(root);
        foreach (var folder in new[] { BackupsFolder, CacheFolder, SecretsFolder })
            Directory.CreateDirectory(Path.Combine(root, folder));
        return root;
    }
}
