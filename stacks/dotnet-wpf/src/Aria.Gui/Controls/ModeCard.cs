using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;

namespace Aria.Gui.Controls;

/// <summary>
/// Carte à onglet : un choix de mode qui se coche. Le nom du mode est dans l'onglet (<see cref="Header"/>), qui se
/// remplit en cyan quand le mode est choisi ; le contenu décrit le mode. Un bouton radio : les cartes d'un même
/// GroupName s'excluent. Forme et couleurs : style implicite dans Theme\Aria.xaml.
/// </summary>
public class ModeCard : RadioButton
{
    public static readonly DependencyProperty HeaderProperty = DependencyProperty.Register(
        nameof(Header), typeof(object), typeof(ModeCard), new PropertyMetadata(null));

    /// <summary>Nom du mode, affiché dans l'onglet.</summary>
    public object? Header
    {
        get => GetValue(HeaderProperty);
        set => SetValue(HeaderProperty, value);
    }
}

/// <summary>
/// Contour d'une <see cref="ModeCard"/> : l'onglet en haut à gauche (bord droit en biais), le corps en dessous, coin
/// bas droit coupé. Un seul tracé pour l'onglet et le corps ; l'onglet a son propre remplissage.
/// </summary>
public sealed class TabCardFrame : FrameworkElement
{
    public static readonly DependencyProperty BackgroundProperty = DependencyProperty.Register(
        nameof(Background), typeof(Brush), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty TabFillProperty = DependencyProperty.Register(
        nameof(TabFill), typeof(Brush), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty BorderBrushProperty = DependencyProperty.Register(
        nameof(BorderBrush), typeof(Brush), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty StrokeProperty = DependencyProperty.Register(
        nameof(Stroke), typeof(double), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(1.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty StrokeOpacityProperty = DependencyProperty.Register(
        nameof(StrokeOpacity), typeof(double), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(1.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty TabWidthProperty = DependencyProperty.Register(
        nameof(TabWidth), typeof(double), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(120.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty TabHeightProperty = DependencyProperty.Register(
        nameof(TabHeight), typeof(double), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(32.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty CutProperty = DependencyProperty.Register(
        nameof(Cut), typeof(double), typeof(TabCardFrame),
        new FrameworkPropertyMetadata(18.0, FrameworkPropertyMetadataOptions.AffectsRender));

    /// <summary>Fond du corps.</summary>
    public Brush? Background { get => (Brush?)GetValue(BackgroundProperty); set => SetValue(BackgroundProperty, value); }

    /// <summary>Remplissage de l'onglet : cyan quand le mode est choisi.</summary>
    public Brush? TabFill { get => (Brush?)GetValue(TabFillProperty); set => SetValue(TabFillProperty, value); }

    /// <summary>Couleur du liseré.</summary>
    public Brush? BorderBrush { get => (Brush?)GetValue(BorderBrushProperty); set => SetValue(BorderBrushProperty, value); }

    /// <summary>Épaisseur du liseré.</summary>
    public double Stroke { get => (double)GetValue(StrokeProperty); set => SetValue(StrokeProperty, value); }

    /// <summary>Opacité du liseré.</summary>
    public double StrokeOpacity { get => (double)GetValue(StrokeOpacityProperty); set => SetValue(StrokeOpacityProperty, value); }

    /// <summary>Largeur de l'onglet à sa base, biais compris (celle du nom et de ses marges).</summary>
    public double TabWidth { get => (double)GetValue(TabWidthProperty); set => SetValue(TabWidthProperty, value); }

    /// <summary>Hauteur de l'onglet ; c'est aussi la largeur du biais.</summary>
    public double TabHeight { get => (double)GetValue(TabHeightProperty); set => SetValue(TabHeightProperty, value); }

    /// <summary>Longueur du biseau du coin bas droit.</summary>
    public double Cut { get => (double)GetValue(CutProperty); set => SetValue(CutProperty, value); }

    protected override void OnRender(DrawingContext drawingContext)
    {
        var width = ActualWidth;
        var height = ActualHeight;
        var tabHeight = Math.Min(TabHeight, height / 2);
        if (width <= 0 || height <= tabHeight) return;

        var h = Math.Max(0, Stroke) / 2;
        var tabBase = Math.Clamp(TabWidth, tabHeight, width);
        var tabTop = tabBase - tabHeight;
        var cut = Math.Max(0, Math.Min(Cut, Math.Min(width, height - tabHeight) / 2));

        var outline = Polygon(
            new Point(h, h), new Point(tabTop, h), new Point(tabBase, tabHeight), new Point(width - h, tabHeight),
            new Point(width - h, height - cut), new Point(width - cut, height - h), new Point(h, height - h));
        var tab = Polygon(new Point(h, h), new Point(tabTop, h), new Point(tabBase, tabHeight), new Point(h, tabHeight));

        if (Background is { } background) drawingContext.DrawGeometry(background, null, outline);
        if (TabFill is { } fill) drawingContext.DrawGeometry(fill, null, tab);
        if (BorderBrush is not { } brush || Stroke <= 0) return;
        drawingContext.PushOpacity(Math.Clamp(StrokeOpacity, 0, 1));
        drawingContext.DrawGeometry(null, new Pen(brush, Stroke), outline);
        drawingContext.Pop();
    }

    private static StreamGeometry Polygon(params Point[] points)
    {
        var geometry = new StreamGeometry();
        using (var context = geometry.Open())
        {
            context.BeginFigure(points[0], isFilled: true, isClosed: true);
            for (var i = 1; i < points.Length; i++) context.LineTo(points[i], isStroked: true, isSmoothJoin: false);
        }
        geometry.Freeze();
        return geometry;
    }
}
