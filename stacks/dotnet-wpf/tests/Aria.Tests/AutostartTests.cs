using Aria.Core;
using Aria.Core.Startup;
using Microsoft.Win32;

namespace Aria.Tests;

/// <summary>
/// Les tests travaillent sous une clé jetable, HKCU\Software\Aria-Tests-&lt;guid&gt;, supprimée à la fin : jamais dans
/// la vraie clé Run.
/// </summary>
public sealed class AutostartTests : IDisposable
{
    private readonly string _root = $@"Software\Aria-Tests-{Guid.NewGuid():N}";
    private string RunPath => _root + @"\Run";
    private string ApprovedPath => _root + @"\StartupApproved\Run";

    private const string Exe = @"C:\Users\essai\.Aria-Essai\programme\Aria-Essai.exe";

    public void Dispose() => Registry.CurrentUser.DeleteSubKeyTree(_root, throwOnMissingSubKey: false);

    [Fact]
    public void DesactiveParDefaut()
    {
        Assert.False(Autostart.IsEnabled(Registry.CurrentUser, RunPath, ApprovedPath));
    }

    [Fact]
    public void ActiverEnregistreLExeEntreGuillemetsEtLArgumentSilencieux()
    {
        Autostart.Enable(Registry.CurrentUser, RunPath, ApprovedPath, Exe);

        using var run = Registry.CurrentUser.OpenSubKey(RunPath);
        Assert.Equal($"\"{Exe}\" --silencieux", run!.GetValue(AppIdentity.ExeName));
        Assert.True(Autostart.IsEnabled(Registry.CurrentUser, RunPath, ApprovedPath));
    }

    [Fact]
    public void DesactiveDansWindowsCEstDesactive()
    {
        Autostart.Enable(Registry.CurrentUser, RunPath, ApprovedPath, Exe);
        using (var approved = Registry.CurrentUser.CreateSubKey(ApprovedPath))
            approved.SetValue(AppIdentity.ExeName, new byte[] { 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 }, RegistryValueKind.Binary);

        Assert.False(Autostart.IsEnabled(Registry.CurrentUser, RunPath, ApprovedPath));
    }

    [Fact]
    public void CocherLaCaseLeveLaDesactivationDeWindows()
    {
        using (var approved = Registry.CurrentUser.CreateSubKey(ApprovedPath))
            approved.SetValue(AppIdentity.ExeName, new byte[] { 3, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0 }, RegistryValueKind.Binary);

        Autostart.Enable(Registry.CurrentUser, RunPath, ApprovedPath, Exe);

        Assert.True(Autostart.IsEnabled(Registry.CurrentUser, RunPath, ApprovedPath));
    }

    [Fact]
    public void DesactiverRetireLaValeur()
    {
        Autostart.Enable(Registry.CurrentUser, RunPath, ApprovedPath, Exe);

        Autostart.Disable(Registry.CurrentUser, RunPath);
        Autostart.Disable(Registry.CurrentUser, RunPath); // une deuxième fois ne casse rien

        Assert.False(Autostart.IsEnabled(Registry.CurrentUser, RunPath, ApprovedPath));
    }

    [Fact]
    public void UnCheminRelatifEstRefuse()
    {
        Assert.Throws<ArgumentException>(() => Autostart.Enable(Registry.CurrentUser, RunPath, ApprovedPath, "Aria-Essai.exe"));
    }
}
