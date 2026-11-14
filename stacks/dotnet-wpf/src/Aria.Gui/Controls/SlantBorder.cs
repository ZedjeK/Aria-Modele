using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;

namespace Aria.Gui.Controls;

/// <summary>
/// Forme en biais des boutons qui agissent : un parallélogramme dont le biais garde la même largeur quelle que soit
/// la taille du bouton (une forme étirée élargirait le biais et le liseré avec le texte).
/// </summary>
public sealed class SlantBorder : Decorator
{
    public static readonly DependencyProperty BackgroundProperty = DependencyProperty.Register(
        nameof(Background), typeof(Brush), typeof(SlantBorder),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty BorderBrushProperty = DependencyProperty.Register(
        nameof(BorderBrush), typeof(Brush), typeof(SlantBorder),
        new FrameworkPropertyMetadata(null, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty SlantProperty = DependencyProperty.Register(
        nameof(Slant), typeof(double), typeof(SlantBorder),
        new FrameworkPropertyMetadata(10.0, FrameworkPropertyMetadataOptions.AffectsRender));

    public static readonly DependencyProperty PaddingProperty = DependencyProperty.Register(
        nameof(Padding), typeof(Thickness), typeof(SlantBorder),
        new FrameworkPropertyMetadata(default(Thickness), FrameworkPropertyMetadataOptions.AffectsMeasure));

    /// <summary>Remplissage : orange pour l'action principale, panneau pour les autres.</summary>
    public Brush? Background { get => (Brush?)GetValue(BackgroundProperty); set => SetValue(BackgroundProperty, value); }

    /// <summary>Contour (1 px) : cyan pour une action secondaire, discret pour annuler ou fermer.</summary>
    public Brush? BorderBrush { get => (Brush?)GetValue(BorderBrushProperty); set => SetValue(BorderBrushProperty, value); }

    /// <summary>Décalage horizontal du biais, en haut à gauche et en bas à droite.</summary>
    public double Slant { get => (double)GetValue(SlantProperty); set => SetValue(SlantProperty, value); }

    /// <summary>Marge entre la forme et le contenu.</summary>
    public Thickness Padding { get => (Thickness)GetValue(PaddingProperty); set => SetValue(PaddingProperty, value); }

    protected override Size MeasureOverride(Size constraint)
    {
        var extra = new Size(Padding.Left + Padding.Right, Padding.Top + Padding.Bottom);
        if (Child is not { } child) return extra;
        child.Measure(new Size(Math.Max(0, constraint.Width - extra.Width), Math.Max(0, constraint.Height - extra.Height)));
        return new Size(child.DesiredSize.Width + extra.Width, child.DesiredSize.Height + extra.Height);
    }

    protected override Size ArrangeOverride(Size arrangeSize)
    {
        Child?.Arrange(new Rect(Padding.Left, Padding.Top,
            Math.Max(0, arrangeSize.Width - Padding.Left - Padding.Right),
            Math.Max(0, arrangeSize.Height - Padding.Top - Padding.Bottom)));
        return arrangeSize;
    }

    protected override void OnRender(DrawingContext drawingContext)
    {
        var width = ActualWidth;
        var height = ActualHeight;
        if (width <= 0 || height <= 0) return;

        const double h = 0.5;
        var slant = Math.Max(0, Math.Min(Slant, width / 3));
        var shape = new StreamGeometry();
        using (var context = shape.Open())
        {
            context.BeginFigure(new Point(slant, h), isFilled: true, isClosed: true);
            context.LineTo(new Point(width - h, h), isStroked: true, isSmoothJoin: false);
            context.LineTo(new Point(width - slant, height - h), isStroked: true, isSmoothJoin: false);
            context.LineTo(new Point(h, height - h), isStroked: true, isSmoothJoin: false);
        }
        shape.Freeze();
        drawingContext.DrawGeometry(Background, BorderBrush is { } brush ? new Pen(brush, 1) : null, shape);
    }
}
