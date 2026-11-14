using System.Text.RegularExpressions;
using Aria.Core;

namespace Aria.Tests;

public class AppIdentityTests
{
    [Fact]
    public void LeDossierDeDonneesEstDansLeProfilDeLUtilisateur()
    {
        var profil = Environment.GetFolderPath(Environment.SpecialFolder.UserProfile);

        Assert.Equal(profil, Path.GetDirectoryName(AppIdentity.DataFolder));
        Assert.Equal(AppIdentity.DataFolderName, Path.GetFileName(AppIdentity.DataFolder));
        Assert.StartsWith(".Aria-", AppIdentity.DataFolderName);
    }

    [Fact]
    public void LeDossierDeDonneesNEstNiDansAppDataNiDansTemp()
    {
        var interdits = new[]
        {
            Environment.GetFolderPath(Environment.SpecialFolder.ApplicationData),
            Environment.GetFolderPath(Environment.SpecialFolder.LocalApplicationData),
            Path.GetTempPath(),
        };

        foreach (var interdit in interdits)
        {
            Assert.False(
                AppIdentity.DataFolder.StartsWith(interdit.TrimEnd('\\'), StringComparison.OrdinalIgnoreCase),
                $"Le dossier de données ne doit pas être sous {interdit}.");
        }
    }

    [Fact]
    public void LExePorteLeNomDeLAppli()
    {
        Assert.Equal(AppIdentity.ExeName, AppIdentity.Name.Replace(' ', '-'));
        Assert.Equal(AppIdentity.DataFolderName, "." + AppIdentity.ExeName);
    }

    [Fact]
    public void LaVersionEstAuFormatXYZ()
    {
        Assert.Matches(new Regex(@"^\d+\.\d+\.\d+$"), AppIdentity.Version);
    }
}
