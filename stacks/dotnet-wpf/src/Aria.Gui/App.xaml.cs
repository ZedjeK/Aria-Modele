using System.Diagnostics;
using System.IO;
using System.Windows;
using System.Windows.Media;
using Aria.Core;
using Aria.Core.Data;
using Aria.Core.Install;
using Aria.Core.Startup;
using Aria.Core.Updates;
using Aria.Gui.Services;
using Aria.Gui.Views;
using Wpf.Ui.Appearance;

namespace Aria.Gui;

public partial class App : Application
{
    private TrayIcon? _tray;
    private MainWindow? _window;
    private bool _quitting;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);
        // Accent de WPF-UI (cases cochées, sélections) : le cyan, couleur de ce qui est actif ou choisi
        ApplicationAccentColorManager.Apply((Color)FindResource("CouleurCyan"), ApplicationTheme.Dark);
        // Rien ne s'arrête tout seul pendant le démarrage : la fenêtre d'installation peut s'ouvrir et se fermer
        // avant la fenêtre principale
        ShutdownMode = ShutdownMode.OnExplicitShutdown;

        // Déjà ouverte dans cette session : elle revient au premier plan, sans seconde fenêtre
        if (!SingleInstance.Claim())
        {
            SingleInstance.SignalRunning();
            Shutdown();
            return;
        }

        try
        {
            DataFolder.Ensure();
        }
        catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
        {
            MessageBox.Show(
                $"Le dossier de données {DataFolder.Root} n'a pas pu être créé : {ex.Message}",
                AppIdentity.Name, MessageBoxButton.OK, MessageBoxImage.Error);
            Shutdown(1);
            return;
        }

        var settings = AppSettingsStore.Load();
        // Restes d'un remplacement précédent, maintenant que l'ancienne version est arrêtée, et fichiers de travail
        // d'une mise à jour (zip téléchargé, extraction)
        SelfInstall.CleanUp();
        UpdateInstaller.CleanUp();

        // Lancé à l'ouverture de session : l'icône seule, la fenêtre reste rangée jusqu'à ce qu'on l'ouvre
        var silent = AppIdentity.RunsInBackground && e.Args.Contains(Autostart.SilentArgument, StringComparer.OrdinalIgnoreCase);

        // Exe lancé hors de programme\ (zip décompressé) : proposition de s'installer, avant toute autre fenêtre
        if (!silent && Environment.ProcessPath is { } self && SelfInstall.ShouldOffer(self, settings, e.Args))
        {
            switch (OfferInstall(self, ref settings))
            {
                case InstallPromptResult.Installed:
                case InstallPromptResult.Closed:
                    Shutdown();
                    return;
            }
        }

        // Le fond animé suit les réglages fond.* (actif, vitesse, nombre d'éléments)
        BackdropService.Set(settings.Background);
        var window = new MainWindow(settings);
        SingleInstance.Listen(window, window.BringToFront);

        if (AppIdentity.RunsInBackground)
        {
            // Fermer la fenêtre ne suffit pas à arrêter l'appli : elle reste dans son icône
            _tray = new TrayIcon(window.BringToFront, Quit);
            // Bloc « Aller à » du menu de l'icône. « Agir sans ouvrir » reste vide (donc masqué) : voir TrayIcon.AddAction
            // et TrayIcon.AddChoices.
            _tray.AddPage("Réglages", () =>
            {
                window.BringToFront();
                window.ShowSettingsTab();
            });
            // Pendant une action de la fenêtre, pastille cyan : l'appli agit
            window.WorkingChanged += text => _tray?.Update(s => s with { Working = text });
            window.Closing += (_, args) =>
            {
                if (_quitting) return;
                args.Cancel = true;
                window.Hide();
            };
        }
        else
        {
            MainWindow = window;
            ShutdownMode = ShutdownMode.OnMainWindowClose;
        }

        _window = window;
        if (!silent) window.Show();

        // Une requête vers GitHub, seulement si le réglage le permet ; aucune installation sans accord
        if (settings.CheckUpdates) _ = CheckForUpdateAsync();
    }

    private async Task CheckForUpdateAsync()
    {
        if (!Version.TryParse(AppIdentity.Version, out var current)) return;
        var update = await UpdateChecker.CheckAsync(current);
        if (update is null || _window is null) return;
        var label = update.Version.ToString(3);
        _window.ShowUpdateAvailable(label, () => OfferUpdate(update));
        _tray?.ShowUpdateAvailable(label, () => OfferUpdate(update));
    }

    /// <summary>
    /// Fenêtre de mise à jour. Installée : la nouvelle version est lancée là où tourne l'exe, et celle-ci s'arrête.
    /// </summary>
    private void OfferUpdate(UpdateInfo update)
    {
        if (Environment.ProcessPath is not { } exe || _window is null) return;
        _window.BringToFront();
        var dialog = new UpdateWindow(update, exe) { Owner = _window };
        if (dialog.ShowDialog() != true || dialog.InstalledExe is not { } installed) return;

        OperationJournal.Record("mise-a-jour",
            $"Version {AppIdentity.Version} remplacée par la {update.Version.ToString(3)} ; l'ancienne est gardée dans les sauvegardes");
        // La place d'exemplaire unique est rendue avant de lancer la nouvelle version, qui la prendra
        SingleInstance.Release();
        Process.Start(new ProcessStartInfo(installed) { UseShellExecute = false, WorkingDirectory = Path.GetDirectoryName(installed)! });
        Quit();
    }

    /// <summary>
    /// Fenêtre « Installer / Lancer sans installer ». Installée : l'appli se relance depuis programme\, et ce lancement
    /// s'arrête. « Ne plus me proposer » est enregistré dans les réglages.
    /// </summary>
    private static InstallPromptResult OfferInstall(string self, ref AppSettings settings)
    {
        var prompt = new InstallPromptWindow(self);
        prompt.ShowDialog();

        if (prompt.Result == InstallPromptResult.Installed && prompt.Outcome is { Exe: { } installed } outcome)
        {
            var shortcut = outcome.ShortcutDone ? ", raccourci du Bureau créé" : outcome.ShortcutError is null ? "" : ", raccourci non créé";
            OperationJournal.Record("installation", $"Version {AppIdentity.Version} installée dans {SelfInstall.Folder}{shortcut}");
            if (outcome.ShortcutError is { } error)
                MessageBox.Show($"L'appli est installée, mais le raccourci du Bureau n'a pas pu être créé : {error}",
                    AppIdentity.Name, MessageBoxButton.OK, MessageBoxImage.Warning);
            // La place d'exemplaire unique est rendue avant de lancer la version installée, qui la prendra
            SingleInstance.Release();
            Process.Start(new ProcessStartInfo(installed) { UseShellExecute = false, WorkingDirectory = SelfInstall.Folder });
            return InstallPromptResult.Installed;
        }

        if (prompt.Result == InstallPromptResult.RunHere && prompt.DontAskAgain)
        {
            settings = settings with { OfferInstall = false };
            try
            {
                AppSettingsStore.Save(settings);
            }
            catch (Exception ex) when (ex is IOException or UnauthorizedAccessException)
            {
                // Réglage non enregistré : la proposition reviendra au prochain lancement
            }
        }
        return prompt.Result;
    }

    /// <summary>« Quitter » du menu de l'icône : la seule façon d'arrêter une appli qui sert en arrière-plan.</summary>
    private void Quit()
    {
        _quitting = true;
        _tray?.Dispose();
        _tray = null;
        Shutdown();
    }

    protected override void OnExit(ExitEventArgs e)
    {
        _tray?.Dispose();
        base.OnExit(e);
    }
}
