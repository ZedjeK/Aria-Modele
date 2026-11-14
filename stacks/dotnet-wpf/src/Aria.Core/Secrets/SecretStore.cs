using System.Security.Cryptography;
using System.Text;
using System.Text.RegularExpressions;
using Aria.Core.Data;

namespace Aria.Core.Secrets;

/// <summary>
/// Les secrets de l'appli (jetons, mots de passe, clés), chiffrés par la protection des données de Windows (DPAPI)
/// pour le compte de l'utilisateur, un fichier par secret dans secrets\ du dossier de données. Jamais en clair dans
/// reglages.json, le journal ni les sauvegardes.
///
/// Contre quoi cette protection ne joue pas (à dire dans le README) : un programme lancé sous le même compte Windows
/// peut déchiffrer ces fichiers.
/// </summary>
public static partial class SecretStore
{
    private const string Extension = ".dpapi";

    // Lie le chiffrement à cette appli : un secret d'une autre appli Aria ne se déchiffre pas ici par erreur
    private static readonly byte[] Entropy = Encoding.UTF8.GetBytes(AppIdentity.ExeName);

    public static string FolderPath => Path.Combine(DataFolder.Root, DataFolder.SecretsFolder);

    /// <summary>Enregistre un secret sous ce nom (lettres minuscules, chiffres et tirets).</summary>
    public static void Save(string name, string secret) => Save(FolderPath, name, secret);

    /// <summary>Le secret enregistré sous ce nom ; null s'il n'existe pas ou ne se déchiffre pas.</summary>
    public static string? Load(string name) => Load(FolderPath, name);

    /// <summary>Supprime le secret ; sans effet s'il n'existe pas.</summary>
    public static void Delete(string name) => Delete(FolderPath, name);

    internal static void Save(string folder, string name, string secret)
    {
        var file = PathOf(folder, name);
        Directory.CreateDirectory(folder);
        var protectedBytes = ProtectedData.Protect(Encoding.UTF8.GetBytes(secret), Entropy, DataProtectionScope.CurrentUser);
        var temporary = file + ".tmp";
        File.WriteAllBytes(temporary, protectedBytes);
        File.Move(temporary, file, overwrite: true);
    }

    internal static string? Load(string folder, string name)
    {
        var file = PathOf(folder, name);
        try
        {
            if (!File.Exists(file)) return null;
            var clear = ProtectedData.Unprotect(File.ReadAllBytes(file), Entropy, DataProtectionScope.CurrentUser);
            return Encoding.UTF8.GetString(clear);
        }
        catch (Exception e) when (e is CryptographicException or IOException or UnauthorizedAccessException)
        {
            // Fichier abîmé, ou chiffré par un autre compte : le secret est à redemander
            return null;
        }
    }

    internal static void Delete(string folder, string name)
    {
        var file = PathOf(folder, name);
        if (File.Exists(file)) File.Delete(file);
    }

    /// <summary>Le nom devient un nom de fichier : rien qui puisse sortir du dossier des secrets.</summary>
    internal static string PathOf(string folder, string name)
    {
        if (!ValidName().IsMatch(name))
            throw new ArgumentException("Nom de secret refusé : lettres minuscules, chiffres et tirets, 64 caractères au plus.", nameof(name));
        return Path.Combine(folder, name + Extension);
    }

    [GeneratedRegex(@"\A[a-z0-9](?:[a-z0-9-]{0,62}[a-z0-9])?\z")]
    private static partial Regex ValidName();
}
