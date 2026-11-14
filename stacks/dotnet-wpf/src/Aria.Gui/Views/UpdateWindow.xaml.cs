using System.ComponentModel;
using System.Diagnostics;
using System.IO;
using System.Net.Http;
using System.Windows;
using System.Windows.Media;
using Aria.Core;
using Aria.Core.Install;
using Aria.Core.Updates;

namespace Aria.Gui.Views;

/// <summary>
/// Proposition de mise à jour : rien ne se télécharge ni ne s'installe sans un clic sur « Installer la mise à jour ».
/// Pendant le téléchargement et le remplacement, la fenêtre ne se ferme pas.
/// </summary>
public partial class UpdateWindow : Window
{
    private readonly UpdateInfo _update;
    private readonly string _exe;
    private bool _busy;

    /// <summary>Exe à relancer : la mise à jour est en place.</summary>
    public string? InstalledExe { get; private set; }

    public UpdateWindow(UpdateInfo update, string exe)
    {
        _update = update;
        _exe = exe;
        InitializeComponent();
        TitleText.Text = $"Version {update.Version.ToString(3)} disponible (tu as la {AppIdentity.Version})";
    }

    private void OnNotes(object sender, RoutedEventArgs e)
    {
        // Page de release de ce dépôt seulement (vérifié à la lecture de la réponse de GitHub)
        Process.Start(new ProcessStartInfo(_update.PageUrl) { UseShellExecute = true });
    }

    private void OnLater(object sender, RoutedEventArgs e)
    {
        if (!_busy) DialogResult = false;
    }

    private async void OnInstall(object sender, RoutedEventArgs e)
    {
        if (_busy) return;
        SetBusy(true, "Téléchargement et vérification de l'empreinte…");
        try
        {
            var zip = await UpdateInstaller.DownloadAsync(_update.Zip);
            SetBusy(true, "Installation…");
            var outcome = await Task.Run(() => UpdateInstaller.Install(zip, _exe));
            if (!outcome.Done)
            {
                SetBusy(false, outcome.Error ?? "La mise à jour a échoué.", error: true);
                return;
            }
            InstalledExe = outcome.Exe;
            // Le mode occupé est levé avant de fermer, sinon OnClosing refuserait la fermeture
            _busy = false;
            DialogResult = true;
        }
        catch (Exception ex) when (ex is HttpRequestException or TaskCanceledException or IOException or InvalidDataException or UnauthorizedAccessException)
        {
            SetBusy(false, $"La mise à jour n'a pas été installée : {ex.Message}", error: true);
        }
    }

    private void SetBusy(bool busy, string text, bool error = false)
    {
        _busy = busy;
        InstallButton.IsEnabled = !busy;
        LaterButton.IsEnabled = !busy;
        StatusText.Text = text;
        StatusText.Foreground = (Brush)FindResource(error ? "Rouge" : "Cyan");
        StatusText.Visibility = Visibility.Visible;
    }

    protected override void OnClosing(CancelEventArgs e)
    {
        // Pas de fermeture au milieu du remplacement : les fichiers resteraient à moitié en place
        if (_busy) e.Cancel = true;
        base.OnClosing(e);
    }
}
