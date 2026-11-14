using System.Diagnostics;
using System.Security.AccessControl;
using System.Security.Principal;
using Aria.Core.Data;

namespace Aria.Core.Install;

/// <summary>Où tourne l'exe : dans son dossier programme\, ou ailleurs (zip décompressé).</summary>
public enum InstallPlace
{
    /// <summary>Chemin illisible : on ne propose rien.</summary>
    Unknown,
    /// <summary>Dans programme\ : installée.</summary>
    Home,
    /// <summary>Téléchargements, Bureau, clé USB… : l'exe peut s'installer.</summary>
    Elsewhere
}

/// <param name="Done">Vrai si les fichiers sont en place ; sinon rien n'a changé, et <paramref name="Error"/> dit pourquoi.</param>
/// <param name="Exe">Exe installé, à relancer.</param>
/// <param name="ShortcutDone">Le raccourci du Bureau a été créé ou mis à jour.</param>
/// <param name="ShortcutError">Le raccourci demandé n'a pas pu être créé (l'installation, elle, est faite).</param>
public sealed record SelfInstallResult(
    bool Done,
    string? Error = null,
    string? Exe = null,
    bool ShortcutDone = false,
    string? ShortcutError = null);

/// <summary>
/// Installation par l'exe lui-même : lancé depuis le zip décompressé, il propose de se ranger dans
/// %USERPROFILE%\.Aria-{{NOM}}\programme avec tout ce que liste fichiers.txt, en gardant la version qui s'y trouvait.
/// Ce n'est pas un installeur : rien n'est écrit hors du dossier de données, sauf le raccourci demandé, et l'exe
/// reste utilisable là où il est (« Lancer sans installer »).
/// </summary>
public static class SelfInstall
{
    public const string FolderName = "programme";
    public const string VersionsFolderName = "versions";

    /// <summary>Argument qui dispense l'appli de proposer l'installation, pour ce lancement.</summary>
    public const string PortableArgument = "--portable";

    private const string NewSuffix = ".nouveau";
    private const string OldSuffix = ".ancien";
    private const string TempSuffix = ".tmp";

    public static string ExeFileName => AppIdentity.ExeName + ".exe";
    public static string Folder => Path.Combine(DataFolder.Root, FolderName);
    public static string HomeExe => Path.Combine(Folder, ExeFileName);

    /// <summary>Version précédente gardée de côté, dans les sauvegardes.</summary>
    public static string VersionsFolder => Path.Combine(DataFolder.Root, DataFolder.BackupsFolder, VersionsFolderName);

    // ------------------------------------------------------------------ Où est l'exe

    public static InstallPlace PlaceOf(string? exe) => PlaceOf(exe, Folder);

    internal static InstallPlace PlaceOf(string? exe, string home)
    {
        if (string.IsNullOrEmpty(exe)) return InstallPlace.Unknown;
        var folder = Path.GetDirectoryName(Path.GetFullPath(exe));
        if (folder is null) return InstallPlace.Unknown;
        return SameFolder(folder, home) ? InstallPlace.Home : InstallPlace.Elsewhere;
    }

    private static bool SameFolder(string a, string b) =>
        string.Equals(Path.GetFullPath(a).TrimEnd(Path.DirectorySeparatorChar), Path.GetFullPath(b).TrimEnd(Path.DirectorySeparatorChar),
            StringComparison.OrdinalIgnoreCase);

    /// <summary>
    /// Proposer l'installation : appli installable, proposition non coupée, pas de « --portable », exe lancé hors de
    /// programme\ et accompagné de son fichiers.txt (un exe compilé à la main, sans zip, ne propose rien).
    /// </summary>
    public static bool ShouldOffer(string? exe, AppSettings settings, IEnumerable<string> args)
    {
        if (!AppIdentity.SelfInstalls || !settings.OfferInstall) return false;
        if (args.Contains(PortableArgument, StringComparer.OrdinalIgnoreCase)) return false;
        if (PlaceOf(exe) != InstallPlace.Elsewhere) return false;
        try
        {
            return ReleaseFiles.Read(Path.GetDirectoryName(Path.GetFullPath(exe!))!) is not null;
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            return false;
        }
    }

