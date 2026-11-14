using System.IO;
using System.Windows;
using Aria.Core;
using Aria.Core.Data;
using Aria.Gui.Services;

namespace Aria.Gui;

public partial class App : Application
{
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
        window.Show();
    }
}
