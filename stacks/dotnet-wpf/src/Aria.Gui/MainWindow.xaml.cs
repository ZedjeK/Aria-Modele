using System.IO;
using System.Windows;
using Aria.Core;
using Aria.Core.Data;
using Aria.Core.Startup;
using Wpf.Ui.Controls;

namespace Aria.Gui;

public partial class MainWindow : FluentWindow
{
    /// <summary>Réglages en cours (reglages.json).</summary>
    public AppSettings Settings { get; private set; }

    public MainWindow(AppSettings settings)
    {
        Settings = settings;
        InitializeComponent();
        VersionTexte.Text = $"version {AppIdentity.Version}";
        DossierTexte.Text = $"Données : {DataFolder.Root}";

        DemarrageAuto.Visibility = AppIdentity.RunsInBackground ? Visibility.Visible : Visibility.Collapsed;
        // État relu du système à chaque retour sur la fenêtre : il a pu changer dans Windows ou dans le menu de l'icône
        Activated += (_, _) => DemarrageAuto.IsChecked = AppIdentity.RunsInBackground && Autostart.IsEnabled();

        ProposerInstallation.Visibility = AppIdentity.SelfInstalls ? Visibility.Visible : Visibility.Collapsed;
        ProposerInstallation.IsChecked = settings.OfferInstall;
    }

    private void OnOfferInstallClick(object sender, RoutedEventArgs e)
    {
        var updated = Settings with { OfferInstall = ProposerInstallation.IsChecked == true };
        try
        {
            AppSettingsStore.Save(updated);
            Settings = updated;
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            System.Windows.MessageBox.Show($"Le réglage n'a pas pu être enregistré : {ex.Message}",
                AppIdentity.Name, System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Warning);
            ProposerInstallation.IsChecked = Settings.OfferInstall;
        }
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