    /// <summary>Version installée dans programme\ ; null s'il n'y en a pas ou si elle est illisible.</summary>
    public static Version? InstalledVersion() => InstalledVersion(Folder);

    internal static Version? InstalledVersion(string home)
    {
        try
        {
            var exe = Path.Combine(home, ExeFileName);
            return File.Exists(exe) && Version.TryParse(FileVersionInfo.GetVersionInfo(exe).ProductVersion, out var version) ? version : null;
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            return null;
        }
    }

    // ------------------------------------------------------------------ Droits du dossier

    /// <summary>Vrai si l'appli tourne avec les droits administrateur.</summary>
    public static bool IsElevated()
    {
        using var identity = WindowsIdentity.GetCurrent();
        return new WindowsPrincipal(identity).IsInRole(WindowsBuiltInRole.Administrator);
    }

    /// <summary>
    /// Pourquoi le dossier ne convient pas pour y lancer un exe élevé ; null s'il est sûr. Sûr : seuls le système et
    /// les administrateurs peuvent y écrire, et l'un d'eux en est le propriétaire.
    /// </summary>
    public static string? WhyUnsafe(string folder)
    {
        try
        {
            return WhyUnsafe(new DirectoryInfo(folder).GetAccessControl());
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidOperationException)
        {
            return $"ses droits n'ont pas pu être lus ({e.Message})";
        }
    }

    private static readonly FileSystemRights WriteRights =
        FileSystemRights.WriteData | FileSystemRights.AppendData | FileSystemRights.WriteExtendedAttributes
        | FileSystemRights.WriteAttributes | FileSystemRights.Delete | FileSystemRights.DeleteSubdirectoriesAndFiles
        | FileSystemRights.ChangePermissions | FileSystemRights.TakeOwnership;

    internal static string? WhyUnsafe(DirectorySecurity security)
    {
        var owner = security.GetOwner(typeof(SecurityIdentifier)) as SecurityIdentifier;
        if (owner is null || !IsTrusted(owner)) return $"son propriétaire est {Describe(owner)}, qui peut en changer les droits";

        foreach (FileSystemAccessRule rule in security.GetAccessRules(includeExplicit: true, includeInherited: true, typeof(SecurityIdentifier)))
        {
            if (rule.AccessControlType != AccessControlType.Allow) continue;
            if (rule.IdentityReference is SecurityIdentifier who && IsTrusted(who)) continue;
            if ((rule.FileSystemRights & WriteRights) != 0)
                return $"{Describe(rule.IdentityReference as SecurityIdentifier)} peut y écrire";
        }
        return null;
    }

    /// <summary>Système, administrateurs, TrustedInstaller : ceux à qui Windows réserve déjà Program Files.</summary>
    private static bool IsTrusted(SecurityIdentifier sid) =>
        sid.IsWellKnown(WellKnownSidType.LocalSystemSid)
        || sid.IsWellKnown(WellKnownSidType.BuiltinAdministratorsSid)
        || sid.Value.StartsWith("S-1-5-80-", StringComparison.Ordinal);

    private static string Describe(SecurityIdentifier? sid)
    {
        if (sid is null) return "inconnu";
        try
        {
            return sid.Translate(typeof(NTAccount)).Value;
        }
        catch (IdentityNotMappedException)
        {
            return sid.Value;
        }
    }

    // ------------------------------------------------------------------ Installer

    /// <summary>
    /// Range dans programme\ l'exe et tout ce que liste son fichiers.txt, en gardant la version qui s'y trouvait.
    /// Ne lève jamais : à la moindre erreur, rien n'a changé et le résultat dit pourquoi.
    /// </summary>
    public static SelfInstallResult Install(string exe, bool shortcut) =>
        Install(exe, shortcut, DataFolder.Root, Folder, VersionsFolder, Path.Combine(DataFolder.Root, DataFolder.CacheFolder),
            DesktopShortcut.DefaultPath, IsElevated() ? WhyUnsafe : null);

