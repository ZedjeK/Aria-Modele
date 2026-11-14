using Aria.Core.Data;

namespace Aria.Core.Display;

/// <summary>Ce que fait le fond animé à un instant donné.</summary>
public enum BackdropMode
{
    /// <summary>Coupé : désactivé dans les réglages, ou Windows demande moins d'animations. Rien n'est dessiné.</summary>
    Off,
    /// <summary>En pause : la fenêtre est cachée ou réduite. Il reprend quand elle revient.</summary>
    Paused,
    /// <summary>Il tourne.</summary>
    Running
}

/// <summary>
/// Règles du fond animé, communes à toutes les applis : le moteur de l'interface les applique, le motif (propre à
/// chaque appli) ne fait que dessiner un passage.
/// </summary>
public static class BackdropRules
{
    /// <summary>
    /// Coupé si le réglage fond.actif est faux ou si Windows demande moins d'animations ; en pause si la fenêtre est
    /// cachée ou réduite ; sinon, il tourne.
    /// </summary>
    /// <param name="settings">Réglages du fond (reglages.json).</param>
    /// <param name="windowVisible">La fenêtre est affichée (pas rangée dans l'icône).</param>
    /// <param name="windowMinimized">La fenêtre est réduite dans la barre des tâches.</param>
    /// <param name="systemAnimations">Réglage Windows « Effets d'animation » (SystemParameters.ClientAreaAnimation).</param>
    public static BackdropMode Mode(BackgroundSettings settings, bool windowVisible, bool windowMinimized, bool systemAnimations)
    {
        if (!settings.Active || !systemAnimations) return BackdropMode.Off;
        return windowVisible && !windowMinimized ? BackdropMode.Running : BackdropMode.Paused;
    }

    /// <summary>Durée d'un passage selon fond.vitesse : 14 s, 7 s ou 3,5 s.</summary>
    public static TimeSpan PassDuration(BackgroundSpeed speed) => TimeSpan.FromSeconds(speed switch
    {
        BackgroundSpeed.Slow => 14,
        BackgroundSpeed.Fast => 3.5,
        _ => 7
    });

    /// <summary>
    /// Nombre d'éléments d'un passage, tiré au hasard entre fond.minimum et fond.maximum compris (bornes ramenées entre
    /// 1 et 12 et remises dans l'ordre si besoin).
    /// </summary>
    public static int DrawCount(BackgroundSettings settings, Random random)
    {
        var bounds = settings.Sanitized();
        return random.Next(bounds.Minimum, bounds.Maximum + 1);
    }
}
