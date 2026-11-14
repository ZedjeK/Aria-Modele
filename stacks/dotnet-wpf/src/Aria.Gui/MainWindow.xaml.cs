using System.IO;
using System.Windows;
using Aria.Core;
using Aria.Core.Data;
using Aria.Core.Startup;
using Aria.Gui.Services;
using Wpf.Ui.Controls;

namespace Aria.Gui;

public partial class MainWindow : FluentWindow
{
    /// <summary>Commentaire « À savoir » de chaque sous-onglet.</summary>
    private const string AboutOverview =
        "Page d'exemple du squelette, à remplacer par la première vraie page. Le bouton orange est la seule action " +
        "principale de la page ; le panneau qu'il remplit porte le liseré accentué.";
    private const string AboutSettings =
        "Chaque réglage est enregistré dès que tu le changes, dans reglages.json de ton dossier de données.";

    private Action? _showUpdate;
    private bool _loading = true;

    /// <summary>Réglages en cours (reglages.json).</summary>
    public AppSettings Settings { get; private set; }

    /// <summary>
    /// Ce que l'appli fait en ce moment (« mesure du disque en cours »), null quand elle a fini : l'icône de
    /// notification l'affiche en pastille cyan.
    /// </summary>
    public event Action<string?>? WorkingChanged;

    public MainWindow(AppSettings settings)
    {
        Settings = settings;
        InitializeComponent();
        NomTexte.Text = (AppIdentity.Name.StartsWith("Aria ", StringComparison.Ordinal) ? AppIdentity.Name[5..] : AppIdentity.Name).ToUpperInvariant();
        VersionTexte.Text = $"version {AppIdentity.Version}";
        DossierTexte.Text = DataFolder.Root;
        ASavoirTexte.Text = AboutOverview;

        DemarrageAuto.Visibility = AppIdentity.RunsInBackground ? Visibility.Visible : Visibility.Collapsed;
        // État relu du système à chaque retour sur la fenêtre : il a pu changer dans Windows ou dans le menu de l'icône
        Activated += (_, _) => DemarrageAuto.IsChecked = AppIdentity.RunsInBackground && Autostart.IsEnabled();

        ProposerInstallation.Visibility = AppIdentity.SelfInstalls ? Visibility.Visible : Visibility.Collapsed;
        foreach (var n in Enumerable.Range(BackgroundSettings.MinElements, BackgroundSettings.MaxElements - BackgroundSettings.MinElements + 1))
        {
            FondMinimum.Items.Add(n);
            FondMaximum.Items.Add(n);
        }
        ShowSettings(settings);
        _loading = false;
    }

    /// <summary>Une nouvelle version est disponible : le bandeau bleu apparaît.</summary>
    public void ShowUpdateAvailable(string version, Action show)
    {
        _showUpdate = show;
        MiseAJourTexte.Text = $"La version {version} est disponible.";
        MiseAJour.Visibility = Visibility.Visible;
    }

    /// <summary>Affiche le sous-onglet « Réglages » (bloc « Aller à » du menu de l'icône).</summary>
    public void ShowSettingsTab() => OngletReglages.IsChecked = true;

    private void OnShowUpdate(object sender, RoutedEventArgs e) => _showUpdate?.Invoke();

    private void OnSubTabChecked(object sender, RoutedEventArgs e)
    {
        // Appelé pendant InitializeComponent, avant que tout le contenu existe
        if (Apercu is null || Reglages is null || ASavoirTexte is null) return;
        var settingsTab = ReferenceEquals(sender, OngletReglages);
        Apercu.Visibility = settingsTab ? Visibility.Collapsed : Visibility.Visible;
        Reglages.Visibility = settingsTab ? Visibility.Visible : Visibility.Collapsed;
        ASavoirTexte.Text = settingsTab ? AboutSettings : AboutOverview;
    }

