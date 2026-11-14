using System.Drawing;
using System.IO;
using System.Windows.Forms;
using Aria.Core;
using Aria.Core.Data;
using Aria.Core.Startup;

namespace Aria.Gui.Services;

/// <summary>
/// Icône de la zone de notification, pour une appli qui sert en arrière-plan (<see cref="AppIdentity.RunsInBackground"/>).
/// Menu : « Ouvrir » en gras, « État », « Démarrer avec Windows » avec sa case, « Quitter ». Le menu aux couleurs
/// Aria (blocs titrés, volets) vient avec l'habillage ; celui-ci est le menu Windows ordinaire.
/// </summary>
public sealed class TrayIcon : IDisposable
{
    private readonly NotifyIcon _icon;
    private readonly ToolStripMenuItem _state;
    private readonly ToolStripMenuItem _autostart;
    private readonly ToolStripMenuItem _update;
    private Action? _showUpdate;

    public TrayIcon(Action open, Action quit)
    {
        _state = new ToolStripMenuItem("État : prêt") { Enabled = false };
        // N'apparaît que si une nouvelle version est disponible
        _update = new ToolStripMenuItem("Voir la mise à jour", null, (_, _) => _showUpdate?.Invoke()) { Visible = false };
        _autostart = new ToolStripMenuItem("Démarrer avec Windows");
        _autostart.Click += (_, _) => ToggleAutostart();

        var openItem = new ToolStripMenuItem("Ouvrir", null, (_, _) => open());
        openItem.Font = new Font(openItem.Font, System.Drawing.FontStyle.Bold);

        var menu = new ContextMenuStrip();
        menu.Items.AddRange([
            openItem,
            _state,
            _update,
            new ToolStripSeparator(),
            _autostart,
            new ToolStripMenuItem("Quitter", null, (_, _) => quit())
        ]);
        // État relu du système à chaque ouverture du menu : il a pu changer dans Windows
        menu.Opening += (_, _) => _autostart.Checked = Autostart.IsEnabled();

        _icon = new NotifyIcon
        {
            Icon = LoadIcon(),
            Text = $"{AppIdentity.Name} : prêt",
            ContextMenuStrip = menu,
            Visible = true
        };
        _icon.MouseClick += (_, e) => { if (e.Button == MouseButtons.Left) open(); };
    }

    /// <summary>Change la ligne « État » et l'infobulle, qui disent la même chose en une ligne.</summary>
    public void SetState(string text)
    {
        _state.Text = $"État : {text}";
        // L'infobulle d'une icône de notification est limitée à 127 caractères
        var tip = $"{AppIdentity.Name} : {text}";
        _icon.Text = tip.Length > 127 ? tip[..127] : tip;
    }

    /// <summary>Une nouvelle version est disponible : « Voir la mise à jour » apparaît dans le menu.</summary>
    public void ShowUpdateAvailable(string version, Action show)
    {
        _showUpdate = show;
        _update.Text = $"Voir la mise à jour ({version})";
        _update.Visible = true;
        SetState($"mise à jour {version} disponible");
    }

    private static void ToggleAutostart()
    {
        try
        {
            if (Autostart.IsEnabled())
            {
                Autostart.Disable();
                OperationJournal.Record("demarrage-auto", "Démarrage avec Windows désactivé");
            }
            else if (Environment.ProcessPath is { } exe)
            {
                Autostart.Enable(exe);
                OperationJournal.Record("demarrage-auto", "Démarrage avec Windows activé");
            }
        }
        catch (Exception e) when (e is UnauthorizedAccessException or System.Security.SecurityException or IOException)
        {
            System.Windows.MessageBox.Show($"Le démarrage avec Windows n'a pas pu être changé : {e.Message}",
                AppIdentity.Name, System.Windows.MessageBoxButton.OK, System.Windows.MessageBoxImage.Warning);
        }
    }

    /// <summary>Le logo de l'appli, celui de l'exe ; l'icône par défaut de Windows s'il est illisible.</summary>
    private static Icon LoadIcon()
    {
        try
        {
            if (Environment.ProcessPath is { } exe && Icon.ExtractAssociatedIcon(exe) is { } icon) return icon;
        }
        catch (Exception e) when (e is ArgumentException or IOException)
        {
            // Exe illisible : icône par défaut
        }
        return SystemIcons.Application;
    }

    public void Dispose()
    {
        _icon.Visible = false;
        _icon.ContextMenuStrip?.Dispose();
        _icon.Dispose();
    }
}
