using System.Windows;
using System.Windows.Controls;
using Aria.Core.Display;

namespace Aria.Gui.Controls;

/// <summary>
/// Jauge d'une mesure, en segments. Sans valeur (mesure pas encore faite), elle reste vide et dit « à tester » :
/// jamais un chiffre supposé. Règle dans <see cref="Gauge"/> ; dessin : style implicite dans Theme\Aria.xaml.
/// </summary>
public class MeasureGauge : Control
{
    /// <summary>Nombre de segments de la jauge.</summary>
    public const int SegmentCount = 5;

    public static readonly DependencyProperty LabelProperty = DependencyProperty.Register(
        nameof(Label), typeof(string), typeof(MeasureGauge), new PropertyMetadata(""));

    public static readonly DependencyProperty ValueProperty = DependencyProperty.Register(
        nameof(Value), typeof(double?), typeof(MeasureGauge), new PropertyMetadata(null, OnMeasureChanged));

    public static readonly DependencyProperty ValueTextProperty = DependencyProperty.Register(
        nameof(ValueText), typeof(string), typeof(MeasureGauge), new PropertyMetadata(null, OnMeasureChanged));

    private static readonly DependencyPropertyKey SegmentsKey = DependencyProperty.RegisterReadOnly(
        nameof(Segments), typeof(IReadOnlyList<bool>), typeof(MeasureGauge), new PropertyMetadata(Array.Empty<bool>()));

    private static readonly DependencyPropertyKey DisplayTextKey = DependencyProperty.RegisterReadOnly(
        nameof(DisplayText), typeof(string), typeof(MeasureGauge), new PropertyMetadata(Gauge.NotMeasured));

    public static readonly DependencyProperty SegmentsProperty = SegmentsKey.DependencyProperty;
    public static readonly DependencyProperty DisplayTextProperty = DisplayTextKey.DependencyProperty;

    public MeasureGauge() => Refresh();

    /// <summary>Ce qui est mesuré (« Disque »).</summary>
    public string Label
    {
        get => (string)GetValue(LabelProperty);
        set => SetValue(LabelProperty, value);
    }

    /// <summary>Mesure ramenée entre 0 et 1 ; null tant qu'elle n'est pas faite.</summary>
    public double? Value
    {
        get => (double?)GetValue(ValueProperty);
        set => SetValue(ValueProperty, value);
    }

    /// <summary>Texte de la mesure (« 120 Go libres ») ; son pourcentage à défaut.</summary>
    public string? ValueText
    {
        get => (string?)GetValue(ValueTextProperty);
        set => SetValue(ValueTextProperty, value);
    }

    /// <summary>Segments, rempli ou non, de gauche à droite.</summary>
    public IReadOnlyList<bool> Segments => (IReadOnlyList<bool>)GetValue(SegmentsProperty);

    /// <summary>Texte affiché : celui de la mesure, ou « à tester ».</summary>
    public string DisplayText => (string)GetValue(DisplayTextProperty);

    private static void OnMeasureChanged(DependencyObject d, DependencyPropertyChangedEventArgs e) => ((MeasureGauge)d).Refresh();

    private void Refresh()
    {
        var filled = Gauge.Filled(Value, SegmentCount);
        SetValue(SegmentsKey, Enumerable.Range(0, SegmentCount).Select(i => i < filled).ToArray());
        SetValue(DisplayTextKey, Gauge.Text(Value, ValueText));
    }
}
