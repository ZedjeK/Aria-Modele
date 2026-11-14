using System.Drawing;
using System.Drawing.Drawing2D;
using System.Drawing.Imaging;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using Aria.Core;
using Aria.Core.Data;
using Aria.Core.Startup;
using Aria.Core.Tray;

namespace Aria.Gui.Services;

/// <summary>Un choix d'un sous-menu de l'icône : bouton radio, nom, détail, et « EN PLACE » sur celui qui tourne.</summary>
/// <param name="Name">Nom du choix (« Équilibre »).</param>
/// <param name="Detail">Détail sur une ligne (« jusqu'à 80 % »).</param>
/// <param name="InPlace">C'est le choix qui tourne en ce moment.</param>
/// <param name="Choose">Ce que fait un clic.</param>
public sealed record TrayChoice(string Name, string Detail, bool InPlace, Action Choose)
{
    /// <summary>Faux : le choix est grisé (par exemple pendant une opération).</summary>
    public bool Enabled { get; init; } = true;
}

/// <summary>
/// Icône de la zone de notification, pour une appli qui sert en arrière-plan (<see cref="AppIdentity.RunsInBackground"/>).
///
///   icône      le logo de l'appli : délavé quand rien n'est actif, plein quand quelque chose l'est, et une seule
///              pastille à la fois (orange à regarder, cyan l'appli agit, bleue une information) ; règle dans
///              <see cref="TrayState"/>
///   infobulle  la même chose en une ligne
///   menu       aux couleurs de l'appli, en blocs titrés : « Ouvrir » en gras, « État », puis les blocs propres à
///              l'appli (« Agir sans ouvrir », « Aller à », masqués tant qu'ils sont vides), « Démarrer avec
///              Windows » avec sa case, « Quitter »
///
/// L'appli dit ce qui se passe par <see cref="State"/> ou <see cref="Update"/>, et remplit ses blocs avec
/// <see cref="AddAction"/>, <see cref="AddChoices"/> et <see cref="AddPage"/>. Le menu est refait à chaque
/// ouverture : il montre l'état du moment.
/// </summary>
public sealed class TrayIcon : IDisposable
{
    private const int IconSize = 32, DotSize = 13;

    private readonly NotifyIcon _icon;
    private readonly ContextMenuStrip _menu;
    private readonly TrayMenuRenderer _renderer = new();
    private readonly Action _open;
    private readonly Action _quit;
    private readonly List<(string Text, Action? Run, Func<IEnumerable<TrayChoice>>? Choices)> _actions = [];
    private readonly List<(string Text, Action Open)> _pages = [];
    private readonly Dictionary<(bool Faded, TrayBadge Badge), Icon> _icons = [];
    private readonly List<IntPtr> _iconHandles = [];
    private Bitmap? _logo;
    private TrayState _state = TrayState.Idle;
    private (string Version, Action Show)? _update;
    private bool _disposed;

    /// <param name="open">Ouvre la fenêtre.</param>
    /// <param name="quit">Arrête l'appli.</param>
    public TrayIcon(Action open, Action quit)
    {
        _open = open;
        _quit = quit;
        _menu = new ContextMenuStrip { Renderer = _renderer, ShowImageMargin = false, ShowCheckMargin = false };
        // Le menu est refait à chaque ouverture : état, choix en place et case relue du système
        _menu.Opening += (_, e) =>
        {
            BuildMenu();
            e.Cancel = false;
        };
        BuildMenu();

        _icon = new NotifyIcon { ContextMenuStrip = _menu, Visible = true };
        _icon.MouseClick += (_, e) => { if (e.Button == MouseButtons.Left) _open(); };
        Show();
    }

    /// <summary>Ce que montrent l'icône, l'infobulle et le bloc « État ». Le changer met tout à jour.</summary>
    public TrayState State
    {
        get => _state;
        set
        {
            _state = value ?? TrayState.Idle;
            Show();
        }
    }

    /// <summary>Change une partie de l'état : <c>tray.Update(s => s with { Working = "copie en cours" })</c>.</summary>
    public void Update(Func<TrayState, TrayState> change) => State = change(_state);