    /// <param name="whyUnsafe">
    /// Contrôle des droits d'un dossier, seulement pour une appli élevée : le contrat interdit de lancer un exe élevé
    /// depuis un dossier qu'un programme sans droits peut modifier. Une appli lancée avec les droits de l'utilisateur
    /// ne peut pas créer un dossier réservé aux administrateurs, et n'en a pas besoin : null, pas de contrôle.
    /// </param>
    internal static SelfInstallResult Install(string exe, bool shortcut, string dataRoot, string home, string versions, string cache,
        string? shortcutPath, Func<string, string?>? whyUnsafe)
    {
        var source = Path.GetFullPath(exe);
        if (!File.Exists(source)) return Failed("L'exe de l'appli est introuvable.");
        var sourceFolder = Path.GetDirectoryName(source)!;

        IReadOnlyList<string>? names;
        try
        {
            names = ReleaseFiles.Read(sourceFolder);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            return Failed($"La liste des fichiers n'a pas pu être lue : {e.Message}");
        }
        if (names is null)
            return Failed($"{ReleaseFiles.ListName} manque à côté de l'exe. Décompresse d'abord le zip en entier (clic droit, « Extraire tout »).");
        // L'exe en cours peut porter un autre nom (« Aria-X (1).exe ») : il est copié sous le bon
        var missing = ReleaseFiles.Missing(sourceFolder, names.Where(n => !n.Equals(ExeFileName, StringComparison.OrdinalIgnoreCase)));
        if (missing.Count > 0)
            return Failed($"Il manque à côté de l'exe : {string.Join(", ", missing)}. Décompresse d'abord le zip en entier (clic droit, « Extraire tout »).");

        if (whyUnsafe?.Invoke(dataRoot) is { } reason)
            return Failed($"Le dossier de données n'est pas protégé : {reason}. Un exe élevé ne se range pas dans un dossier qu'un programme sans droits pourrait modifier.");

        var staging = Path.Combine(cache, "installation");
        try
        {
            DeleteFolder(staging);
            Directory.CreateDirectory(staging);
            foreach (var name in names)
            {
                var from = name.Equals(ExeFileName, StringComparison.OrdinalIgnoreCase) ? source : Path.Combine(sourceFolder, name);
                ReleaseFiles.CopyContent(from, Path.Combine(staging, name));
            }
            ReleaseFiles.CopyContent(Path.Combine(sourceFolder, ReleaseFiles.ListName), Path.Combine(staging, ReleaseFiles.ListName));

            Directory.CreateDirectory(home);
            // programme\ hérite des droits des données : on le vérifie, il ne suffit pas de le supposer
            if (whyUnsafe?.Invoke(home) is { } homeReason)
                return Failed($"Le dossier du programme n'est pas protégé : {homeReason}.");

            if (InstalledVersion(home) is { } installed)
            {
                try
                {
                    Keep(home, installed, versions);
                }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException)
                {
                    return Failed($"La version installée n'a pas pu être mise de côté : {e.Message}");
                }
            }

            var swapped = Swap(staging, home, [.. names, ReleaseFiles.ListName]);
            if (!swapped.Done) return swapped;
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            return Failed($"Les fichiers n'ont pas pu être copiés : {e.Message}");
        }
        finally
        {
            DeleteFolder(staging);
        }

