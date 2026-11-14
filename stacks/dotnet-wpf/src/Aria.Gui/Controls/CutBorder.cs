using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;

namespace Aria.Gui.Controls;

/// <summary>Coins biseautés d'un <see cref="CutBorder"/>.</summary>
[Flags]
public enum CutCorners
{
    None = 0,
    TopLeft = 1,
    TopRight = 2,
    BottomRight = 4,
    BottomLeft = 8
}

/// <summary>
/// Cadre à coins coupés. Un Border ne sait dessiner que des coins arrondis ; celui-ci trace sa forme lui-même, fond
/// puis liseré, et place son contenu à l'intérieur comme le ferait un Border.
///   panneau       coin haut droit et coin bas gauche coupés (par défaut)
///   sous-onglet   coin haut droit seulement (l'encoche)
/// </summary>
public sealed class CutBorder : Decorator
{
    public static readonly DependencyProperty BackgroundProperty = DependencyProperty.Register(
        nameof(Background), typeof(Brush), typeof(CutBorder),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty BorderBrushProperty = DependencyProperty.Register(
        nameof(BorderBrush), typeof(Brush), typeof(CutBorder),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty StrokeProperty = DependencyProperty.Register(
        nameof(Stroke), typeof(double), typeof(CutBorder),
        new FrameworkPropertyMetadata(1.0, FrameworkPropertyMetadataOptions.AffectsMeasure | FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty StrokeOpacityProperty = DependencyProperty.Register(
        nameof(StrokeOpacity), typeof(double), typeof(CutBorder),
        new FrameworkPropertyMetadata(1.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty CutProperty = DependencyProperty.Register(
        nameof(Cut), typeof(double), typeof(CutBorder),
        new FrameworkPropertyMetadata(16.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty CornersProperty = DependencyProperty.Register(
        nameof(Corners), typeof(CutCorners), typeof(CutBorder),
        new FrameworkPropertyMetadata(CutCorners.TopRight | CutCorners.BottomLeft, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty PaddingProperty = DependencyProperty.Register(
        nameof(Padding), typeof(Thickness), typeof(CutBorder),
        new FrameworkPropertyMetadata(default(Thickness), FrameworkPropertyMetadataOptions.AffectsMeasure));

    /// <summary>Fond, opaque pour un panneau : le fond animé ne se voit pas au travers.</summary>
    public Brush? Background
    {
        get => (Brush?)GetValue(BackgroundProperty);
        set => SetValue(BackgroundProperty, value);
    }

    /// <summary>Couleur du liseré.</summary>
    public Brush? BorderBrush
    {
        get => (Brush?)GetValue(BorderBrushProperty);
        set => SetValue(BorderBrushProperty, value);
    }

    /// <summary>Épaisseur du liseré : 1 pour le liseré fin, 2 pour le liseré accentué.</summary>
    public double Stroke
    {
        get => (double)GetValue(StrokeProperty);
        set => SetValue(StrokeProperty, value);
    }

    /// <summary>Opacité du liseré : la même couleur sert au liseré fin (atténué) et au liseré accentué (franc).</summary>
    public double StrokeOpacity
    {
        get => (double)GetValue(StrokeOpacityProperty);
        set => SetValue(StrokeOpacityProperty, value);
    }

    /// <summary>Longueur du biseau, sur chaque côté d'un coin coupé.</summary>
    public double Cut
    {
        get => (double)GetValue(CutProperty);
        set => SetValue(CutProperty, value);
    }

    /// <summary>Coins coupés.</summary>
    public CutCorners Corners
    {
        get => (CutCorners)GetValue(CornersProperty);
        set => SetValue(CornersProperty, value);
    }

    /// <summary>Marge entre le liseré et le contenu.</summary>
    public Thickness Padding
    {
        get => (Thickness)GetValue(PaddingProperty);
        set => SetValue(PaddingProperty, value);
    }

    private double Edge => Math.Max(0, Stroke);

    protected override Size MeasureOverride(Size constraint)
    {
        var extra = new Size(Padding.Left + Padding.Right + 2 * Edge, Padding.Top + Padding.Bottom + 2 * Edge);
        if (Child is not { } child) return extra;
        child.Measure(new Size(Math.Max(0, constraint.Width - extra.Width), Math.Max(0, constraint.Height - extra.Height)));
        return new Size(child.DesiredSize.Width + extra.Width, child.DesiredSize.Height + extra.Height);
    }

    protected override Size ArrangeOverride(Size arrangeSize)
    {
        Child?.Arrange(new Rect(
            Padding.Left + Edge,
            Padding.Top + Edge,
            Math.Max(0, arrangeSize.Width - Padding.Left - Padding.Right - 2 * Edge),
            Math.Max(0, arrangeSize.Height - Padding.Top - Padding.Bottom - 2 * Edge)));
        return arrangeSize;
    }

    protected override void OnRender(DrawingContext drawingContext)
    {
        var width = ActualWidth;
        var height = ActualHeight;
        if (width <= 0 || height <= 0) return;

        // Le tracé passe au milieu du liseré, pour qu'il ne déborde pas du cadre
        var h = Edge / 2;
        var cut = Math.Max(0, Math.Min(Cut, Math.Min(width, height) / 2));
        var corners = Corners;
        var points = new List<Point>();
        void Corner(CutCorners corner, Point square, Point before, Point after)
        {
            if (corners.HasFlag(corner) && cut > 0) { points.Add(before); points.Add(after); }
            else points.Add(square);
        }
        Corner(CutCorners.TopLeft, new Point(h, h), new Point(h, cut), new Point(cut, h));
        Corner(CutCorners.TopRight, new Point(width - h, h), new Point(width - cut, h), new Point(width - h, cut));
        Corner(CutCorners.BottomRight, new Point(width - h, height - h), new Point(width - h, height - cut), new Point(width - cut, height - h));
        Corner(CutCorners.BottomLeft, new Point(h, height - h), new Point(cut, height - h), new Point(h, height - cut));

        var shape = new StreamGeometry();
        using (var context = shape.Open())
        {
            context.BeginFigure(points[0], isFilled: true, isClosed: true);
            for (var i = 1; i < points.Count; i++) context.LineTo(points[i], isStroked: true, isSmoothJoin: false);
        }
        shape.Freeze();

        if (Background is { } background) drawingContext.DrawGeometry(background, null, shape);
        if (BorderBrush is not { } brush || Edge <= 0) return;
        drawingContext.PushOpacity(Math.Clamp(StrokeOpacity, 0, 1));
        drawingContext.DrawGeometry(null, new Pen(brush, Edge), shape);
        drawingContext.Pop();
    }
}
