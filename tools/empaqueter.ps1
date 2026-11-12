<#
.SYNOPSIS
Fait le zip de la version et son empreinte, à partir de ce que build.ps1 a posé dans .work\sortie\.

.DESCRIPTION
Produit, dans .work\release\ :
  Aria-{{NOM}}-<version>.zip          dossier racine Aria-{{NOM}}-<version>\ : l'exe, LICENCES-TIERS.txt, BUILDINFO.txt
  Aria-{{NOM}}-<version>.zip.sha256   « empreinte  nom du fichier »

Commun à toutes les stacks. La version vient de tools\version.ps1.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = Split-Path -Parent $PSScriptRoot
$nom = 'Aria-{{NOM}}'
$version = "$(& (Join-Path $PSScriptRoot 'version.ps1') | Select-Object -Last 1)".Trim()
if ($version -notmatch '^\d+\.\d+\.\d+$') { throw "Version inattendue : « $version ». Attendu : X.Y.Z." }

$sortie = Join-Path $racine '.work\sortie'
if (-not (Test-Path -LiteralPath $sortie) -or -not (Get-ChildItem -LiteralPath $sortie -Force)) {
    throw "Rien à empaqueter : .work\sortie\ est vide. Lance d'abord .\build.ps1."
}

$release = Join-Path $racine '.work\release'
if (Test-Path -LiteralPath $release) { Remove-Item -LiteralPath $release -Recurse -Force }
$dossier = Join-Path $release "$nom-$version"
New-Item -ItemType Directory -Path $dossier | Out-Null

Get-ChildItem -LiteralPath $sortie -Force | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $dossier -Recurse -Force
}
Copy-Item -LiteralPath (Join-Path $racine 'LICENCES-TIERS.txt') -Destination $dossier -Force

# BUILDINFO.txt : de quoi retrouver d'où vient ce zip. La stack peut ajouter les versions de ses outils dans .work\outils.txt.
# Sous PowerShell 5.1, une sortie d'erreur de git redirigée arrêterait le script : on la tolère le temps de l'appel.
$ErrorActionPreference = 'Continue'
$commit = "$(git -C $racine rev-parse HEAD 2>$null)".Trim()
$ErrorActionPreference = 'Stop'
$lignes = @(
    "$nom $version",
    "Commit : $commit",
    "Date   : $((Get-Date).ToUniversalTime().ToString('yyyy-MM-dd HH:mm')) UTC"
)
$outils = Join-Path $racine '.work\outils.txt'
if (Test-Path -LiteralPath $outils) { $lignes += '', 'Outils :'; $lignes += Get-Content -LiteralPath $outils -Encoding UTF8 }
[IO.File]::WriteAllLines((Join-Path $dossier 'BUILDINFO.txt'), [string[]]$lignes, (New-Object Text.UTF8Encoding($false)))

$zip = Join-Path $release "$nom-$version.zip"
Compress-Archive -LiteralPath $dossier -DestinationPath $zip
$empreinte = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant()
[IO.File]::WriteAllText("$zip.sha256", "$empreinte  $nom-$version.zip`n", (New-Object Text.UTF8Encoding($false)))

Write-Host "Zip       : $zip"
Write-Host "Empreinte : $empreinte"
