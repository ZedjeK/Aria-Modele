using System.Runtime.InteropServices;
using System.Windows;
using System.Windows.Interop;
using Aria.Core;

namespace Aria.Gui.Services;

/// <summary>
/// Une seule fenêtre de l'appli par session Windows. Relancer l'exe ramène la fenêtre déjà ouverte au premier plan,
/// au lieu d'en ouvrir une seconde.
/// </summary>
public static class SingleInstance
{
    private static readonly string MutexName = $@"Local\{AppIdentity.ExeName}.Instance";
    private static readonly uint ActivateMessage = RegisterWindowMessage($"{AppIdentity.ExeName}.Activate");

    private static Mutex? _mutex;

    /// <summary>Devient l'exemplaire de la session. Faux si un autre l'est déjà.</summary>
    public static bool Claim()
    {
        _mutex = new Mutex(true, MutexName, out var created);
        if (created) return true;
        _mutex.Dispose();
        _mutex = null;
        return false;
    }

    /// <summary>
    /// Rend la place, juste avant de lancer une autre version qui la prendra (mise à jour). À appeler sur le fil qui
    /// a appelé <see cref="Claim"/>.
    /// </summary>
    public static void Release()
    {
        if (_mutex is null) return;
        try
        {
            _mutex.ReleaseMutex();
        }
        catch (ApplicationException)
        {
            // Pas tenu par ce fil : le fermer suffit à le rendre
        }
        _mutex.Dispose();
        _mutex = null;
    }

    /// <summary>Demande à l'exemplaire ouvert de revenir au premier plan.</summary>
    public static void SignalRunning()
    {
        AllowSetForegroundWindow(AnyProcess);   // l'exemplaire ouvert a le droit de passer devant
        PostMessage(BroadcastHandle, ActivateMessage, IntPtr.Zero, IntPtr.Zero);
    }

    /// <summary>Écoute les lancements suivants : <paramref name="onRequest"/> ramène la fenêtre.</summary>
    public static void Listen(Window window, Action onRequest)
    {
        var handle = new WindowInteropHelper(window).EnsureHandle();
        HwndSource.FromHwnd(handle)!.AddHook((IntPtr _, int msg, IntPtr _, IntPtr _, ref bool handled) =>
        {
            if (msg != ActivateMessage) return IntPtr.Zero;
            onRequest();
            handled = true;
            return IntPtr.Zero;
        });
    }

    private static readonly IntPtr BroadcastHandle = new(0xFFFF);
    private const int AnyProcess = -1;

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern uint RegisterWindowMessage(string name);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool PostMessage(IntPtr window, uint message, IntPtr wParam, IntPtr lParam);

    [DllImport("user32.dll")]
    [return: MarshalAs(UnmanagedType.Bool)]
    private static extern bool AllowSetForegroundWindow(int processId);
}
