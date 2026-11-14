namespace Aria.Core.Install;

/// <summary>
/// fichiers.txt, posé dans le zip par tools\empaqueter.ps1 : la liste de ce que livre une version, un nom par ligne.
/// L'installation et les mises à jour copient exactement cette liste, plus fichiers.txt lui-même.
/// </summary>
public static class ReleaseFiles
{
    public const string ListName = "fichiers.txt";

    /// <summary>
    /// Fichiers livrés avec l'exe de ce dossier ; null si fichiers.txt manque ou est incohérent (nom avec un chemin,
    /// exe absent de la liste). Lève seulement si le dossier est illisible.
    /// </summary>
    public static IReadOnlyList<string>? Read(string folder)
    {
        var list = Path.Combine(folder, ListName);
        if (!File.Exists(list)) return null;
        var names = File.ReadAllLines(list).Select(l => l.Trim()).Where(l => l.Length > 0).ToList();
        // Un nom de la liste devient un chemin : rien qui puisse sortir du dossier
        if (names.Count == 0 || names.Any(n => n != Path.GetFileName(n) || n is "." or ".." || n.Equals(ListName, StringComparison.OrdinalIgnoreCase)))
            return null;
        if (!names.Contains(AppIdentity.ExeName + ".exe", StringComparer.OrdinalIgnoreCase)) return null;
        return names;
    }

    /// <summary>Fichiers de la liste absents du dossier.</summary>
    public static IReadOnlyList<string> Missing(string folder, IEnumerable<string> names) =>
        names.Where(n => !File.Exists(Path.Combine(folder, n))).ToList();

    /// <summary>
    /// Copie le contenu seul, sans les flux de données alternatifs : la marque « téléchargé depuis Internet »
    /// (Zone.Identifier) ne suit pas. L'utilisateur a lancé cet exe et accepté l'installation.
    /// </summary>
    public static void CopyContent(string from, string to)
    {
        using (var input = File.OpenRead(from))
        using (var output = File.Create(to))
            input.CopyTo(output);
        File.SetLastWriteTimeUtc(to, File.GetLastWriteTimeUtc(from));
    }
}
