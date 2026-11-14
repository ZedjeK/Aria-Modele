using Aria.Core.Tray;

namespace Aria.Tests;

public class TrayStateTests
{
    [Fact]
    public void RienDActifIconeDelaveeSansPastille()
    {
        var state = TrayState.Idle;

        Assert.True(state.Faded);
        Assert.Equal(TrayBadge.None, state.Badge);
        Assert.Equal(new[] { new TrayStateLine(TrayTone.Idle, "Rien d'actif") }, state.Lines());
        Assert.Equal("Aria Essai · rien d'actif", state.Tooltip("Aria Essai"));
    }

    [Fact]
    public void LOrangePasseAvantLeCyanQuiPasseAvantLeBleu()
    {
        var tout = new TrayState { Attention = "à regarder", Working = "en cours", Information = "info" };

        Assert.Equal(TrayBadge.Attention, tout.Badge);
        Assert.Equal(TrayBadge.Working, (tout with { Attention = null }).Badge);
        Assert.Equal(TrayBadge.Information, (tout with { Attention = null, Working = null }).Badge);
        Assert.Equal(TrayBadge.None, (tout with { Attention = null, Working = null, Information = null }).Badge);
    }

    [Fact]
    public void UnActifSeulRendLIconePleineSansPastille()
    {
        var state = new TrayState { Active = "surveillance en place" };

        Assert.False(state.Faded);
        Assert.Equal(TrayBadge.None, state.Badge);
    }

    [Fact]
    public void UneInformationSeuleLaisseLIconeDelavee()
    {
        var state = new TrayState { Information = "mise à jour 0.2.0 disponible" };

        Assert.True(state.Faded);
        Assert.Equal(TrayBadge.Information, state.Badge);
    }

    [Fact]
    public void AgirOuAlerterRendLIconePleine()
    {
        Assert.False(new TrayState { Working = "copie en cours" }.Faded);
        Assert.False(new TrayState { Attention = "réglage perdu" }.Faded);
    }

    [Fact]
    public void UnTexteVideCompteCommeRien()
    {
        var state = new TrayState { Attention = "  ", Working = "", Information = "\t" };

        Assert.Equal(TrayBadge.None, state.Badge);
        Assert.True(state.Faded);
    }

    [Fact]
    public void LesLignesDEtatSuiventLaPrioriteAvecLeurCouleur()
    {
        var state = new TrayState { Active = "surveillance en place", Working = "copie en cours", Attention = "réglage perdu", Information = "mise à jour 0.2.0 disponible" };

        Assert.Equal(
        new[]
        {
            new TrayStateLine(TrayTone.Attention, "Réglage perdu"),
            new TrayStateLine(TrayTone.Active, "Copie en cours"),
            new TrayStateLine(TrayTone.Active, "Surveillance en place"),
            new TrayStateLine(TrayTone.Information, "Mise à jour 0.2.0 disponible")
        }, state.Lines());
    }

    [Fact]
    public void LInfobulleDitLaMemeChoseEnUneLigne()
    {
        var state = new TrayState { Working = "copie\nen cours", Information = "mise à jour 0.2.0 disponible" };

        Assert.Equal("Aria Essai · copie en cours · mise à jour 0.2.0 disponible", state.Tooltip("Aria Essai"));
    }

    [Fact]
    public void LInfobulleNeDepassePas127Caracteres()
    {
        var state = new TrayState { Information = new string('x', 300) };

        var tip = state.Tooltip("Aria Essai");

        Assert.Equal(TrayState.MaxTooltip, tip.Length);
        Assert.EndsWith("…", tip);
    }
}
