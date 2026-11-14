using System.IO.Compression;
using System.Security.Cryptography;
using Aria.Core.Data;
using Aria.Core.Install;

namespace Aria.Core.Updates;

/// <summary>
/// Mise à jour intégrée : télécharge le zip de la version, vérifie sa taille et son empreinte SHA-256, garde la
/// version en cours dans les sauvegardes, puis remplace les fichiers là où l'exe tourne (programme\ s'il est
/// installé). Rien n'est installé sans l'accord de l'utilisateur : c'est l'interface qui appelle ces méthodes.
///
/// L'empreinte prouve que le téléchargement est intact, pas d'où il vient : la provenance repose sur le compte de
/// l'auteur, les releases immuables et l'attestation jointe (voir « Limites à connaître » du README).
/// </summary>
public static class UpdateInstaller
{
    public static string WorkFolder => Path.Combine(DataFolder.Root, DataFolder.CacheFolder, "mise-a-jour");

    /// <summary>Télécharge et vérifie le zip ; renvoie son chemin. Lève une exception si le fichier n'est pas le bon.</summary>
    public static async Task<string> DownloadAsync(UpdateAsset asset, CancellationToken cancel = default)
    {
        using var http = UpdateChecker.CreateClient();
        http.Timeout = TimeSpan.FromMinutes(10);
        return await DownloadAsync(http, asset, WorkFolder, cancel).ConfigureAwait(false);
    }

    internal static async Task<string> DownloadAsync(HttpClient http, UpdateAsset asset, string folder, CancellationToken cancel)
    {
        if (!asset.Url.StartsWith(UpdateChecker.DownloadPrefix, StringComparison.Ordinal))
            throw new InvalidDataException("Adresse de téléchargement refusée : elle n'est pas une release de ce dépôt sur github.com.");
        Directory.CreateDirectory(folder);
        var target = Path.Combine(folder, asset.Name);
        var partial = target + ".part";
        try
        {
            using var response = await http.GetAsync(asset.Url, HttpCompletionOption.ResponseHeadersRead, cancel).ConfigureAwait(false);
            response.EnsureSuccessStatusCode();
            // GitHub redirige vers son hôte de téléchargement : toujours en HTTPS
            if (response.RequestMessage?.RequestUri is { Scheme: not "https" })
                throw new InvalidDataException("Le téléchargement a été redirigé hors de HTTPS.");

            using var hash = IncrementalHash.CreateHash(HashAlgorithmName.SHA256);
            long total = 0;
            await using (var input = await response.Content.ReadAsStreamAsync(cancel).ConfigureAwait(false))
            await using (var output = File.Create(partial))
            {
                var buffer = new byte[81920];
                int read;
                while ((read = await input.ReadAsync(buffer, cancel).ConfigureAwait(false)) > 0)
                {
                    total += read;
                    if (total > asset.Size) throw new InvalidDataException("Le fichier téléchargé est plus gros qu'annoncé.");
                    hash.AppendData(buffer, 0, read);
                    await output.WriteAsync(buffer.AsMemory(0, read), cancel).ConfigureAwait(false);
                }
            }
            var actual = Convert.ToHexStringLower(hash.GetHashAndReset());
            if (total != asset.Size || actual != asset.Sha256)
                throw new InvalidDataException("Le fichier téléchargé ne correspond pas à l'empreinte annoncée par GitHub : rien n'est installé.");
            File.Move(partial, target, overwrite: true);
            return target;
        }
        finally
        {
            if (File.Exists(partial)) File.Delete(partial);
        }
    }

    /// <summary>
    /// Installe le zip vérifié dans le dossier de l'exe en cours, en gardant la version actuelle. Ne lève jamais :
    /// à la moindre erreur, rien n'a changé et le résultat dit pourquoi.
    /// </summary>
    public static SelfInstallResult Install(string zip, string exe) =>
        Install(zip, exe, SelfInstall.VersionsFolder, WorkFolder);

    internal static SelfInstallResult Install(string zip, string exe, string versions, string work)
    {
        var exeFolder = Path.GetDirectoryName(Path.GetFullPath(exe))!;
        var extracted = Path.Combine(work, "extrait");
        try
        {
            if (Directory.Exists(extracted)) Directory.Delete(extracted, recursive: true);
            // ExtractToDirectory refuse une entrée qui sortirait du dossier de destination
            ZipFile.ExtractToDirectory(zip, extracted);
            var root = Directory.GetDirectories(extracted) is [var only] && Directory.GetFiles(extracted).Length == 0 ? only : extracted;

            var names = ReleaseFiles.Read(root);
            if (names is null) return new SelfInstallResult(false, $"Le zip ne contient pas de {ReleaseFiles.ListName} lisible : rien n'est installé.");
            if (ReleaseFiles.Missing(root, names) is { Count: > 0 } missing)
                return new SelfInstallResult(false, $"Il manque dans le zip : {string.Join(", ", missing)}. Rien n'est installé.");

            if (SelfInstall.InstalledVersion(exeFolder) is { } current)
            {
                try
                {
                    SelfInstall.Keep(exeFolder, current, versions);
                }
                catch (Exception e) when (e is IOException or UnauthorizedAccessException)
                {
                    return new SelfInstallResult(false, $"La version en cours n'a pas pu être mise de côté : {e.Message}");
                }
            }

            var swapped = SelfInstall.Swap(root, exeFolder, [.. names, ReleaseFiles.ListName]);
            return swapped.Done ? new SelfInstallResult(true, Exe: Path.Combine(exeFolder, SelfInstall.ExeFileName)) : swapped;
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidDataException)
        {
            return new SelfInstallResult(false, $"La mise à jour n'a pas pu être installée : {e.Message}");
        }
        finally
        {
            try
            {
                if (Directory.Exists(extracted)) Directory.Delete(extracted, recursive: true);
            }
            catch (Exception e) when (e is IOException or UnauthorizedAccessException)
            {
                // Dossier de travail resté : retiré au prochain passage
            }
        }
    }
}
