namespace Aria.Tests;

/// <summary>Un dossier jetable par test : les tests ne touchent jamais au vrai dossier de données.</summary>
public sealed class TemporaryFolder : IDisposable
{
    public string Path { get; } = System.IO.Path.Combine(System.IO.Path.GetTempPath(), "aria-tests-" + Guid.NewGuid().ToString("N")[..8]);

    public TemporaryFolder() => Directory.CreateDirectory(Path);

    public void Dispose()
    {
        try
        {
            Directory.Delete(Path, recursive: true);
        }
        catch (IOException)
        {
            // Fichier encore ouvert : le dossier temporaire sera nettoyé par Windows
        }
    }
}