    /// <summary>
    /// Exemple d'action principale : mesure la place libre du disque qui porte le dossier de données. Lecture seule ;
    /// la jauge reste « à tester » tant que la mesure n'est pas faite.
    /// </summary>
    private async void OnMeasure(object sender, RoutedEventArgs e)
    {
        MesurerBouton.IsEnabled = false;
        WorkingChanged?.Invoke("mesure du disque en cours");
        try
        {
            var root = Path.GetPathRoot(DataFolder.Root) ?? DataFolder.Root;
            var (free, total) = await Task.Run(() =>
            {
                var drive = new DriveInfo(root);
                return (drive.AvailableFreeSpace, drive.TotalSize);
            });
            const double gigabyte = 1024d * 1024 * 1024;
            JaugeDisque.ValueText = $"{free / gigabyte:0.#} Go libres";
            JaugeDisque.Value = total > 0 ? (double)(total - free) / total : null;
            MesureTexte.Text = $"Disque {root} : {free / gigabyte:0.#} Go libres sur {total / gigabyte:0.#} Go, mesuré à {DateTime.Now:HH:mm:ss}.";
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException or ArgumentException)
        {
            JaugeDisque.Value = null;
            MesureTexte.Text = $"La mesure a échoué : {ex.Message}";
        }
        finally
        {
            WorkingChanged?.Invoke(null);
            MesurerBouton.IsEnabled = true;
        }
    }

    private void ShowSettings(AppSettings settings)
    {
        ProposerInstallation.IsChecked = settings.OfferInstall;
        ChercherMisesAJour.IsChecked = settings.CheckUpdates;
        var background = settings.Background;
        FondActif.IsChecked = background.Active;
        VitesseLente.IsChecked = background.Speed == BackgroundSpeed.Slow;
        VitesseNormale.IsChecked = background.Speed == BackgroundSpeed.Normal;
        VitesseRapide.IsChecked = background.Speed == BackgroundSpeed.Fast;
        FondMinimum.SelectedItem = background.Minimum;
        FondMaximum.SelectedItem = background.Maximum;
    }

    private void OnOfferInstallClick(object sender, RoutedEventArgs e) =>
        SaveSettings(Settings with { OfferInstall = ProposerInstallation.IsChecked == true });

    private void OnCheckUpdatesClick(object sender, RoutedEventArgs e) =>
        SaveSettings(Settings with { CheckUpdates = ChercherMisesAJour.IsChecked == true });

    /// <summary>Un réglage du fond a changé : enregistré, et le fond le suit tout de suite.</summary>
    private void OnBackgroundChanged(object sender, RoutedEventArgs e)
    {
        if (_loading) return;
        var speed = VitesseLente.IsChecked == true ? BackgroundSpeed.Slow
            : VitesseRapide.IsChecked == true ? BackgroundSpeed.Fast
            : BackgroundSpeed.Normal;
        var background = new BackgroundSettings(
            FondActif.IsChecked == true,
            speed,
            FondMinimum.SelectedItem as int? ?? Settings.Background.Minimum,
            FondMaximum.SelectedItem as int? ?? Settings.Background.Maximum);
        SaveSettings(Settings with { Background = background });
    }

    private void SaveSettings(AppSettings updated)
    {
        // Bornes remises dans l'ordre avant d'être affichées et appliquées
        updated = updated.Sanitized();
        try
        {
            AppSettingsStore.Save(updated);
            Settings = updated;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            System.Windows.MessageBox.Show($"Le réglage n'a pas pu être enregistré : {ex.Message}",
                AppIdentity.Name, System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Warning);
        }
        BackdropService.Set(Settings.Background);
        _loading = true;
        ShowSettings(Settings);
        _loading = false;
    }

    /// <summary>Ramène la fenêtre au premier plan : un second lancement, l'icône de notification.</summary>
    public void BringToFront()
    {
        if (!IsVisible) Show();
        if (WindowState == System.Windows.WindowState.Minimized) WindowState = System.Windows.WindowState.Normal;
        Activate();
        // Passer devant les autres fenêtres sans y rester
        Topmost = true;
        Topmost = false;
        Focus();
    }

    private void OnAutostartClick(object sender, RoutedEventArgs e)
    {
        try
        {
            if (DemarrageAuto.IsChecked == true && Environment.ProcessPath is { } exe)
            {
                Autostart.Enable(exe);
                OperationJournal.Record("demarrage-auto", "Démarrage avec Windows activé");
            }
            else
            {
                Autostart.Disable();
                OperationJournal.Record("demarrage-auto", "Démarrage avec Windows désactivé");
            }
        }
        catch (Exception ex) when (ex is UnauthorizedAccessException or System.Security.SecurityException or IOException)
        {
            System.Windows.MessageBox.Show($"Le démarrage avec Windows n'a pas pu être changé : {ex.Message}",
                AppIdentity.Name, System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Warning);
        }
        DemarrageAuto.IsChecked = Autostart.IsEnabled();
    }
}
