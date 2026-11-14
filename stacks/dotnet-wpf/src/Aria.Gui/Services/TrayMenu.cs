using System.Drawing;
using System.Drawing.Text;
using System.IO;
using System.Runtime.InteropServices;
using System.Windows.Forms;
using Aria.Core.Tray;

namespace Aria.Gui.Services;

// Dessin du menu de l'icône de notification, aux couleurs de l'appli. Le rendu standard de Windows Forms reprend
// les couleurs du système dans une appli qui n'est pas Windows Forms : tout est dessiné ici.

/// <summary>Palette de l'appli, en couleurs Windows Forms (voir Theme\Aria.xaml).</summary>
internal static class TrayPalette
{
    public static readonly Color Fond = Color.FromArgb(0x03, 0x08, 0x12);
    public static readonly Color Panneau = Color.FromArgb(0x06, 0x10, 0x1F);
    public static readonly Color Survol = Color.FromArgb(0x0B, 0x1A, 0x30);
    public static readonly Color Texte = Color.FromArgb(0xDD, 0xEB, 0xFF);
    public static readonly Color TexteSecondaire = Color.FromArgb(0x93, 0xA8, 0xC8);
    public static readonly Color Inactif = Color.FromArgb(0x5D, 0x70, 0x92);
    public static readonly Color PointRepos = Color.FromArgb(0x3A, 0x4D, 0x70);
    public static readonly Color Cyan = Color.FromArgb(0x7D, 0xF9, 0xFF);
    public static readonly Color Bleu = Color.FromArgb(0x5C, 0x8B, 0xFF);
    public static readonly Color Orange = Color.FromArgb(0xFF, 0x8A, 0x1F);
    public static readonly Color Ligne = Color.FromArgb(0x1A, 0x2D, 0x4D);

    /// <summary>Couleur de la pastille de l'icône ; null sans pastille.</summary>
    public static Color? Badge(TrayBadge badge) => badge switch
    {
        TrayBadge.Attention => Orange,
        TrayBadge.Working => Cyan,
        TrayBadge.Information => Bleu,
        _ => null
    };

    /// <summary>Couleur du point d'une ligne « État ».</summary>
    public static Color Dot(TrayTone tone) => tone switch
    {
        TrayTone.Attention => Orange,
        TrayTone.Active => Cyan,
        TrayTone.Information => Bleu,
        _ => PointRepos
    };

    /// <summary>Couleur du texte d'une ligne « État » : orange pour ce qui est à regarder, discret au repos.</summary>
    public static Color Text(TrayTone tone) => tone switch
    {
        TrayTone.Attention => Orange,
        TrayTone.Idle => TexteSecondaire,
        _ => Texte
    };
}

/// <summary>
/// Polices embarquées (Space Grotesk, IBM Plex Mono), chargées depuis les ressources de l'exe pour Windows Forms :
/// rien n'est installé dans Windows, elles ne servent qu'à ce processus. Illisibles, le menu prend la police du
/// système.
/// </summary>
internal static class TrayFonts
{
    private static readonly PrivateFontCollection Collection = new();
    private static readonly Dictionary<(string Family, float Size, bool Bold), Font> Cache = [];
    private static readonly FontFamily Regular, Bold, Mono;

    static TrayFonts()
    {
        foreach (var file in new[] { "SpaceGrotesk-Medium.ttf", "SpaceGrotesk-Bold.ttf", "IBMPlexMono-Regular.ttf" }) Load(file);
        var system = SystemFonts.MenuFont?.FontFamily ?? FontFamily.GenericSansSerif;
        Regular = Find("Space Grotesk Medium", FontStyle.Regular) ?? system;
        Bold = Find("Space Grotesk", FontStyle.Bold) ?? system;
        Mono = Find("IBM Plex Mono", FontStyle.Regular) ?? FontFamily.GenericMonospace;
    }

    /// <summary>Police de l'interface (Space Grotesk).</summary>
    public static Font Interface(float size, bool bold = false) => Get(bold ? Bold : Regular, size, bold);

    /// <summary>Police du code et des titres de bloc (IBM Plex Mono).</summary>
    public static Font Code(float size) => Get(Mono, size, false);

    private static Font Get(FontFamily family, float size, bool bold)
    {
        var key = (family.Name, size, bold);
        if (!Cache.TryGetValue(key, out var font))
        {
            var style = bold && family.IsStyleAvailable(FontStyle.Bold) ? FontStyle.Bold : FontStyle.Regular;
            Cache[key] = font = new Font(family, size, style);
        }
        return font;
    }

