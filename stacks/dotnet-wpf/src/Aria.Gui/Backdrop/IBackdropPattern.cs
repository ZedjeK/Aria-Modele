using System.Windows.Controls;

namespace Aria.Gui.Backdrop;

/// <summary>
/// Motif du fond animé : ce que l'appli dessine à chaque passage. Il dit ce que fait l'appli ; chaque appli remplace
/// <see cref="DriftingDots"/> par le sien. Le motif ne décide de rien : quand tourner, combien d'éléments et à
/// quelle vitesse, c'est <see cref="BackdropHost"/> qui le décide, d'après les réglages et Windows.
/// </summary>
public interface IBackdropPattern
{
    /// <summary>
    /// Pose et anime un passage : <paramref name="count"/> éléments placés au hasard, animés pendant
    /// <paramref name="duration"/> au plus, en couleurs de la palette. La zone a déjà sa taille (ActualWidth,
    /// ActualHeight) ; le motif n'y ajoute rien d'interactif.
    /// </summary>
    void BeginPass(Canvas area, int count, TimeSpan duration, Random random);

    /// <summary>Retire tout ce que le motif a posé dans la zone (fin de passage, pause, coupure, nouvelle taille).</summary>
    void Clear(Canvas area);
}
