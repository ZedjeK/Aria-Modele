using System.IO;
using System.Runtime.InteropServices;
using Aria.Core;
using Aria.Core.Install;

namespace Aria.Gui;

/// <summary>
/// Point d'entrée : vérifie que l'exe a ses fichiers à côté de lui avant de démarrer WPF. Lancé depuis l'aperçu du
/// zip dans l'Explorateur, Windows n'extrait que l'exe ; sans ses bibliothèques natives, WPF ne démarrerait pas, et
/// sans un mot. Une boîte Windows simple, qui ne dépend pas de WPF, dit alors quoi faire.
/// </summary>
public static class Program
{
    /// <summary>Bibliothèque native de WPF, livrée à côté de l'exe (voir Aria.Gui.csproj).</summary>
    private const string WpfNative = "PresentationNative_cor3.dll";

    [STAThread]
    public static int Main()
    {
        if (WhyIncomplete() is { } message)
        {
            MessageBoxW(IntPtr.Zero, message, AppIdentity.Name, MbOk | MbIconWarning);
            return 2;
        }
        var app = new App();
        app.InitializeComponent();
        return app.Run();
    }

    /// <summary>Ce qui manque à côté de l'exe, en clair ; null si rien ne manque.</summary>
    private static string? WhyIncomplete()
    {
        // Lancé depuis bin\ (développement) : WPF vient du runtime installé, il n'y a rien à vérifier. Location est
        // vide exactement dans l'exe en un seul fichier : c'est ce qu'on veut savoir ici (d'où l'avertissement retiré).
#pragma warning disable IL3000
        if (!string.IsNullOrEmpty(typeof(Program).Assembly.Location)) return null;
#pragma warning restore IL3000

        var folder = AppContext.BaseDirectory;
        IReadOnlyList<string> missing;
        try
        {
            missing = ReleaseFiles.Read(folder) is { } names
                ? ReleaseFiles.Missing(folder, names.Where(n => !n.Equals(SelfInstall.ExeFileName, StringComparison.OrdinalIgnoreCase)))
                : File.Exists(Path.Combine(folder, WpfNative)) ? [] : [WpfNative];
        }
        catch (Exception e) when (e is IOException or UnauthorizedAccessException)
        {
            return null; // dossier illisible : WPF dira lui-même ce qui ne va pas
        }
        if (missing.Count == 0) return null;

        return $"Il manque des fichiers à côté de l'exe : {string.Join(", ", missing)}.\n\n"
             + "Décompresse d'abord le zip en entier : clic droit sur le zip, « Extraire tout », puis lance l'exe "
             + "depuis le dossier obtenu.\n\n"
             + "Lancé depuis l'aperçu du zip dans l'Explorateur, Windows n'extrait que l'exe, sans les fichiers livrés "
             + "à côté de lui.";
    }

    private const uint MbOk = 0x0;
    private const uint MbIconWarning = 0x30;

    [DllImport("user32.dll", CharSet = CharSet.Unicode)]
    private static extern int MessageBoxW(IntPtr owner, string text, string caption, uint type);
}
