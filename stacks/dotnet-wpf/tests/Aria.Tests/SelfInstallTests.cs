using Aria.Core.Install;

namespace Aria.Tests;

public sealed class SelfInstallTests : IDisposable
{
    private readonly TemporaryFolder _temp = new();
    private string Zip => Path.Combine(_temp.Path, "Telechargements", "Aria-Essai-0.1.0");
    private string Data => Path.Combine(_temp.Path, ".Aria-Essai");
    private string Home => Path.Combine(Data, SelfInstall.FolderName);
    private string Versions => Path.Combine(Data, "sauvegardes", SelfInstall.VersionsFolderName);
    private string Cache => Path.Combine(Data, "cache");

    private static readonly string[] Delivered = [SelfInstall.ExeFileName, "PresentationNative_cor3.dll", "LICENCES-TIERS.txt"];

    public void Dispose() => _temp.Dispose();

    /// <summary>Un dossier comme celui du zip décompressé : les fichiers livrés et leur fichiers.txt.</summary>
    private string UnzippedRelease(string exeName = "")
    {
        Directory.CreateDirectory(Zip);
        foreach (var name in Delivered) File.WriteAllText(Path.Combine(Zip, name), "contenu de " + name);
        File.WriteAllLines(Path.Combine(Zip, ReleaseFiles.ListName), Delivered);
        var exe = Path.Combine(Zip, SelfInstall.ExeFileName);
        if (exeName.Length == 0) return exe;
        var renamed = Path.Combine(Zip, exeName);
        File.Move(exe, renamed);
        return renamed;
    }

    private SelfInstallResult Install(string exe, bool shortcut = false, string? shortcutPath = null, Func<string, string?>? whyUnsafe = null) =>
        SelfInstall.Install(exe, shortcut, Data, Home, Versions, Cache, shortcutPath, whyUnsafe);

    [Fact]
    public void LExeDeProgrammeEstChezLuiLesAutresSontAilleurs()
    {
        Assert.Equal(InstallPlace.Home, SelfInstall.PlaceOf(Path.Combine(Home, SelfInstall.ExeFileName), Home));
        Assert.Equal(InstallPlace.Elsewhere, SelfInstall.PlaceOf(Path.Combine(Zip, SelfInstall.ExeFileName), Home));
        Assert.Equal(InstallPlace.Unknown, SelfInstall.PlaceOf(null, Home));
    }

    [Fact]
    public void LInstallationCopieToutLaListeEtLaListeElleMeme()
    {
        var result = Install(UnzippedRelease());

        Assert.True(result.Done, result.Error);
        Assert.Equal(Path.Combine(Home, SelfInstall.ExeFileName), result.Exe);
        foreach (var name in Delivered.Append(ReleaseFiles.ListName))
            Assert.True(File.Exists(Path.Combine(Home, name)), name);
        Assert.False(Directory.Exists(Path.Combine(Cache, "installation")));
    }

    [Fact]
    public void UnExeRenommeParLeNavigateurEstInstalleSousSonNom()
    {
        var result = Install(UnzippedRelease("Aria-Essai (1).exe"));

        Assert.True(result.Done, result.Error);
        Assert.Equal("contenu de " + SelfInstall.ExeFileName, File.ReadAllText(Path.Combine(Home, SelfInstall.ExeFileName)));
    }

    [Fact]
    public void SansFichiersTxtLInstallationEstRefuseeEtDitDeDecompresser()
    {
        var exe = UnzippedRelease();
        File.Delete(Path.Combine(Zip, ReleaseFiles.ListName));

        var result = Install(exe);

        Assert.False(result.Done);
        Assert.Contains("Décompresse d'abord", result.Error);
        Assert.False(Directory.Exists(Home));
    }

    [Fact]
    public void UnFichierManquantBloqueLInstallationSansRienToucher()
    {
        var exe = UnzippedRelease();
        File.Delete(Path.Combine(Zip, "PresentationNative_cor3.dll"));

        var result = Install(exe);

        Assert.False(result.Done);
        Assert.Contains("PresentationNative_cor3.dll", result.Error);
        Assert.Contains("Décompresse d'abord", result.Error);
        Assert.False(Directory.Exists(Home));
    }