    /// <summary>Ajoute une action au bloc « Agir sans ouvrir ».</summary>
    public void AddAction(string text, Action run) => _actions.Add((text, run, null));

    /// <summary>
    /// Ajoute au bloc « Agir sans ouvrir » une ligne qui ouvre un volet de choix à côté d'elle. La liste est
    /// redemandée à chaque ouverture du menu. Par exemple :
    /// <code>
    /// tray.AddChoices("Mode", () =>
    /// [
    ///     new TrayChoice("Prudent", "plus lent, sans risque", InPlace: mode == "prudent", () => SetMode("prudent")),
    ///     new TrayChoice("Rapide", "tout de suite", InPlace: mode == "rapide", () => SetMode("rapide"))
    /// ]);
    /// </code>
    /// </summary>
    public void AddChoices(string text, Func<IEnumerable<TrayChoice>> choices) => _actions.Add((text, null, choices));

    /// <summary>Ajoute une page au bloc « Aller à » : un clic ouvre la fenêtre sur cette page.</summary>
    public void AddPage(string text, Action open) => _pages.Add((text, open));

    /// <summary>
    /// Une nouvelle version est disponible : pastille bleue (si rien de plus urgent), ligne d'état et « Voir la mise
    /// à jour » dans le menu.
    /// </summary>
    public void ShowUpdateAvailable(string version, Action show)
    {
        _update = (version, show);
        Update(s => s with { Information = $"mise à jour {version} disponible" });
    }

    private void Show()
    {
        if (_disposed || _icon is null) return;
        _icon.Text = _state.Tooltip(AppIdentity.Name);
        _icon.Icon = IconFor(_state.Faded, _state.Badge);
    }

    // ---------------------------------------------------------------- Menu

    private void BuildMenu()
    {
        foreach (var old in _menu.Items.Cast<ToolStripItem>().ToList()) old.Dispose();
        _menu.Items.Clear();

        var open = TrayMenuItems.Item($"Ouvrir {AppIdentity.Name}", _open);
        open.Font = TrayFonts.Interface(9.5f, bold: true);
        _menu.Items.Add(open);
        _menu.Items.Add(new ToolStripSeparator());

        _menu.Items.Add(new TrayBlockTitle("ÉTAT"));
        foreach (var line in _state.Lines()) _menu.Items.Add(new TrayStateItem(line));
        // Utile quand l'appli démarre rangée avec Windows : le bandeau de la fenêtre ne se voit pas
        if (_update is { } update) _menu.Items.Add(TrayMenuItems.Item("Voir la mise à jour", update.Show));

        if (_actions.Count > 0)
        {
            _menu.Items.Add(new ToolStripSeparator());
            _menu.Items.Add(new TrayBlockTitle("AGIR SANS OUVRIR"));
            foreach (var (text, run, choices) in _actions)
                _menu.Items.Add(choices is null ? TrayMenuItems.Item(text, run!) : Submenu(text, choices));
        }

        if (_pages.Count > 0)
        {
            _menu.Items.Add(new ToolStripSeparator());
            _menu.Items.Add(new TrayBlockTitle("ALLER À"));
            foreach (var (text, openPage) in _pages) _menu.Items.Add(TrayMenuItems.Item(text, openPage));
        }

        _menu.Items.Add(new ToolStripSeparator());
        // État relu du système à chaque ouverture : il a pu changer dans Windows ou dans la fenêtre
        var autostart = new TrayCheckItem("Démarrer avec Windows") { Checked = Autostart.IsEnabled() };
        autostart.Click += (_, _) => ToggleAutostart();
        _menu.Items.Add(autostart);
        _menu.Items.Add(TrayMenuItems.Item($"Quitter {AppIdentity.Name}", _quit));
        TrayMenuItems.FitWidth(_menu);
    }

