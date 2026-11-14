using System.Windows;
using Aria.Core;
using Aria.Core.Data;
using Wpf.Ui.Controls;

namespace Aria.Gui;

public partial class MainWindow : FluentWindow
{
    /// <summary>Réglages lus au lancement (reglages.json).</summary>
    public AppSettings Settings { get; }

    public MainWindow(AppSettings settings)
    {
        Settings = settings;
        InitializeComponent();
        VersionTexte.Text = $"version {AppIdentity.Version}";
        DossierTexte.Text = $"Données : {DataFolder.Root}";
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
}