        var installedExe = Path.Combine(home, ExeFileName);
        if (!shortcut || shortcutPath is null) return new SelfInstallResult(true, Exe: installedExe);
        try
        {
            DesktopShortcut.Create(shortcutPath, installedExe);
            return new SelfInstallResult(true, Exe: installedExe, ShortcutDone: true);
        }
        catch (IOException e)
        {
            return new SelfInstallResult(true, Exe: installedExe, ShortcutError: e.Message);
        }
    }

    /// <summary>
    /// Copie la version installée dans <paramref name="versions"/>, sous son numéro, et retire les versions gardées
    /// avant elle. La copie se fait à côté, puis prend sa place : une copie interrompue ne remplace jamais une version
    /// déjà gardée.
    /// </summary>
    internal static void Keep(string home, Version installed, string versions)
    {
        var names = ReleaseFiles.Read(home) ?? [ExeFileName];
        var name = installed.ToString(3);
        var temp = Path.Combine(versions, name + TempSuffix);
        DeleteFolder(temp);
        Directory.CreateDirectory(temp);
        foreach (var file in names.Append(ReleaseFiles.ListName))
        {
            var from = Path.Combine(home, file);
            if (File.Exists(from)) File.Copy(from, Path.Combine(temp, file), overwrite: true);
        }
        if (!File.Exists(Path.Combine(temp, ExeFileName)))
        {
            DeleteFolder(temp);
            throw new FileNotFoundException($"{ExeFileName} introuvable dans « {home} ».");
        }
        foreach (var other in Directory.GetDirectories(versions))
            if (!string.Equals(other, temp, StringComparison.OrdinalIgnoreCase)) Directory.Delete(other, recursive: true);
        Directory.Move(temp, Path.Combine(versions, name));
    }

    /// <summary>
    /// Remplace, dans <paramref name="home"/>, chaque fichier de la liste par celui de <paramref name="source"/>.
    /// Le fichier en place est renommé en « .ancien » (un exe qui tourne se renomme, il ne s'écrase pas). À la
    /// première erreur, tout ce qui a été fait est défait.
    /// </summary>
    internal static SelfInstallResult Swap(string source, string home, IReadOnlyList<string> names)
    {
        var done = new List<(string Target, bool HadOld)>();
        foreach (var name in names)
        {
            var from = Path.Combine(source, name);
            if (!File.Exists(from)) continue;
            var target = Path.Combine(home, name);
            var fresh = target + NewSuffix;
            var old = target + OldSuffix;
            var hadOld = false;
            try
            {
                File.Copy(from, fresh, overwrite: true);
                if (File.Exists(target))
                {
                    File.Delete(old);
                    File.Move(target, old);
                    hadOld = true;
                }
                File.Move(fresh, target);
                done.Add((target, hadOld));
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException)
            {
                TryDelete(fresh);
                // Le fichier en place avait déjà été mis de côté : il reprend sa place
                if (hadOld && !File.Exists(target)) TryMove(old, target);
                for (var i = done.Count - 1; i >= 0; i--)
                {
                    TryDelete(done[i].Target);
                    if (done[i].HadOld) TryMove(done[i].Target + OldSuffix, done[i].Target);
                }
                return Failed($"« {name} » n'a pas pu être remplacé : {e.Message} Rien n'a été changé.");
            }
        }
        return done.Count > 0 ? new SelfInstallResult(true) : Failed("Aucun fichier à installer.");
    }

    /// <summary>Retire les fichiers « .ancien » laissés par un remplacement, une fois l'ancienne version arrêtée.</summary>
    public static void CleanUp() => CleanUp(Folder);

    internal static void CleanUp(string home)
    {
        if (!Directory.Exists(home)) return;
        foreach (var file in Directory.EnumerateFiles(home, "*" + OldSuffix))
            TryDelete(file);
    }

    private static SelfInstallResult Failed(string error) => new(false, error);

    private static void TryDelete(string file)
    {
        try
        {
            File.Delete(file);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            // Encore utilisé : le prochain ménage le retirera
        }
    }

    private static void TryMove(string from, string to)
    {
        try
        {
            File.Move(from, to);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            // Rien de plus à tenter : l'erreur d'origine est rapportée
        }
    }

    private static void DeleteFolder(string folder)
    {
        try
        {
            if (Directory.Exists(folder)) Directory.Delete(folder, recursive: true);
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            // Dossier de travail resté : il sera retiré au prochain passage
        }
    }
}