    private static FontFamily? Find(string name, FontStyle style) =>
        Collection.Families.FirstOrDefault(f => f.Name.Equals(name, StringComparison.OrdinalIgnoreCase) && f.IsStyleAvailable(style));

    private static void Load(string file)
    {
        try
        {
            var resource = System.Windows.Application.GetResourceStream(new Uri($"pack://application:,,,/Assets/Fonts/{file}"));
            if (resource is null) return;
            byte[] bytes;
            using (var stream = resource.Stream)
            using (var copy = new MemoryStream())
            {
                stream.CopyTo(copy);
                bytes = copy.ToArray();
            }
            // Mémoire gardée jusqu'à la fin du processus : les deux moteurs de texte la lisent
            var memory = Marshal.AllocCoTaskMem(bytes.Length);
            Marshal.Copy(bytes, 0, memory, bytes.Length);
            // GDI+ pour connaître la famille, GDI pour que TextRenderer dessine avec elle
            Collection.AddMemoryFont(memory, bytes.Length);
            uint count = 0;
            AddFontMemResourceEx(memory, (uint)bytes.Length, IntPtr.Zero, ref count);
        }
        catch (Exception e) when (e is IOException or ArgumentException or InvalidOperationException or ExternalException)
        {
            // Police illisible : celle du système la remplace
        }
    }

    [DllImport("gdi32.dll")]
    private static extern IntPtr AddFontMemResourceEx(IntPtr font, uint length, IntPtr reserved, ref uint fonts);
}

/// <summary>Une ligne dessinée ici : la largeur qu'il lui faut, mesurée depuis le bord gauche du menu.</summary>
internal interface ITrayRow
{
    int RequiredWidth { get; }
}

/// <summary>Lignes ordinaires du menu, et ce qui leur est commun.</summary>
internal static class TrayMenuItems
{
    /// <summary>Retrait du texte à gauche, le même pour toutes les lignes.</summary>
    public static int Inset => TrayFonts.Interface(9.5f).Height;

    /// <summary>Place laissée à droite (flèche d'un sous-menu).</summary>
    public static int RightSpace => TrayFonts.Interface(9.5f).Height * 3 / 2;

    /// <summary>Dessin d'une ligne de texte, coupée par « … » si la place manque.</summary>
    public const TextFormatFlags OneLine = TextFormatFlags.Left | TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine
        | TextFormatFlags.EndEllipsis | TextFormatFlags.NoPrefix | TextFormatFlags.NoPadding;

    /// <summary>Largeur d'un texte sur une ligne (sans « … » : la mesure doit donner toute la largeur).</summary>
    public static Size Measure(string text, Font font) =>
        TextRenderer.MeasureText(text, font, Size.Empty, TextFormatFlags.SingleLine | TextFormatFlags.NoPrefix | TextFormatFlags.NoPadding);

    /// <summary>Ligne cliquable ; sans action, c'est la ligne d'un sous-menu.</summary>
    public static ToolStripMenuItem Item(string text, Action? onClick)
    {
        var item = new ToolStripMenuItem(text)
        {
            ForeColor = TrayPalette.Texte,
            Font = TrayFonts.Interface(9.5f),
            Padding = new Padding(0, 5, RightSpace, 5)
        };
        if (onClick is not null) item.Click += (_, _) => onClick();
        return item;
    }

    /// <summary>
    /// Largeur du menu ou d'un volet. Windows Forms la calcule d'après le texte des lignes de menu (et de leurs
    /// raccourcis clavier) seulement : les lignes dessinées ici (titres, état, choix) seraient coupées. La place qui
    /// manque est réservée par un « raccourci » fait d'espaces insécables, que le dessin du menu n'affiche jamais.
    /// </summary>
    public static void FitWidth(ToolStrip strip)
    {
        var items = strip.Items.Cast<ToolStripItem>().ToList();
        var menuItems = items.OfType<ToolStripMenuItem>().ToList();
        if (menuItems.Count == 0) return;
        var needed = items.Select(i => i switch
        {
            ITrayRow row => row.RequiredWidth,
            ToolStripMenuItem menuItem => Inset + Measure(menuItem.Text ?? "", menuItem.Font).Width + RightSpace,
            _ => 0
        }).Max();
        var widest = menuItems.Max(m => Measure(m.Text ?? "", m.Font).Width);
        // Windows Forms ajoute au moins le retrait de gauche autour du texte : le reste est à réserver
        var missing = needed - widest - Inset;
        var filler = menuItems[0];
        filler.ShortcutKeyDisplayString = null;
        if (missing <= 0) return;
        var space = Math.Max(1, Measure(Filler, filler.Font).Width);
        filler.ShortcutKeyDisplayString = new string('\u00A0', (missing + space - 1) / space);
    }