    private ToolStripMenuItem Submenu(string text, Func<IEnumerable<TrayChoice>> choices)
    {
        var item = TrayMenuItems.Item(text, null);
        var dropDown = (ToolStripDropDownMenu)item.DropDown;
        dropDown.Renderer = _renderer;
        dropDown.ShowImageMargin = false;
        dropDown.ShowCheckMargin = false;
        dropDown.Items.Add(new TrayBlockTitle(text.ToUpperInvariant()));
        try
        {
            foreach (var choice in choices()) dropDown.Items.Add(new TrayChoiceItem(choice));
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException or InvalidOperationException)
        {
            // Liste illisible à cet instant : le volet le dit, le reste du menu reste utilisable
            dropDown.Items.Add(new TrayStateItem(new TrayStateLine(TrayTone.Attention, $"Choix illisibles : {e.Message}")));
        }
        TrayMenuItems.FitWidth(dropDown);
        return item;
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

    // ---------------------------------------------------------------- Dessin de l'icône

    /// <summary>Le logo dans l'état demandé, dessiné une fois puis gardé.</summary>
    private Icon IconFor(bool faded, TrayBadge badge)
    {
        if (_icons.TryGetValue((faded, badge), out var cached)) return cached;
        _logo ??= LoadLogo();
        if (_logo is null) return SystemIcons.Application;

        using var bitmap = new Bitmap(IconSize, IconSize, PixelFormat.Format32bppArgb);
        using (var g = Graphics.FromImage(bitmap))
        {
            g.SmoothingMode = SmoothingMode.AntiAlias;
            g.InterpolationMode = InterpolationMode.HighQualityBicubic;
            var area = new Rectangle(0, 0, IconSize, IconSize);
            if (faded)
            {
                // Rien d'actif : logo en gris, à moitié effacé
                using var attributes = new ImageAttributes();
                attributes.SetColorMatrix(new ColorMatrix(
                [
                    [0.30f, 0.30f, 0.30f, 0, 0],
                    [0.59f, 0.59f, 0.59f, 0, 0],
                    [0.11f, 0.11f, 0.11f, 0, 0],
                    [0, 0, 0, 0.55f, 0],
                    [0, 0, 0, 0, 1]
                ]));
                g.DrawImage(_logo, area, 0, 0, _logo.Width, _logo.Height, GraphicsUnit.Pixel, attributes);
            }
            else g.DrawImage(_logo, area);

            if (TrayPalette.Badge(badge) is { } color)
            {
                const int x = IconSize - DotSize, y = IconSize - DotSize;
                // Anneau sombre autour de la pastille : elle se détache du logo
                using var edge = new SolidBrush(TrayPalette.Fond);
                using var fill = new SolidBrush(color);
                g.FillEllipse(edge, x - 1, y - 1, DotSize + 1, DotSize + 1);
                g.FillEllipse(fill, x + 1, y + 1, DotSize - 3, DotSize - 3);
            }
        }

        var handle = bitmap.GetHicon();
        _iconHandles.Add(handle);
        var icon = Icon.FromHandle(handle);
        _icons[(faded, badge)] = icon;
        return icon;
    }

    /// <summary>Le logo de l'appli (Assets\Aria-Modele.ico, embarqué) en 32 px ; null s'il est illisible.</summary>
    private static Bitmap? LoadLogo()
    {
        try
        {
            var resource = System.Windows.Application.GetResourceStream(new Uri("pack://application:,,,/Assets/Aria-Modele.ico"));
            if (resource is null) return null;
            using var stream = resource.Stream;
            using var icon = new Icon(stream, IconSize, IconSize);
            return icon.ToBitmap();
        }
        catch (Exception e) when (e is IOException or ArgumentException or InvalidOperationException)
        {
            return null;
        }
    }

    public void Dispose()
    {
        if (_disposed) return;
        _disposed = true;
        // Sans ce retrait, l'icône reste affichée jusqu'au passage de la souris
        _icon.Visible = false;
        _icon.ContextMenuStrip = null;
        _menu.Dispose();
        _icon.Dispose();
        foreach (var icon in _icons.Values) icon.Dispose();
        foreach (var handle in _iconHandles) DestroyIcon(handle);
        _logo?.Dispose();
    }

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool DestroyIcon(IntPtr handle);
}
