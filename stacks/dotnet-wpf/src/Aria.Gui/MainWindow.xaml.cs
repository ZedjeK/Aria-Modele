using Aria.Core;
using Wpf.Ui.Controls;

namespace Aria.Gui;

public partial class MainWindow : FluentWindow
{
    public MainWindow()
    {
        InitializeComponent();
        VersionTexte.Text = $"version {AppIdentity.Version}";
        DossierTexte.Text = $"Données : {AppIdentity.DataFolder}";
    }
}
