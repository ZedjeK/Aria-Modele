using System.IO.Compression;
using System.Net;
using System.Security.Cryptography;
using System.Text;
using Aria.Core;
using Aria.Core.Install;
using Aria.Core.Updates;

namespace Aria.Tests;

public class UpdateCheckerTests
{
    private static readonly string Sha = new('a', 64);

    private static string Release(string tag = "v0.2.0", string? name = null, string? url = null, string? digest = "default", bool prerelease = false)
    {
        var version = tag.TrimStart('v');
        name ??= $"{AppIdentity.ExeName}-{version}.zip";
        url ??= $"{UpdateChecker.DownloadPrefix}{tag}/{name}";
        var digestJson = digest is null ? "" : $"""
            , "digest": "{(digest == "default" ? "sha256:" + Sha.ToUpperInvariant() : digest)}"
            """;
        return $$"""
            { "tag_name": "{{tag}}", "draft": false, "prerelease": {{(prerelease ? "true" : "false")}},
              "html_url": "{{UpdateChecker.RepositoryUrl}}/releases/tag/{{tag}}",
              "assets": [ { "name": "{{name}}", "browser_download_url": "{{url}}", "size": 1234 {{digestJson}} } ] }
            """;
    }

    [Fact]
    public void UneVersionPlusRecenteEstProposeeAvecSonZipEtSonEmpreinte()
    {
        var update = UpdateChecker.Parse(Release(), new Version(0, 1, 0));

        Assert.NotNull(update);
        Assert.Equal(new Version(0, 2, 0), update.Version);
        Assert.Equal($"{AppIdentity.ExeName}-0.2.0.zip", update.Zip.Name);
        Assert.Equal(Sha, update.Zip.Sha256);
        Assert.Equal(1234, update.Zip.Size);
    }

    [Theory]
    [InlineData("0.2.0")]
    [InlineData("0.3.0")]
    public void LaMemeVersionOuUnePlusAncienneNeSontPasProposees(string current)
    {
        Assert.Null(UpdateChecker.Parse(Release(), Version.Parse(current)));
    }

    [Fact]
    public void UnePrereleaseNEstPasProposee() =>
        Assert.Null(UpdateChecker.Parse(Release(prerelease: true), new Version(0, 1, 0)));

    [Fact]
    public void UnZipSansEmpreinteNEstPasPropose() =>
        Assert.Null(UpdateChecker.Parse(Release(digest: null), new Version(0, 1, 0)));

    [Fact]
    public void UneEmpreinteMalFormeeNEstPasProposee() =>
        Assert.Null(UpdateChecker.Parse(Release(digest: "sha1:abc"), new Version(0, 1, 0)));

    [Theory]
    [InlineData("https://exemple.invalid/Aria.zip")]
    [InlineData("http://github.com/ZedjeK/autre-depot/releases/download/v0.2.0/x.zip")]
    public void UneAdresseHorsDeLaReleaseDuDepotEstRefusee(string url) =>
        Assert.Null(UpdateChecker.Parse(Release(url: url), new Version(0, 1, 0)));

    [Fact]
    public void UnAutreFichierQueLeZipDeLaVersionNEstPasPris() =>
        Assert.Null(UpdateChecker.Parse(Release(name: "autre.zip"), new Version(0, 1, 0)));

    [Theory]
    [InlineData("0.2.0")]
    [InlineData("v0.2")]
    [InlineData("v0.2.0-beta")]
    public void UnTagMalFormeEstIgnore(string tag) =>
        Assert.Null(UpdateChecker.Parse(Release(tag: tag, name: "x.zip", url: UpdateChecker.DownloadPrefix + "x.zip"), new Version(0, 1, 0)));

    [Fact]
    public void UneReponseIllisibleNeProposeRien() =>
        Assert.Null(UpdateChecker.Parse("<html>trop de requêtes</html>", new Version(0, 1, 0)));
}

public class UpdateInstallerTests
{
    private sealed class FakeServer(byte[] content) : HttpMessageHandler
    {
        protected override Task<HttpResponseMessage> SendAsync(HttpRequestMessage request, CancellationToken cancellationToken) =>
            Task.FromResult(new HttpResponseMessage(HttpStatusCode.OK) { Content = new ByteArrayContent(content), RequestMessage = request });
    }

    private static UpdateAsset Asset(byte[] content, string? sha = null, long? size = null) => new(
        $"{AppIdentity.ExeName}-0.2.0.zip",
        $"{UpdateChecker.DownloadPrefix}v0.2.0/{AppIdentity.ExeName}-0.2.0.zip",
        size ?? content.Length,
        sha ?? Convert.ToHexStringLower(SHA256.HashData(content)));

    [Fact]
    public async Task UnTelechargementConformeEstGarde()
    {
        using var temp = new TemporaryFolder();
        var content = Encoding.UTF8.GetBytes("le zip de la version");
        using var http = new HttpClient(new FakeServer(content));

        var file = await UpdateInstaller.DownloadAsync(http, Asset(content), temp.Path, CancellationToken.None);

        Assert.Equal(content, File.ReadAllBytes(file));
    }

