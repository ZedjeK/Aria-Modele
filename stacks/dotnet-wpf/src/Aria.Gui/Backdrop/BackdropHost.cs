using System.ComponentModel;
using System.Windows;
using System.Windows.Controls;
using System.Windows.Threading;
using Aria.Core.Display;
using Aria.Gui.Services;

namespace Aria.Gui.Backdrop;

/// <summary>
/// Moteur du fond animé, commun à toutes les applis. À poser dans la zone principale seulement, jamais sous le menu
/// latéral, et sous les panneaux (opaques). Il applique <see cref="BackdropRules"/> :
///   coupé      fond.actif faux, ou Windows demande moins d'animations (relu dès que le réglage change)
///   en pause   fenêtre cachée (rangée dans l'icône) ou réduite
///   passage    nombre d'éléments tiré entre fond.minimum et fond.maximum, durée selon fond.vitesse
/// Le dessin d'un passage revient au motif (<see cref="Pattern"/>), propre à chaque appli.
/// </summary>
public sealed class BackdropHost : Canvas
{
    private readonly Random _random = new();
    private readonly DispatcherTimer _next;
    private Window? _window;
    private IBackdropPattern _pattern = new DriftingDots();

    public BackdropHost()
    {
        IsHitTestVisible = false;
        Focusable = false;
        ClipToBounds = true;
        _next = new DispatcherTimer(DispatcherPriority.Background, Dispatcher);
        // Fin d'un passage : les règles sont relues avant le suivant
        _next.Tick += (_, _) => Restart();

        Loaded += OnLoaded;
        Unloaded += OnUnloaded;
        SizeChanged += (_, _) => Restart();
    }

    /// <summary>Motif dessiné à chaque passage. Par défaut, des points qui traversent la zone.</summary>
    public IBackdropPattern Pattern
    {
        get => _pattern;
        set
        {
            ArgumentNullException.ThrowIfNull(value);
            Stop();
            _pattern = value;
            Restart();
        }
    }

    /// <summary>État du fond au dernier passage.</summary>
    public BackdropMode Mode { get; private set; } = BackdropMode.Off;

    private void OnLoaded(object sender, RoutedEventArgs e)
    {
        _window = Window.GetWindow(this);
        if (_window is not null)
        {
            _window.IsVisibleChanged += OnWindowVisibleChanged;
            _window.StateChanged += OnWindowStateChanged;
        }
        BackdropService.Changed += OnSettingsChanged;
        SystemParameters.StaticPropertyChanged += OnSystemChanged;
        Restart();
    }

    private void OnUnloaded(object sender, RoutedEventArgs e)
    {
        if (_window is not null)
        {
            _window.IsVisibleChanged -= OnWindowVisibleChanged;
            _window.StateChanged -= OnWindowStateChanged;
            _window = null;
        }
        BackdropService.Changed -= OnSettingsChanged;
        SystemParameters.StaticPropertyChanged -= OnSystemChanged;
        Stop();
        Mode = BackdropMode.Off;
    }

    private void OnWindowVisibleChanged(object sender, DependencyPropertyChangedEventArgs e) => Restart();
    private void OnWindowStateChanged(object? sender, EventArgs e) => Restart();
    private void OnSettingsChanged() => Dispatcher.InvokeAsync(Restart);

    /// <summary>« Effets d'animation » changé dans Windows : le fond se coupe ou reprend tout de suite.</summary>
    private void OnSystemChanged(object? sender, PropertyChangedEventArgs e)
    {
        if (e.PropertyName == nameof(SystemParameters.ClientAreaAnimation)) Dispatcher.InvokeAsync(Restart);
    }

    private void Stop()
    {
        _next.Stop();
        _pattern.Clear(this);
    }

    private void Restart()
    {
        Stop();
        var settings = BackdropService.Current;
        Mode = BackdropRules.Mode(
            settings,
            windowVisible: _window?.IsVisible == true,
            windowMinimized: _window?.WindowState == System.Windows.WindowState.Minimized,
            systemAnimations: SystemParameters.ClientAreaAnimation);
        if (Mode != BackdropMode.Running || ActualWidth < 1 || ActualHeight < 1) return;

        var duration = BackdropRules.PassDuration(settings.Speed);
        _pattern.BeginPass(this, BackdropRules.DrawCount(settings, _random), duration, _random);
        _next.Interval = duration;
        _next.Start();
    }
}
