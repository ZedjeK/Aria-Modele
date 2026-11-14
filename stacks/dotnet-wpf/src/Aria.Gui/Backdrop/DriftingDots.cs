using System.Windows;
using System.Windows.Controls;
using System.Windows.Media;
using System.Windows.Media.Animation;
using System.Windows.Shapes;

namespace Aria.Gui.Backdrop;

/// <summary>
/// Motif générique du squelette : des points cyan et bleus traversent la zone de gauche à droite, à une hauteur et
/// un départ tirés au hasard à chaque passage. À remplacer par le motif de l'appli (il dit ce qu'elle fait).
/// </summary>
public sealed class DriftingDots : IBackdropPattern
{
    private const double Edge = 24;

    // Couleurs de la palette : cyan et bleu, discrets sous les panneaux
    private static readonly Color[] Palette = [Color.FromRgb(0x7D, 0xF9, 0xFF), Color.FromRgb(0x5C, 0x8B, 0xFF)];

    private readonly List<UIElement> _placed = [];

    public void BeginPass(Canvas area, int count, TimeSpan duration, Random random)
    {
        var width = area.ActualWidth;
        var height = area.ActualHeight;
        for (var i = 0; i < count; i++)
        {
            var size = 5 + random.NextDouble() * 5;
            var color = Palette[random.Next(Palette.Length)];
            var dot = new Ellipse
            {
                Width = size,
                Height = size,
                Opacity = 0,
                Fill = new SolidColorBrush(color),
                Effect = new System.Windows.Media.Effects.DropShadowEffect { BlurRadius = 12, ShadowDepth = 0, Opacity = 0.7, Color = color }
            };
            Canvas.SetLeft(dot, -size);
            Canvas.SetTop(dot, Edge + random.NextDouble() * Math.Max(0, height - 2 * Edge));
            area.Children.Add(dot);
            _placed.Add(dot);

            // Chaque point traverse en 60 à 100 % du passage, et part assez tôt pour arriver avant la fin
            var travel = TimeSpan.FromTicks((long)(duration.Ticks * (0.6 + 0.4 * random.NextDouble())));
            var start = TimeSpan.FromTicks((long)((duration - travel).Ticks * random.NextDouble()));

            var move = new DoubleAnimation(-size, width, new Duration(travel)) { BeginTime = start, FillBehavior = FillBehavior.Stop };
            var fade = new DoubleAnimationUsingKeyFrames { BeginTime = start, FillBehavior = FillBehavior.Stop };
            fade.KeyFrames.Add(new LinearDoubleKeyFrame(0.8, KeyTime.FromTimeSpan(TimeSpan.FromTicks(travel.Ticks / 6))));
            fade.KeyFrames.Add(new LinearDoubleKeyFrame(0.8, KeyTime.FromTimeSpan(TimeSpan.FromTicks(travel.Ticks * 5 / 6))));
            fade.KeyFrames.Add(new LinearDoubleKeyFrame(0, KeyTime.FromTimeSpan(travel)));
            dot.BeginAnimation(Canvas.LeftProperty, move);
            dot.BeginAnimation(UIElement.OpacityProperty, fade);
        }
    }

    public void Clear(Canvas area)
    {
        foreach (var element in _placed)
        {
            element.BeginAnimation(Canvas.LeftProperty, null);
            element.BeginAnimation(UIElement.OpacityProperty, null);
            area.Children.Remove(element);
        }
        _placed.Clear();
    }
}
