using Microsoft.Win32;

namespace Aria.Core.Startup;

/// <summary>
/// « Démarrer avec Windows » : une valeur à son nom dans HKCU\Software\Microsoft\Windows\CurrentVersion\Run, qui
/// lance l'exe avec <see cref="SilentArgument"/>. Désactivé par défaut, sans droits administrateur. L'état est
/// toujours relu dans le registre, jamais retenu : l'utilisateur peut le changer ailleurs, notamment dans la page
/// « Applications de démarrage » de Windows (StartupApproved).
/// </summary>
public static class Autostart
{
    /// <summary>Argument du lancement à l'ouverture de session : l'icône seule, la fenêtre reste rangée.</summary>
    public const string SilentArgument = "--silencieux";

    internal const string RunPath = @"Software\Microsoft\Windows\CurrentVersion\Run";
    internal const string ApprovedPath = @"Software\Microsoft\Windows\CurrentVersion\Explorer\StartupApproved\Run";

    /// <summary>Nom de la valeur dans le registre.</summary>
    public static string ValueName => AppIdentity.ExeName;

    /// <summary>Vrai si Windows lancera l'appli à l'ouverture de session.</summary>
    public static bool IsEnabled() => IsEnabled(Registry.CurrentUser, RunPath, ApprovedPath);

    /// <summary>Lance cet exe à l'ouverture de session. Réactive aussi l'entrée si Windows l'avait désactivée.</summary>
    public static void Enable(string exePath) => Enable(Registry.CurrentUser, RunPath, ApprovedPath, exePath);

    /// <summary>Ne lance plus l'appli à l'ouverture de session.</summary>
    public static void Disable() => Disable(Registry.CurrentUser, RunPath);

    /// <summary>Ligne de commande enregistrée : l'exe entre guillemets, puis l'argument silencieux.</summary>
    public static string CommandFor(string exePath) => $"\"{exePath}\" {SilentArgument}";

    internal static bool IsEnabled(RegistryKey root, string runPath, string approvedPath)
    {
        using var run = root.OpenSubKey(runPath);
        if (run?.GetValue(ValueName) is not string { Length: > 0 }) return false;

        // Page « Applications de démarrage » de Windows : premier octet pair = activée, impair = désactivée.
        // Pas de valeur : jamais touchée, donc activée.
        using var approved = root.OpenSubKey(approvedPath);
        return approved?.GetValue(ValueName) is not byte[] { Length: > 0 } state || state[0] % 2 == 0;
    }

    internal static void Enable(RegistryKey root, string runPath, string approvedPath, string exePath)
    {
        if (!Path.IsPathFullyQualified(exePath)) throw new ArgumentException("Chemin de l'exe attendu en entier.", nameof(exePath));
        using (var run = root.CreateSubKey(runPath))
            run.SetValue(ValueName, CommandFor(exePath), RegistryValueKind.String);

        // L'utilisateur vient de cocher la case : une désactivation faite dans Windows est levée.
        using var approved = root.OpenSubKey(approvedPath, writable: true);
        approved?.DeleteValue(ValueName, throwOnMissingValue: false);
    }

    internal static void Disable(RegistryKey root, string runPath)
    {
        using var run = root.OpenSubKey(runPath, writable: true);
        run?.DeleteValue(ValueName, throwOnMissingValue: false);
    }
}