    /// <summary>Espace insécable : mesuré par Windows Forms, jamais dessiné.</summary>
    private const string Filler = "\u00A0";

    /// <summary>Ferme le menu et ses volets avant que l'action ne s'exécute.</summary>
    public static void CloseMenus(ToolStripItem item)
    {
        var strip = item.Owner as ToolStripDropDown;
        while (strip is not null)
        {
            var ownerItem = strip.OwnerItem;
            strip.Close(ToolStripDropDownCloseReason.ItemClicked);
            strip = ownerItem?.Owner as ToolStripDropDown;
        }
    }

    /// <summary>
    /// Largeur visible d'une ligne : Windows Forms donne aux lignes de menu la largeur du volet avant d'en retirer
    /// la marge, leur bord droit est donc caché. Ce qui s'aligne à droite (« EN PLACE ») se cale sur la partie visible.
    /// </summary>
    public static int VisibleWidth(ToolStripItem item) =>
        item.Owner is { } owner ? Math.Clamp(owner.DisplayRectangle.Right - item.Bounds.X, 0, item.Width) : item.Width;

    public static void FillBackground(Graphics g, ToolStripItem item)
    {
        using var back = new SolidBrush(item.Enabled && item.Selected ? TrayPalette.Survol : TrayPalette.Panneau);
        g.FillRectangle(back, new Rectangle(Point.Empty, item.Size));
    }
}

/// <summary>Titre d'un bloc (« ÉTAT », « AGIR SANS OUVRIR », « ALLER À ») : petit, discret, non cliquable.</summary>
internal sealed class TrayBlockTitle : ToolStripLabel, ITrayRow
{
    public TrayBlockTitle(string text) : base(text)
    {
        Font = TrayFonts.Code(7f);
        ForeColor = TrayPalette.TexteSecondaire;
        Margin = new Padding(0, 4, 0, 0);
    }

    public int RequiredWidth => TrayMenuItems.Inset * 2 + TrayMenuItems.Measure(Text ?? "", Font).Width;

    public override Size GetPreferredSize(Size constrainingSize) =>
        new(RequiredWidth, TrayMenuItems.Measure(Text ?? "", Font).Height + 6);

    protected override void OnPaint(PaintEventArgs e)
    {
        // Windows Forms décale les lignes qui ne sont pas des lignes de menu : le texte est recalé sur les autres
        var left = Math.Max(0, TrayMenuItems.Inset - Bounds.X);
        TextRenderer.DrawText(e.Graphics, Text, Font, new Rectangle(left, 0, Math.Max(0, Width - left), Height), ForeColor, TrayMenuItems.OneLine);
    }
}

/// <summary>Ligne du bloc « État » : du texte, avec un point de couleur. Pas une commande.</summary>
internal sealed class TrayStateItem : ToolStripLabel, ITrayRow
{
    private readonly TrayTone _tone;

    public TrayStateItem(TrayStateLine line) : base(line.Text)
    {
        _tone = line.Tone;
        Font = TrayFonts.Code(8.5f);
        ForeColor = TrayPalette.Text(line.Tone);
    }

    private int DotSize => Math.Max(6, Font.Height * 9 / 20);

    public int RequiredWidth => TrayMenuItems.Inset * 2 + DotSize * 2 + TrayMenuItems.Measure(Text ?? "", Font).Width;

    public override Size GetPreferredSize(Size constrainingSize) =>
        new(RequiredWidth, TrayMenuItems.Measure(Text ?? "", Font).Height + 8);

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
        var dot = DotSize;
        var left = Math.Max(0, TrayMenuItems.Inset - Bounds.X);
        using (var fill = new SolidBrush(TrayPalette.Dot(_tone)))
            g.FillEllipse(fill, left, (Height - dot) / 2, dot, dot);
        left += dot * 2;
        TextRenderer.DrawText(g, Text, Font, new Rectangle(left, 0, Math.Max(0, Width - left), Height), ForeColor, TrayMenuItems.OneLine);
    }
}

/// <summary>« Démarrer avec Windows » : une case dessinée (pleine en cyan quand elle est cochée), puis le texte.</summary>
internal sealed class TrayCheckItem : ToolStripMenuItem, ITrayRow
{
    public TrayCheckItem(string text) : base(text)
    {
        Font = TrayFonts.Interface(9.5f);
        ForeColor = TrayPalette.Texte;
        Padding = new Padding(0, 5, TrayMenuItems.RightSpace, 5);
    }

