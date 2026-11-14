using System.IO;
using System.Windows;
using Aria.Core;
using Aria.Core.Data;
using Aria.Core.Startup;
using Aria.Gui.Services;

namespace Aria.Gui;

public partial class App : Application
{
    private TrayIcon? _tray;
    private bool _quitting;

    protected override void OnStartup(StartupEventArgs e)
    {
        base.OnStartup(e);

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

        var window = new MainWindow(AppSettingsStore.Load());
        SingleInstance.Listen(window, window.BringToFront);

        if (AppIdentity.RunsInBackground)
        {
            // Fermer la fenêtre ne suffit plus à arrêter l'appli : elle reste dans son icône
            ShutdownMode = ShutdownMode.OnExplicitShutdown;
            _tray = new TrayIcon(window.BringToFront, Quit);
            window.Closing += (_, args) =>
            {
                if (_quitting) return;
                args.Cancel = true;
                window.Hide();
            };
        }

        // Lancé à l'ouverture de session : l'icône seule, la fenêtre reste rangée jusqu'à ce qu'on l'ouvre
        var silent = AppIdentity.RunsInBackground && e.Args.Contains(Autostart.SilentArgument, StringComparer.OrdinalIgnoreCase);
        if (!silent) window.Show();
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
