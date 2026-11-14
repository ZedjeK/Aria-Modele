using Aria.Core.Data;
using Aria.Core.Display;

namespace Aria.Tests;

public class BackdropRulesTests
{
    private static readonly BackgroundSettings Actif = new(Active: true);

    [Fact]
    public void LeFondTourneQuandToutLePermet()
    {
        Assert.Equal(BackdropMode.Running, BackdropRules.Mode(Actif, windowVisible: true, windowMinimized: false, systemAnimations: true));
    }

    [Fact]
    public void LeFondEstCoupeSiLeReglageLeDesactive()
    {
        var inactif = new BackgroundSettings(Active: false);

        Assert.Equal(BackdropMode.Off, BackdropRules.Mode(inactif, windowVisible: true, windowMinimized: false, systemAnimations: true));
    }

    [Fact]
    public void LeFondEstCoupeSiWindowsDemandeMoinsDAnimations()
    {
        Assert.Equal(BackdropMode.Off, BackdropRules.Mode(Actif, windowVisible: true, windowMinimized: false, systemAnimations: false));
        // Coupé l'emporte sur en pause : rien ne doit reprendre au retour de la fenêtre
        Assert.Equal(BackdropMode.Off, BackdropRules.Mode(Actif, windowVisible: false, windowMinimized: false, systemAnimations: false));
    }

    [Fact]
    public void LeFondSeMetEnPauseFenetreCacheeOuReduite()
    {
        Assert.Equal(BackdropMode.Paused, BackdropRules.Mode(Actif, windowVisible: false, windowMinimized: false, systemAnimations: true));
        Assert.Equal(BackdropMode.Paused, BackdropRules.Mode(Actif, windowVisible: true, windowMinimized: true, systemAnimations: true));
    }

    [Fact]
    public void LaDureeDUnPassageSuitLaVitesse()
    {
        var lente = BackdropRules.PassDuration(BackgroundSpeed.Slow);
        var normale = BackdropRules.PassDuration(BackgroundSpeed.Normal);
        var rapide = BackdropRules.PassDuration(BackgroundSpeed.Fast);

        Assert.Equal(TimeSpan.FromSeconds(7), normale);
        Assert.True(lente > normale);
        Assert.True(rapide < normale);
        Assert.True(rapide > TimeSpan.Zero);
    }

    [Fact]
    public void UneVitesseInconnueDonneLaDureeNormale()
    {
        Assert.Equal(BackdropRules.PassDuration(BackgroundSpeed.Normal), BackdropRules.PassDuration((BackgroundSpeed)42));
    }

    [Fact]
    public void LeNombreDElementsResteDansLesBornesEtLesAtteint()
    {
        var settings = new BackgroundSettings(Minimum: 3, Maximum: 6);
        var random = new Random(1234);

        var tirages = Enumerable.Range(0, 2000).Select(_ => BackdropRules.DrawCount(settings, random)).ToList();

        Assert.All(tirages, n => Assert.InRange(n, 3, 6));
        Assert.Equal(new[] { 3, 4, 5, 6 }, tirages.Distinct().Order());
    }

    [Fact]
    public void DesBornesEgalesDonnentToujoursCeNombre()
    {
        var settings = new BackgroundSettings(Minimum: 4, Maximum: 4);
        var random = new Random(7);

        Assert.All(Enumerable.Range(0, 100), _ => Assert.Equal(4, BackdropRules.DrawCount(settings, random)));
    }

    [Fact]
    public void DesBornesHorsLimitesOuInverseesSontRamenees()
    {
        var random = new Random(99);

        var inversees = Enumerable.Range(0, 500).Select(_ => BackdropRules.DrawCount(new BackgroundSettings(Minimum: 8, Maximum: 2), random));
        var horsLimites = Enumerable.Range(0, 500).Select(_ => BackdropRules.DrawCount(new BackgroundSettings(Minimum: -5, Maximum: 40), random));

        Assert.All(inversees, n => Assert.InRange(n, 2, 8));
        Assert.All(horsLimites, n => Assert.InRange(n, BackgroundSettings.MinElements, BackgroundSettings.MaxElements));
    }
}

public class GaugeTests
{
    [Fact]
    public void SansMesureLaJaugeEstVideEtDitATester()
    {
        Assert.Equal(0, Gauge.Filled(null, 5));
        Assert.Equal("à tester", Gauge.Text(null, "42 %"));
        Assert.Equal(0, Gauge.Filled(double.NaN, 5));
        Assert.Equal("à tester", Gauge.Text(double.NaN, null));
    }

    [Fact]
    public void UneMesureRemplitLaJaugeEnProportion()
    {
        Assert.Equal(0, Gauge.Filled(0, 5));
        Assert.Equal(1, Gauge.Filled(0.01, 5));
        Assert.Equal(3, Gauge.Filled(0.5, 5));
        Assert.Equal(5, Gauge.Filled(1, 5));
        Assert.Equal(5, Gauge.Filled(3, 5));
        Assert.Equal(0, Gauge.Filled(-1, 5));
    }

    [Fact]
    public void UneMesureAfficheSonTexteOuSonPourcentage()
    {
        Assert.Equal("120 Go libres", Gauge.Text(0.4, " 120 Go libres "));
        Assert.Equal("40 %", Gauge.Text(0.4, null));
        Assert.Equal("0 %", Gauge.Text(0, ""));
    }
}