    private int Box => Math.Max(10, Font.Height * 4 / 5);

    public int RequiredWidth => TrayMenuItems.Inset * 2 + Box + Box / 2 + TrayMenuItems.Measure(Text ?? "", Font).Width;

    protected override void OnClick(EventArgs e)
    {
        TrayMenuItems.CloseMenus(this);
        base.OnClick(e);
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        TrayMenuItems.FillBackground(g, this);
        var box = new Rectangle(TrayMenuItems.Inset, (Height - Box) / 2, Box, Box);
        if (Checked)
        {
            using var fill = new SolidBrush(TrayPalette.Cyan);
            g.FillRectangle(fill, box);
            g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;
            using var mark = new Pen(TrayPalette.Fond, Math.Max(2, Box / 7f));
            g.DrawLines(mark,
            [
                new PointF(box.Left + Box * 0.22f, box.Top + Box * 0.52f),
                new PointF(box.Left + Box * 0.42f, box.Top + Box * 0.72f),
                new PointF(box.Left + Box * 0.78f, box.Top + Box * 0.30f)
            ]);
        }
        else
        {
            using var edge = new Pen(TrayPalette.TexteSecondaire);
            g.DrawRectangle(edge, box.X, box.Y, box.Width - 1, box.Height - 1);
        }
        var left = box.Right + Box / 2;
        TextRenderer.DrawText(g, Text, Font, new Rectangle(left, 0, Math.Max(0, TrayMenuItems.VisibleWidth(this) - left - TrayMenuItems.Inset), Height),
            Enabled ? ForeColor : TrayPalette.Inactif, TrayMenuItems.OneLine);
    }
}

/// <summary>
/// Un choix d'un volet : bouton radio, nom, détail sur la ligne du dessous, et « EN PLACE » sur celui qui tourne
/// (bouton radio plein et nom en cyan).
/// </summary>
internal sealed class TrayChoiceItem : ToolStripMenuItem, ITrayRow
{
    private const string InPlaceTag = "EN PLACE";
    private readonly TrayChoice _choice;
    private readonly Font _detailFont = TrayFonts.Code(8f);
    private readonly Font _tagFont = TrayFonts.Code(7f);

    public TrayChoiceItem(TrayChoice choice) : base(choice.Name)
    {
        _choice = choice;
        Font = TrayFonts.Interface(9.5f);
        ForeColor = choice.InPlace ? TrayPalette.Cyan : TrayPalette.Texte;
        Enabled = choice.Enabled;
        // Deux lignes : la hauteur de la ligne de menu laisse la place au détail
        Padding = new Padding(0, Pad, TrayMenuItems.RightSpace, Pad + _detailFont.Height);
    }

    private int Radio => Math.Max(10, Font.Height * 3 / 4);
    private int Pad => Font.Height / 3;
    private int TagWidth => _choice.InPlace ? TrayMenuItems.Measure(InPlaceTag, _tagFont).Width + Pad * 2 : 0;

    public int RequiredWidth
    {
        get
        {
            var name = TrayMenuItems.Measure(_choice.Name, Font).Width;
            var detail = TrayMenuItems.Measure(_choice.Detail, _detailFont).Width;
            return TrayMenuItems.Inset * 2 + Radio + Radio / 2 + Math.Max(name, detail) + (TagWidth > 0 ? TagWidth + Pad * 2 : 0);
        }
    }

    protected override void OnClick(EventArgs e)
    {
        TrayMenuItems.CloseMenus(this);
        base.OnClick(e);
        _choice.Choose();
    }

