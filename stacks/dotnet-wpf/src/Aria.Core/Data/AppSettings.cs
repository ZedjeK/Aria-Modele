using System.Text.Json;
using System.Text.Json.Serialization;

namespace Aria.Core.Data;

/// <summary>Vitesse du fond animé. Dans le fichier : « lente », « normale », « rapide ».</summary>
public enum BackgroundSpeed
{
    Slow,
    Normal,
    Fast
}

/// <summary>Écrit la vitesse en français ; une valeur inconnue donne « normale ».</summary>
public sealed class BackgroundSpeedConverter : JsonConverter<BackgroundSpeed>
{
    public override BackgroundSpeed Read(ref Utf8JsonReader reader, Type typeToConvert, JsonSerializerOptions options)
    {
        if (reader.TokenType != JsonTokenType.String)
        {
            reader.Skip();
            return BackgroundSpeed.Normal;
        }
        return reader.GetString()?.Trim().ToLowerInvariant() switch
        {
            "lente" => BackgroundSpeed.Slow,
            "rapide" => BackgroundSpeed.Fast,
            _ => BackgroundSpeed.Normal
        };
    }

    public override void Write(Utf8JsonWriter writer, BackgroundSpeed value, JsonSerializerOptions options) =>
        writer.WriteStringValue(value switch
        {
            BackgroundSpeed.Slow => "lente",
            BackgroundSpeed.Fast => "rapide",
            _ => "normale"
        });
}

/// <summary>
/// Le fond animé, réglages obligatoires de toute appli Aria. Dans le fichier : fond.actif, fond.vitesse,
/// fond.minimum, fond.maximum (nombre d'éléments tirés au hasard à chaque passage).
/// </summary>
public sealed record BackgroundSettings(
    [property: JsonPropertyName("actif")] bool Active = true,
    [property: JsonPropertyName("vitesse"), JsonConverter(typeof(BackgroundSpeedConverter))] BackgroundSpeed Speed = BackgroundSpeed.Normal,
    [property: JsonPropertyName("minimum")] int Minimum = 2,
    [property: JsonPropertyName("maximum")] int Maximum = 5)
{
    public const int MinElements = 1, MaxElements = 12;

    public static BackgroundSettings Default { get; } = new();

    /// <summary>Bornes ramenées entre 1 et 12, et remises dans l'ordre si le fichier les a inversées.</summary>
    public BackgroundSettings Sanitized()
    {
        var low = Math.Clamp(Minimum, MinElements, MaxElements);
        var high = Math.Clamp(Maximum, MinElements, MaxElements);
        return this with
        {
            Speed = Enum.IsDefined(Speed) ? Speed : BackgroundSpeed.Normal,
            Minimum = Math.Min(low, high),
            Maximum = Math.Max(low, high)
        };
    }
}

/// <summary>
/// Les réglages de l'appli, enregistrés dans reglages.json. Jamais de secret ici : ils vont dans
/// <see cref="Secrets.SecretStore"/>. Chaque appli ajoute ses propres réglages à ce record.
/// </summary>
public sealed record AppSettings
{
    public static AppSettings Default { get; } = new();

    /// <summary>Fond animé ; absent du fichier, c'est celui par défaut.</summary>
    [JsonPropertyName("fond")]
    public BackgroundSettings Background { get; init; } = BackgroundSettings.Default;

    /// <summary>
    /// Proposer de s'installer quand l'exe est lancé hors de programme\ (zip décompressé). Décoché par « Ne plus me
    /// proposer » ; se recoche dans la fenêtre.
    /// </summary>
    [JsonPropertyName("proposerInstallation")]
    public bool OfferInstall { get; init; } = true;

    /// <summary>Valeurs ramenées dans leurs bornes : un fichier modifié à la main ne casse rien.</summary>
    public AppSettings Sanitized() => this with
    {
        // « fond » à null dans un fichier modifié à la main : le fond par défaut
        Background = (Background ?? BackgroundSettings.Default).Sanitized()
    };
}

/// <summary>Lecture et écriture de reglages.json, dans le dossier de données.</summary>
public static class AppSettingsStore
{
    private static readonly JsonSerializerOptions Json = new() { WriteIndented = true };

    public static string FilePath => Path.Combine(DataFolder.Root, DataFolder.SettingsFile);

    /// <summary>Réglages enregistrés ; ceux par défaut si le fichier est absent, illisible ou incohérent.</summary>
    public static AppSettings Load() => Load(DataFolder.Root);

    /// <summary>Lève une IOException ou une UnauthorizedAccessException si l'écriture échoue.</summary>
    public static void Save(AppSettings settings) => Save(DataFolder.Root, settings);

    internal static AppSettings Load(string folder)
    {
        try
        {
            var file = Path.Combine(folder, DataFolder.SettingsFile);
            return File.Exists(file) ? FromJson(File.ReadAllText(file)) : AppSettings.Default;
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            return AppSettings.Default;
        }
    }

    internal static void Save(string folder, AppSettings settings)
    {
        Directory.CreateDirectory(folder);
        var file = Path.Combine(folder, DataFolder.SettingsFile);
        // Écrit à côté puis remplace : une coupure en pleine écriture ne laisse pas un fichier tronqué
        var temporary = file + ".tmp";
        File.WriteAllText(temporary, ToJson(settings.Sanitized()));
        File.Move(temporary, file, overwrite: true);
    }

    internal static string ToJson(AppSettings settings) => JsonSerializer.Serialize(settings, Json);

    internal static AppSettings FromJson(string json)
    {
        try
        {
            return (JsonSerializer.Deserialize<AppSettings>(json, Json) ?? AppSettings.Default).Sanitized();
        }
        catch (Exception e) when (e is JsonException or NotSupportedException)
        {
            return AppSettings.Default;
        }
    }
}
