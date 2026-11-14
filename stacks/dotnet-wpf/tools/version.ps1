# Affiche la version sur sa dernière ligne, sans le « v ». Elle vit dans Directory.Build.props, et nulle part ailleurs.
Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$props = Join-Path (Split-Path -Parent $PSScriptRoot) 'Directory.Build.props'
$noeud = ([xml](Get-Content -LiteralPath $props -Raw -Encoding UTF8)).SelectSingleNode('/Project/PropertyGroup/Version')
if ($null -eq $noeud -or $noeud.InnerText.Trim() -notmatch '^\d+\.\d+\.\d+$') {
    throw "Version introuvable ou mal formée dans Directory.Build.props (attendu : X.Y.Z)."
}
$noeud.InnerText.Trim()
