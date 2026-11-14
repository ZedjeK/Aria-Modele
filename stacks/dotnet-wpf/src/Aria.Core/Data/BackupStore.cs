namespace Aria.Core.Data;

/// <summary>
/// Sauvegardes : une copie datée de chaque fichier de l'utilisateur, faite avant de le modifier, dans
/// sauvegardes\ du dossier de données. Rien n'est jamais supprimé ni écrasé ici ; restaurer, c'est recopier.
/// </summary>
public static class BackupStore
{
    public static string FolderPath => Path.Combine(DataFolder.Root, DataFolder.BackupsFolder);

    /// <summary>
    /// Copie le fichier avant sa modification et renvoie le chemin de la copie. Lève une exception si la copie
    /// échoue : sans sauvegarde, la modification ne doit pas avoir lieu.
    /// </summary>
    public static string Snapshot(string file) => Snapshot(FolderPath, file, DateTime.Now);

    internal static string Snapshot(string folder, string file, DateTime at)
    {
        if (!File.Exists(file)) throw new FileNotFoundException("Fichier à sauvegarder introuvable.", file);
        Directory.CreateDirectory(folder);
        var name = $"{at:yyyy-MM-dd_HH-mm-ss}_{Path.GetFileName(file)}";
        var target = Path.Combine(folder, name);
        // Deux sauvegardes du même fichier dans la même seconde : un suffixe, jamais un écrasement
        for (var n = 2; File.Exists(target); n++)
            target = Path.Combine(folder, $"{Path.GetFileNameWithoutExtension(name)}_{n}{Path.GetExtension(name)}");
        File.Copy(file, target, overwrite: false);
        return target;
    }
}
