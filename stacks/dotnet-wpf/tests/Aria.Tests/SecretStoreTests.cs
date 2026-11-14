using System.Text;
using Aria.Core.Secrets;

namespace Aria.Tests;

public class SecretStoreTests
{
    private const string Secret = "jeton-de-test-1234";

    [Fact]
    public void UnSecretSeRelitApresChiffrement()
    {
        using var temp = new TemporaryFolder();

        SecretStore.Save(temp.Path, "jeton-github", Secret);

        Assert.Equal(Secret, SecretStore.Load(temp.Path, "jeton-github"));
    }

    [Fact]
    public void LeFichierNeContientPasLeSecretEnClair()
    {
        using var temp = new TemporaryFolder();

        SecretStore.Save(temp.Path, "jeton-github", Secret);

        var bytes = File.ReadAllBytes(Path.Combine(temp.Path, "jeton-github.dpapi"));
        Assert.DoesNotContain(Secret, Encoding.UTF8.GetString(bytes));
        Assert.DoesNotContain(Secret, Encoding.Unicode.GetString(bytes));
    }

    [Fact]
    public void UnSecretAbsentOuAbimeDonneNull()
    {
        using var temp = new TemporaryFolder();
        File.WriteAllBytes(Path.Combine(temp.Path, "abime.dpapi"), [1, 2, 3, 4]);

        Assert.Null(SecretStore.Load(temp.Path, "absent"));
        Assert.Null(SecretStore.Load(temp.Path, "abime"));
    }

    [Fact]
    public void SupprimerUnSecretLeRendIntrouvable()
    {
        using var temp = new TemporaryFolder();
        SecretStore.Save(temp.Path, "cle", Secret);

        SecretStore.Delete(temp.Path, "cle");
        SecretStore.Delete(temp.Path, "cle"); // une deuxième fois ne casse rien

        Assert.Null(SecretStore.Load(temp.Path, "cle"));
    }

    [Theory]
    [InlineData("..\\reglages")]
    [InlineData("../reglages")]
    [InlineData("C:\\Windows\\x")]
    [InlineData("Majuscule")]
    [InlineData("")]
    [InlineData("-tiret-au-debut")]
    public void UnNomQuiSortiraitDuDossierEstRefuse(string name)
    {
        using var temp = new TemporaryFolder();

        Assert.Throws<ArgumentException>(() => SecretStore.Save(temp.Path, name, Secret));
    }
}