    protected override void OnPaint(PaintEventArgs e)
    {
        var g = e.Graphics;
        TrayMenuItems.FillBackground(g, this);
        g.SmoothingMode = System.Drawing.Drawing2D.SmoothingMode.AntiAlias;

        var nameHeight = Font.Height;
        var top = Math.Max(0, (Height - nameHeight - _detailFont.Height) / 2);
        var radio = new Rectangle(TrayMenuItems.Inset, top + (nameHeight - Radio) / 2, Radio, Radio);
        using (var ring = new Pen(_choice.InPlace ? TrayPalette.Cyan : Enabled ? TrayPalette.TexteSecondaire : TrayPalette.Inactif))
            g.DrawEllipse(ring, radio.X, radio.Y, radio.Width - 1, radio.Height - 1);
        if (_choice.InPlace)
        {
            var inner = Radio * 2 / 5;
            using var fill = new SolidBrush(TrayPalette.Cyan);
            g.FillEllipse(fill, radio.X + (Radio - inner) / 2f - 0.5f, radio.Y + (Radio - inner) / 2f - 0.5f, inner, inner);
        }

        var left = radio.Right + Radio / 2;
        var visible = TrayMenuItems.VisibleWidth(this);
        var reserved = 0;
        if (_choice.InPlace)
        {
            var tagText = TrayMenuItems.Measure(InPlaceTag, _tagFont);
            var tag = new Rectangle(visible - TrayMenuItems.Inset - TagWidth, top + (nameHeight - tagText.Height - 4) / 2, TagWidth, tagText.Height + 4);
            using var edge = new Pen(TrayPalette.Cyan);
            g.DrawRectangle(edge, tag.X, tag.Y, tag.Width - 1, tag.Height - 1);
            TextRenderer.DrawText(g, InPlaceTag, _tagFont, tag, TrayPalette.Cyan,
                TextFormatFlags.HorizontalCenter | TextFormatFlags.VerticalCenter | TextFormatFlags.SingleLine | TextFormatFlags.NoPadding);
            reserved = TagWidth + Pad * 2;
        }

        var textWidth = Math.Max(0, visible - left - TrayMenuItems.Inset - reserved);
        TextRenderer.DrawText(g, _choice.Name, Font, new Rectangle(left, top, textWidth, nameHeight),
            Enabled ? ForeColor : TrayPalette.Inactif, TrayMenuItems.OneLine);
        TextRenderer.DrawText(g, _choice.Detail, _detailFont, new Rectangle(left, top + nameHeight, textWidth, _detailFont.Height),
            Enabled ? TrayPalette.TexteSecondaire : TrayPalette.Inactif, TrayMenuItems.OneLine);
    }
}

/// <summary>Dessin du menu et de ses volets : fond panneau, liseré cyan, survol bleu nuit, séparateurs discrets.</summary>
internal sealed class TrayMenuRenderer : ToolStripRenderer
{
    protected override void OnRenderToolStripBackground(ToolStripRenderEventArgs e)
    {
        using var back = new SolidBrush(TrayPalette.Panneau);
        e.Graphics.FillRectangle(back, e.AffectedBounds);
    }

    protected override void OnRenderToolStripBorder(ToolStripRenderEventArgs e)
    {
        using var border = new Pen(TrayPalette.Cyan);
        e.Graphics.DrawRectangle(border, 0, 0, e.ToolStrip.Width - 1, e.ToolStrip.Height - 1);
    }

    /// <summary>Ligne survolée, ou dont le volet est ouvert : bleu nuit, sur lequel le texte clair reste lisible.</summary>
    protected override void OnRenderMenuItemBackground(ToolStripItemRenderEventArgs e) =>
        TrayMenuItems.FillBackground(e.Graphics, e.Item);

    protected override void OnRenderItemText(ToolStripItemTextRenderEventArgs e)
    {
        // Le « raccourci » qui réserve la largeur (voir TrayMenuItems.FitWidth) ne se dessine pas
        if (e.Item is ToolStripMenuItem && !string.Equals(e.Text, e.Item.Text, StringComparison.Ordinal)) return;
        var color = e.Item.Enabled ? e.Item.ForeColor : TrayPalette.Inactif;
        if (e.Item is ToolStripMenuItem { IsOnDropDown: true })
        {
            // Même retrait que les lignes dessinées ici (titres, état, case, choix)
            var area = new Rectangle(TrayMenuItems.Inset, 0, Math.Max(0, TrayMenuItems.VisibleWidth(e.Item) - TrayMenuItems.Inset - TrayMenuItems.RightSpace), e.Item.Height);
            TextRenderer.DrawText(e.Graphics, e.Text, e.TextFont, area, color, TrayMenuItems.OneLine);
            return;
        }
        TextRenderer.DrawText(e.Graphics, e.Text, e.TextFont, e.TextRectangle, color, e.TextFormat);
    }

    protected override void OnRenderArrow(ToolStripArrowRenderEventArgs e)
    {
        e.ArrowColor = e.Item is { Enabled: false } ? TrayPalette.Inactif : TrayPalette.Texte;
        base.OnRenderArrow(e);
    }

    protected override void OnRenderSeparator(ToolStripSeparatorRenderEventArgs e)
    {
        using var line = new Pen(TrayPalette.Ligne);
        var y = e.Item.Height / 2;
        e.Graphics.DrawLine(line, 1, y, e.Item.Width - 2, y);
    }
}
