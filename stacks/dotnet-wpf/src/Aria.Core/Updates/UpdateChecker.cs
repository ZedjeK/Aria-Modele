using System.Net.Http.Headers;
using System.Text.Json;

namespace Aria.Core.Updates;

/// <summary>Le zip d'une version : son adresse, sa taille, et l'empreinte SHA-256 annoncée par GitHub.</summary>
public sealed record UpdateAsset(string Name, string Url, long Size, string Sha256);

/// <summary>Une version plus récente que celle en cours.</summary>
public sealed record UpdateInfo(Version Version, string PageUrl, UpdateAsset Zip);

/// <summary>
/// Recherche d'une nouvelle version : une seule requête, vers releases/latest du dépôt de l'appli, sur l'API GitHub.
/// Rien n'est envoyé d'autre que cette requête (pas d'identifiant, pas de télémétrie). Seul le zip de la version,
/// sur github.com en HTTPS et accompagné de l'empreinte que GitHub calcule à l'envoi (champ « digest »), est retenu.
/// </summary>
public static class UpdateChecker
{
    public const string Owner = "ZedjeK";
    public static string RepositoryUrl => $"https://github.com/{Owner}/{AppIdentity.ExeName}";

    /// <summary>Début obligatoire de l'adresse du zip : une release de ce dépôt, en HTTPS, sur github.com.</summary>
    public static string DownloadPrefix => RepositoryUrl + "/releases/download/";

    private static string LatestReleaseApi => $"https://api.github.com/repos/{Owner}/{AppIdentity.ExeName}/releases/latest";

    /// <summary>La version plus récente ; null si celle-ci est la dernière, ou si GitHub ne répond pas utilement.</summary>
    public static async Task<UpdateInfo?> CheckAsync(Version current, CancellationToken cancel = default)
    {
        try
        {
            using var http = CreateClient();
            return Parse(await http.GetStringAsync(LatestReleaseApi, cancel).ConfigureAwait(false), current);
        }
        catch (Exception e) when (e is HttpRequestException or TaskCanceledException or OperationCanceledException)
        {
            return null;
        }
    }

    /// <summary>Client HTTP de l'appli : HTTPS, délai court, et un nom d'appli sans rien qui identifie l'utilisateur.</summary>
    public static HttpClient CreateClient()
    {
        var http = new HttpClient { Timeout = TimeSpan.FromSeconds(30) };
        http.DefaultRequestHeaders.UserAgent.Add(new ProductInfoHeaderValue(AppIdentity.ExeName, AppIdentity.Version));
        http.DefaultRequestHeaders.Accept.Add(new MediaTypeWithQualityHeaderValue("application/vnd.github+json"));
        return http;
    }

    /// <summary>Lit la réponse de l'API ; null si la version n'est pas plus récente ou si la réponse est inutilisable.</summary>
    internal static UpdateInfo? Parse(string json, Version current)
    {
        try
        {
            using var doc = JsonDocument.Parse(json);
            var root = doc.RootElement;
            if (root.ValueKind != JsonValueKind.Object) return null;
            if (IsTrue(root, "draft") || IsTrue(root, "prerelease")) return null;
            if (Text(root, "tag_name") is not { } tag || !TryParseVersion(tag, out var latest)) return null;
            if (latest <= new Version(current.Major, current.Minor, Math.Max(current.Build, 0))) return null;

            var expected = $"{AppIdentity.ExeName}-{latest.ToString(3)}.zip";
            if (!root.TryGetProperty("assets", out var assets) || assets.ValueKind != JsonValueKind.Array) return null;
            foreach (var asset in assets.EnumerateArray())
            {
                if (Text(asset, "name") != expected) continue;
                if (Text(asset, "browser_download_url") is not { } url || !url.StartsWith(DownloadPrefix, StringComparison.Ordinal)) return null;
                if (!asset.TryGetProperty("size", out var s) || !s.TryGetInt64(out var size) || size <= 0) return null;
                if (Text(asset, "digest") is not { } digest || ParseSha256(digest) is not { } sha) return null;

                // Seule une page de ce dépôt est proposée à l'ouverture
                var page = Text(root, "html_url") is { } html && html.StartsWith(RepositoryUrl + "/", StringComparison.OrdinalIgnoreCase)
                    ? html : RepositoryUrl + "/releases/latest";
                return new UpdateInfo(latest, page, new UpdateAsset(expected, url, size, sha));
            }
            return null;
        }
        catch (JsonException)
        {
            return null;
        }
    }

    /// <summary>« sha256:ABC… » → les 64 chiffres en minuscules ; null pour tout autre format.</summary>
    internal static string? ParseSha256(string digest)
    {
        const string prefix = "sha256:";
        if (!digest.StartsWith(prefix, StringComparison.OrdinalIgnoreCase)) return null;
        var hex = digest[prefix.Length..].Trim().ToLowerInvariant();
        return hex.Length == 64 && hex.All(Uri.IsHexDigit) ? hex : null;
    }

    /// <summary>« v1.2.3 » → 1.2.3 ; rien d'autre n'est accepté.</summary>
    internal static bool TryParseVersion(string tag, out Version version)
    {
        version = new Version(0, 0, 0);
        if (tag.Length < 2 || tag[0] != 'v') return false;
        var parts = tag[1..].Split('.');
        if (parts.Length != 3 || !parts.All(p => p.Length > 0 && p.All(char.IsAsciiDigit))) return false;
        if (!Version.TryParse(tag[1..], out var parsed)) return false;
        version = parsed;
        return true;
    }

    private static bool IsTrue(JsonElement e, string name) =>
        e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.True;

    private static string? Text(JsonElement e, string name) =>
        e.ValueKind == JsonValueKind.Object && e.TryGetProperty(name, out var v) && v.ValueKind == JsonValueKind.String ? v.GetString() : null;
}
