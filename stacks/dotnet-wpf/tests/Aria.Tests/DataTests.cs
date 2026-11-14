using System.Text.Json;
using Aria.Core.Data;

namespace Aria.Tests;

public class DataFolderTests
{
    [Fact]
    public void EnsureCreeLeDossierEtSesSousDossiers()
    {
        using var temp = new TemporaryFolder();
        var root = Path.Combine(temp.Path, ".Aria-Essai");

        DataFolder.Ensure(root);
        DataFolder.Ensure(root); // une deuxième fois ne casse rien

        Assert.True(Directory.Exists(Path.Combine(root, DataFolder.BackupsFolder)));
        Assert.True(Directory.Exists(Path.Combine(root, DataFolder.CacheFolder)));
        Assert.True(Directory.Exists(Path.Combine(root, DataFolder.SecretsFolder)));
    }
}

public class AppSettingsTests
{
    [Fact]
    public void SansFichierCeSontLesReglagesParDefaut()
    {
        using var temp = new TemporaryFolder();

        Assert.Equal(AppSettings.Default, AppSettingsStore.Load(temp.Path));
    }

    [Fact]
    public void UnFichierIllisibleDonneLesReglagesParDefaut()
    {
        using var temp = new TemporaryFolder();
        File.WriteAllText(Path.Combine(temp.Path, DataFolder.SettingsFile), "{ pas du json");

        Assert.Equal(AppSettings.Default, AppSettingsStore.Load(temp.Path));
    }

    [Fact]
    public void LesReglagesSurviventALAllerRetour()
    {
        using var temp = new TemporaryFolder();
        var settings = new AppSettings { Background = new BackgroundSettings(false, BackgroundSpeed.Fast, 3, 8) };

        AppSettingsStore.Save(temp.Path, settings);

        Assert.Equal(settings, AppSettingsStore.Load(temp.Path));
        Assert.False(File.Exists(Path.Combine(temp.Path, DataFolder.SettingsFile + ".tmp")));
    }

    [Fact]
    public void LeFondSEcritEnFrancaisSousLaCleFond()
    {
        var json = AppSettingsStore.ToJson(new AppSettings { Background = new BackgroundSettings(true, BackgroundSpeed.Slow, 2, 5) });
        var fond = JsonDocument.Parse(json).RootElement.GetProperty("fond");

        Assert.True(fond.GetProperty("actif").GetBoolean());
        Assert.Equal("lente", fond.GetProperty("vitesse").GetString());
        Assert.Equal(2, fond.GetProperty("minimum").GetInt32());
        Assert.Equal(5, fond.GetProperty("maximum").GetInt32());
    }

    [Fact]
    public void UnFondModifieALaMainEstRemisDansSesBornes()
    {
        var settings = AppSettingsStore.FromJson("""{ "fond": { "actif": true, "vitesse": "folle", "minimum": 40, "maximum": -3 } }""");

        Assert.Equal(BackgroundSpeed.Normal, settings.Background.Speed);
        Assert.Equal(BackgroundSettings.MinElements, settings.Background.Minimum);
        Assert.Equal(BackgroundSettings.MaxElements, settings.Background.Maximum);
    }
}

public class OperationJournalTests
{
    [Fact]
    public void LeJournalGardeLesOperationsDansLOrdre()
    {
        using var temp = new TemporaryFolder();
        var debut = new DateTime(2026, 11, 14, 10, 0, 0);

        OperationJournal.Record(temp.Path, "essai", "première", debut);
        OperationJournal.Record(temp.Path, "essai", "seconde", debut.AddMinutes(1));

        var entries = OperationJournal.Load(temp.Path);
        Assert.Equal(["première", "seconde"], entries.Select(e => e.Summary));
    }

    [Fact]
    public void LeJournalNeGardeQueLes500DernieresOperations()
    {
        using var temp = new TemporaryFolder();
        var debut = new DateTime(2026, 11, 14);
        var entries = Enumerable.Range(0, OperationJournal.MaxEntries)
            .Select(i => new JournalEntry(debut.AddSeconds(i), "essai", $"n{i}")).ToList();
        File.WriteAllText(Path.Combine(temp.Path, DataFolder.JournalFile), JsonSerializer.Serialize(entries));

        OperationJournal.Record(temp.Path, "essai", "la plus récente", debut.AddDays(1));

        var loaded = OperationJournal.Load(temp.Path);
        Assert.Equal(OperationJournal.MaxEntries, loaded.Count);
        Assert.Equal("n1", loaded[0].Summary);
        Assert.Equal("la plus récente", loaded[^1].Summary);
    }

    [Fact]
    public void UnJournalIllisibleNEmpechePasDEnregistrer()
    {
        using var temp = new TemporaryFolder();
        File.WriteAllText(Path.Combine(temp.Path, DataFolder.JournalFile), "abîmé");

        Assert.Empty(OperationJournal.Load(temp.Path));
        OperationJournal.Record(temp.Path, "essai", "après", DateTime.Now);
        Assert.Single(OperationJournal.Load(temp.Path));
    }
}

public class BackupStoreTests
{
    [Fact]
    public void LaSauvegardeEstUneCopieDateeQuiNEcraseRien()
    {
        using var temp = new TemporaryFolder();
        var file = Path.Combine(temp.Path, "hosts");
        File.WriteAllText(file, "contenu d'origine");
        var backups = Path.Combine(temp.Path, DataFolder.BackupsFolder);
        var at = new DateTime(2026, 11, 14, 10, 30, 0);

        var first = BackupStore.Snapshot(backups, file, at);
        File.WriteAllText(file, "contenu modifié");
        var second = BackupStore.Snapshot(backups, file, at);

        Assert.NotEqual(first, second);
        Assert.Equal("contenu d'origine", File.ReadAllText(first));
        Assert.Equal("contenu modifié", File.ReadAllText(second));
        Assert.StartsWith("2026-11-14_10-30-00_hosts", Path.GetFileName(first));
    }

    [Fact]
    public void UnFichierAbsentNeSeSauvegardePas()
    {
        using var temp = new TemporaryFolder();

        Assert.Throws<FileNotFoundException>(() => BackupStore.Snapshot(temp.Path, Path.Combine(temp.Path, "absent"), DateTime.Now));
    }
}