    [Fact]
    public async Task UneEmpreinteQuiNeCorrespondPasRefuseLeFichier()
    {
        using var temp = new TemporaryFolder();
        var content = Encoding.UTF8.GetBytes("un zip remplacé");
        using var http = new HttpClient(new FakeServer(content));

        await Assert.ThrowsAsync<InvalidDataException>(() =>
            UpdateInstaller.DownloadAsync(http, Asset(content, sha: new string('0', 64)), temp.Path, CancellationToken.None));
        Assert.Empty(Directory.GetFiles(temp.Path));
    }

    [Fact]
    public async Task UnFichierPlusGrosQuAnnonceEstRefuse()
    {
        using var temp = new TemporaryFolder();
        var content = Encoding.UTF8.GetBytes("beaucoup plus long qu'annoncé");
        using var http = new HttpClient(new FakeServer(content));

        await Assert.ThrowsAsync<InvalidDataException>(() =>
            UpdateInstaller.DownloadAsync(http, Asset(content, size: 4), temp.Path, CancellationToken.None));
        Assert.Empty(Directory.GetFiles(temp.Path));
    }

    [Fact]
    public async Task UneAdresseHorsDuDepotEstRefuseeSansTelecharger()
    {
        using var temp = new TemporaryFolder();
        using var http = new HttpClient(new FakeServer([1, 2, 3]));
        var asset = Asset([1, 2, 3]) with { Url = "https://exemple.invalid/x.zip" };

        await Assert.ThrowsAsync<InvalidDataException>(() => UpdateInstaller.DownloadAsync(http, asset, temp.Path, CancellationToken.None));
    }

    /// <summary>Un zip comme ceux de la release : un dossier racine, les fichiers et fichiers.txt.</summary>
    private static string ReleaseZip(string folder, string content, bool withList = true)
    {
        var source = Path.Combine(folder, "source", $"{AppIdentity.ExeName}-0.2.0");
        Directory.CreateDirectory(source);
        string[] names = [SelfInstall.ExeFileName, "PresentationNative_cor3.dll"];
        foreach (var name in names) File.WriteAllText(Path.Combine(source, name), content + name);
        if (withList) File.WriteAllLines(Path.Combine(source, ReleaseFiles.ListName), names);
        var zip = Path.Combine(folder, "maj.zip");
        ZipFile.CreateFromDirectory(Path.GetDirectoryName(source)!, zip);
        return zip;
    }

    private static string InstalledApp(string folder)
    {
        var home = Path.Combine(folder, "programme");
        Directory.CreateDirectory(home);
        File.WriteAllText(Path.Combine(home, SelfInstall.ExeFileName), "ancien");
        File.WriteAllText(Path.Combine(home, "PresentationNative_cor3.dll"), "ancienne dll");
        return Path.Combine(home, SelfInstall.ExeFileName);
    }

    [Fact]
    public void LaMiseAJourRemplaceLesFichiersLaOuTourneLExe()
    {
        using var temp = new TemporaryFolder();
        var exe = InstalledApp(temp.Path);

        var result = UpdateInstaller.Install(ReleaseZip(temp.Path, "nouveau "), exe, Path.Combine(temp.Path, "versions"), Path.Combine(temp.Path, "travail"));

        Assert.True(result.Done, result.Error);
        Assert.Equal("nouveau " + SelfInstall.ExeFileName, File.ReadAllText(exe));
        Assert.True(File.Exists(Path.Combine(Path.GetDirectoryName(exe)!, ReleaseFiles.ListName)));
        Assert.False(Directory.Exists(Path.Combine(temp.Path, "travail", "extrait")));
    }

    [Fact]
    public void UnZipSansFichiersTxtNInstalleRien()
    {
        using var temp = new TemporaryFolder();
        var exe = InstalledApp(temp.Path);

        var result = UpdateInstaller.Install(ReleaseZip(temp.Path, "nouveau ", withList: false), exe, Path.Combine(temp.Path, "versions"), Path.Combine(temp.Path, "travail"));

        Assert.False(result.Done);
        Assert.Equal("ancien", File.ReadAllText(exe));
    }

    [Fact]
    public void UnZipQuiSortiraitDuDossierEstRefuse()
    {
        using var temp = new TemporaryFolder();
        var exe = InstalledApp(temp.Path);
        var zip = Path.Combine(temp.Path, "piege.zip");
        using (var archive = ZipFile.Open(zip, ZipArchiveMode.Create))
        using (var writer = new StreamWriter(archive.CreateEntry("../../hors-du-dossier.txt").Open()))
            writer.Write("piège");

        var result = UpdateInstaller.Install(zip, exe, Path.Combine(temp.Path, "versions"), Path.Combine(temp.Path, "travail"));

        Assert.False(result.Done);
        Assert.Equal("ancien", File.ReadAllText(exe));
        Assert.False(File.Exists(Path.Combine(temp.Path, "hors-du-dossier.txt")));
    }
}