    [Fact]
    public void LaMarqueInternetNeSuitPas()
    {
        var exe = UnzippedRelease();
        File.WriteAllText(exe + ":Zone.Identifier", "[ZoneTransfer]\r\nZoneId=3\r\n");
        Assert.True(File.Exists(exe + ":Zone.Identifier"));

        Assert.True(Install(exe).Done);

        Assert.False(File.Exists(Path.Combine(Home, SelfInstall.ExeFileName) + ":Zone.Identifier"));
    }

    [Fact]
    public void UnDossierNonProtegeEstRefuseQuandLAppliEstElevee()
    {
        var result = Install(UnzippedRelease(), whyUnsafe: _ => "Utilisateurs peut y écrire");

        Assert.False(result.Done);
        Assert.Contains("Utilisateurs peut y écrire", result.Error);
        Assert.False(Directory.Exists(Home));
    }

    [Fact]
    public void LeRaccourciDemandeEstCreeVersLExeInstalle()
    {
        var shortcut = Path.Combine(_temp.Path, "Bureau", "Aria Essai.lnk");
        Directory.CreateDirectory(Path.GetDirectoryName(shortcut)!);

        var result = Install(UnzippedRelease(), shortcut: true, shortcutPath: shortcut);

        Assert.True(result.ShortcutDone, result.ShortcutError);
        Assert.Equal(Path.Combine(Home, SelfInstall.ExeFileName), DesktopShortcut.TargetOf(shortcut), ignoreCase: true);
    }

    [Fact]
    public void LaVersionInstalleeEstGardeeDeCoteEtLesPlusAnciennesRetirees()
    {
        Directory.CreateDirectory(Home);
        foreach (var name in Delivered) File.WriteAllText(Path.Combine(Home, name), "ancienne " + name);
        File.WriteAllLines(Path.Combine(Home, ReleaseFiles.ListName), Delivered);
        Directory.CreateDirectory(Path.Combine(Versions, "0.0.9"));

        SelfInstall.Keep(Home, new Version(0, 1, 0), Versions);

        var kept = Path.Combine(Versions, "0.1.0");
        Assert.Equal("ancienne " + SelfInstall.ExeFileName, File.ReadAllText(Path.Combine(kept, SelfInstall.ExeFileName)));
        Assert.True(File.Exists(Path.Combine(kept, ReleaseFiles.ListName)));
        Assert.False(Directory.Exists(Path.Combine(Versions, "0.0.9")));
    }

    [Fact]
    public void LeRemplacementLaisseDesAnciensQueLeMenageRetire()
    {
        Directory.CreateDirectory(Home);
        File.WriteAllText(Path.Combine(Home, SelfInstall.ExeFileName), "ancien");
        var source = Path.Combine(_temp.Path, "source");
        Directory.CreateDirectory(source);
        File.WriteAllText(Path.Combine(source, SelfInstall.ExeFileName), "nouveau");

        Assert.True(SelfInstall.Swap(source, Home, [SelfInstall.ExeFileName]).Done);
        Assert.Equal("nouveau", File.ReadAllText(Path.Combine(Home, SelfInstall.ExeFileName)));
        Assert.True(File.Exists(Path.Combine(Home, SelfInstall.ExeFileName + ".ancien")));

        SelfInstall.CleanUp(Home);
        Assert.False(File.Exists(Path.Combine(Home, SelfInstall.ExeFileName + ".ancien")));
    }

    [Theory]
    [InlineData("..\\reglages.json")]
    [InlineData("sous-dossier\\fichier.dll")]
    [InlineData("C:\\Windows\\system32\\x.dll")]
    public void UneListeQuiSortiraitDuDossierEstRejetee(string name)
    {
        Directory.CreateDirectory(Zip);
        File.WriteAllLines(Path.Combine(Zip, ReleaseFiles.ListName), [SelfInstall.ExeFileName, name]);

        Assert.Null(ReleaseFiles.Read(Zip));
    }

    [Fact]
    public void UneListeSansLExeEstRejetee()
    {
        Directory.CreateDirectory(Zip);
        File.WriteAllLines(Path.Combine(Zip, ReleaseFiles.ListName), ["LICENCES-TIERS.txt"]);

        Assert.Null(ReleaseFiles.Read(Zip));
    }
}
