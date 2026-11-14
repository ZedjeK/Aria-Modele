namespace Aria.Core.Install;

/// <summary>
/// Raccourci de l'appli sur le Bureau de l'utilisateur, jamais sur le Bureau public. Créé seulement sur sa demande ;
/// un raccourci déjà là est proposé en mise à jour, jamais remplacé en silence. Sa cible, l'exe de programme\, ne
/// change pas d'une version à l'autre.
/// </summary>
public static class DesktopShortcut
{
    public static string FileName => AppIdentity.Name + ".lnk";

    /// <summary>Bureau de l'utilisateur, redirigé ou non (OneDrive).</summary>
    public static string DefaultPath =>
        Path.Combine(Environment.GetFolderPath(Environment.SpecialFolder.DesktopDirectory), FileName);

    /// <summary>Cible du raccourci ; null s'il n'existe pas ou n'est pas lisible.</summary>
    public static string? TargetOf(string shortcut)
    {
        try
        {
            if (!File.Exists(shortcut)) return null;
            dynamic link = Open(shortcut);
            var target = (string)link.TargetPath;
            return target.Length == 0 ? null : target;
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or System.Runtime.InteropServices.COMException
                                      or InvalidOperationException or Microsoft.CSharp.RuntimeBinder.RuntimeBinderException)
        {
            return null;
        }
    }

    /// <summary>Crée le raccourci, ou en remplace la cible. Lève une IOException si Windows refuse.</summary>
    public static void Create(string shortcut, string target)
    {
        try
        {
            dynamic link = Open(shortcut);
            link.TargetPath = target;
            link.WorkingDirectory = Path.GetDirectoryName(target) ?? "";
            link.IconLocation = target + ",0";
            link.Description = AppIdentity.Name;
            link.Save();
        }
        catch (Exception e) when (e is System.Runtime.InteropServices.COMException or InvalidOperationException
                                      or Microsoft.CSharp.RuntimeBinder.RuntimeBinderException)
        {
            throw new IOException($"Le raccourci n'a pas pu être créé : {e.Message}", e);
        }
    }

    private static object Open(string shortcut)
    {
        var shell = Type.GetTypeFromProgID("WScript.Shell") ?? throw new InvalidOperationException("WScript.Shell est indisponible.");
        dynamic wsh = Activator.CreateInstance(shell)!;
        return wsh.CreateShortcut(shortcut);
    }
}
