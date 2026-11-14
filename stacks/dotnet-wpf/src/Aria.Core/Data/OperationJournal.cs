using System.Text.Json;
using System.Text.Json.Serialization;

namespace Aria.Core.Data;

/// <summary>Une opération du journal : quand, quoi, et son bilan en une ligne.</summary>
public sealed record JournalEntry(
    [property: JsonPropertyName("date")] DateTime At,
    [property: JsonPropertyName("type")] string Kind,
    [property: JsonPropertyName("bilan")] string Summary);

/// <summary>
/// journal.json, dans le dossier de données : les 500 dernières opérations, de la plus ancienne à la plus récente.
/// Jamais de secret dans un bilan.
/// </summary>
public static class OperationJournal
{
    public const int MaxEntries = 500;

    private static readonly JsonSerializerOptions Json = new() { WriteIndented = true };
    private static readonly object Gate = new();

    public static string FilePath => Path.Combine(DataFolder.Root, DataFolder.JournalFile);

    /// <summary>
    /// Ajoute une opération au journal. Ne lève jamais : un journal qui ne s'écrit pas ne doit pas faire échouer
    /// l'opération elle-même.
    /// </summary>
    public static void Record(string kind, string summary) => Record(DataFolder.Root, kind, summary, DateTime.Now);

    internal static void Record(string folder, string kind, string summary, DateTime at)
    {
        try
        {
            lock (Gate)
            {
                var entries = Load(folder).ToList();
                entries.Add(new JournalEntry(at, kind, summary));
                if (entries.Count > MaxEntries) entries.RemoveRange(0, entries.Count - MaxEntries);
                Directory.CreateDirectory(folder);
                var file = Path.Combine(folder, DataFolder.JournalFile);
                var temporary = file + ".tmp";
                File.WriteAllText(temporary, JsonSerializer.Serialize(entries, Json));
                File.Move(temporary, file, overwrite: true);
            }
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or JsonException)
        {
            // Dossier en lecture seule, fichier verrouillé : l'opération a eu lieu quand même
        }
    }

    /// <summary>Opérations enregistrées, de la plus ancienne à la plus récente ; aucune si le fichier est absent ou illisible.</summary>
    public static IReadOnlyList<JournalEntry> Load() => Load(DataFolder.Root);

    internal static IReadOnlyList<JournalEntry> Load(string folder)
    {
        try
        {
            var file = Path.Combine(folder, DataFolder.JournalFile);
            if (!File.Exists(file)) return [];
            return JsonSerializer.Deserialize<List<JournalEntry>>(File.ReadAllText(file), Json) ?? [];
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or JsonException)
        {
            return [];
        }
    }
}
