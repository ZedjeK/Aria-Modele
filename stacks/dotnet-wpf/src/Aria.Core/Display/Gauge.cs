namespace Aria.Core.Display;

/// <summary>
/// Jauge d'une mesure. Tant que la mesure n'est pas faite, la jauge reste vide et dit « à tester » : jamais un
/// chiffre supposé.
/// </summary>
public static class Gauge
{
    /// <summary>Texte d'une mesure pas encore faite.</summary>
    public const string NotMeasured = "à tester";

    /// <summary>Nombre de segments remplis : aucun sans mesure, au moins un dès qu'une mesure dépasse zéro.</summary>
    /// <param name="fraction">Mesure ramenée entre 0 et 1 ; null si elle n'est pas faite.</param>
    /// <param name="segments">Nombre de segments de la jauge.</param>
    public static int Filled(double? fraction, int segments)
    {
        if (fraction is not { } value || double.IsNaN(value) || segments <= 0) return 0;
        var clamped = Math.Clamp(value, 0, 1);
        return clamped == 0 ? 0 : Math.Max(1, (int)Math.Round(clamped * segments, MidpointRounding.AwayFromZero));
    }

    /// <summary>
    /// Texte affiché à côté de la jauge : « à tester » sans mesure ; sinon le texte de la mesure, ou à défaut son
    /// pourcentage.
    /// </summary>
    public static string Text(double? fraction, string? measured)
    {
        if (fraction is not { } value || double.IsNaN(value)) return NotMeasured;
        return string.IsNullOrWhiteSpace(measured) ? $"{Math.Round(Math.Clamp(value, 0, 1) * 100)} %" : measured.Trim();
    }
}
