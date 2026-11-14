namespace Aria.Core.Tray;

/// <summary>Pastille de l'icône de notification. Une seule à la fois : la plus haute valeur l'emporte.</summary>
public enum TrayBadge
{
    /// <summary>Pas de pastille.</summary>
    None = 0,
    /// <summary>Bleue : une information, rien d'urgent (par exemple une mise à jour disponible).</summary>
    Information = 1,
    /// <summary>Cyan : l'appli agit en ce moment.</summary>
    Working = 2,
    /// <summary>Orange : à regarder.</summary>
    Attention = 3
}

/// <summary>Couleur du point devant une ligne « État » du menu de l'icône.</summary>
public enum TrayTone
{
    /// <summary>Gris : rien d'actif.</summary>
    Idle,
    /// <summary>Cyan : actif, ou en cours.</summary>
    Active,
    /// <summary>Orange : à regarder.</summary>
    Attention,
    /// <summary>Bleu : une information.</summary>
    Information
}

/// <summary>Une ligne du bloc « État » : du texte, avec un point de couleur.</summary>
public sealed record TrayStateLine(TrayTone Tone, string Text);

/// <summary>
/// Ce que montrent l'icône de notification, son infobulle et le bloc « État » de son menu. Chaque champ est un texte
/// court en minuscules (« surveillance en place »), ou null quand il n'y a rien à dire.
///
///   icône      délavée quand rien n'est actif, pleine dès que quelque chose l'est (<see cref="Faded"/>)
///   pastille   une seule, par priorité : orange (à regarder), cyan (agit en ce moment), bleue (information)
///   infobulle  la même chose en une ligne : « Aria Nom · mise à jour 0.2.0 disponible »
/// </summary>
public sealed record TrayState
{
    /// <summary>Longueur maximale de l'infobulle d'une icône de notification (limite de Windows).</summary>
    public const int MaxTooltip = 127;

    /// <summary>Rien d'actif, rien à signaler.</summary>
    public static TrayState Idle { get; } = new();

    /// <summary>Ce qui est actif (icône pleine, sans pastille) ; null si rien ne l'est.</summary>
    public string? Active { get; init; }

    /// <summary>Ce que l'appli fait en ce moment (pastille cyan) ; null sinon.</summary>
    public string? Working { get; init; }

    /// <summary>Ce qui est à regarder (pastille orange) ; null sinon.</summary>
    public string? Attention { get; init; }

    /// <summary>Une information (pastille bleue), par exemple une mise à jour disponible ; null sinon.</summary>
    public string? Information { get; init; }

    /// <summary>La pastille affichée : orange, puis cyan, puis bleue.</summary>
    public TrayBadge Badge =>
        Has(Attention) ? TrayBadge.Attention
        : Has(Working) ? TrayBadge.Working
        : Has(Information) ? TrayBadge.Information
        : TrayBadge.None;

    /// <summary>Icône délavée : rien n'est actif. Une simple information ne la rend pas pleine.</summary>
    public bool Faded => !Has(Active) && !Has(Working) && !Has(Attention);

    /// <summary>Lignes du bloc « État », dans l'ordre de priorité ; « Rien d'actif » s'il n'y a rien.</summary>
    public IReadOnlyList<TrayStateLine> Lines()
    {
        var lines = new List<TrayStateLine>();
        if (Has(Attention)) lines.Add(new(TrayTone.Attention, Sentence(Attention!)));
        if (Has(Working)) lines.Add(new(TrayTone.Active, Sentence(Working!)));
        if (Has(Active)) lines.Add(new(TrayTone.Active, Sentence(Active!)));
        if (Has(Information)) lines.Add(new(TrayTone.Information, Sentence(Information!)));
        if (lines.Count == 0) lines.Add(new(TrayTone.Idle, "Rien d'actif"));
        return lines;
    }

    /// <summary>Infobulle : le nom de l'appli puis les mêmes informations, en une ligne de 127 caractères au plus.</summary>
    public string Tooltip(string appName)
    {
        string[] parts = [.. new[] { Attention, Working, Active, Information }.Where(Has).Select(p => OneLine(p!))];
        var tip = $"{appName} · {(parts.Length == 0 ? "rien d'actif" : string.Join(" · ", parts))}";
        return tip.Length <= MaxTooltip ? tip : tip[..(MaxTooltip - 1)] + "…";
    }

    private static bool Has(string? text) => !string.IsNullOrWhiteSpace(text);

    private static string OneLine(string text) => string.Join(' ', text.Split((char[]?)null, StringSplitOptions.RemoveEmptyEntries));

    private static string Sentence(string text)
    {
        var line = OneLine(text);
        return char.ToUpperInvariant(line[0]) + line[1..];
    }
}
