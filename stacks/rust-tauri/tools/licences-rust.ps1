<#
.SYNOPSIS
Génère la partie Rust de LICENCES-TIERS.txt : les crates compilées dans l'exe, leur licence et le texte de chaque
licence, par cargo-about.

.DESCRIPTION
À lancer avant chaque version, et après toute mise à jour de Cargo.lock : .\tools\verifier-release.ps1 refuse une liste
qui ne correspond plus à Cargo.lock (son empreinte est écrite en tête de la partie générée).

Seule la partie entre les lignes « [généré par tools\licences-rust.ps1 : ne pas modifier à la main] » et « [fin de la
partie générée] » est remplacée ; le reste du fichier (polices, paquets npm, bibliothèque standard de Rust) s'écrit à la
main. La forme de la liste est dans about.hbs, les licences acceptées et la cible dans about.toml.

Il faut cargo-about, à la version que suit la CI :
    cargo install cargo-about --version 0.9.3 --locked

    .\tools\licences-rust.ps1
    .\tools\licences-rust.ps1 -WhatIf     affiche la liste sans toucher à LICENCES-TIERS.txt
#>
#Requires -Version 5.1
[CmdletBinding(SupportsShouldProcess)]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

# La version de cargo-about attendue : celle qu'installe la CI (taiki-e/install-action, dans les workflows).
$versionAttendue = '0.9.3'
$debut = '[généré par tools\licences-rust.ps1 : ne pas modifier à la main]'
$fin = '[fin de la partie générée]'

$racine = Split-Path -Parent $PSScriptRoot
$licences = Join-Path $racine 'LICENCES-TIERS.txt'
$verrou = Join-Path $racine 'Cargo.lock'
foreach ($fichier in $licences, $verrou) {
    if (-not (Test-Path -LiteralPath $fichier -PathType Leaf)) { throw "$fichier est introuvable." }
}

# L'empreinte de Cargo.lock, fins de ligne ramenées à LF : la même sous Windows et sous Linux, quelle que soit la
# conversion faite par git. verifier-release.ps1 calcule la même.
function Empreinte-Verrou([string]$chemin) {
    $texte = [IO.File]::ReadAllText($chemin, [Text.Encoding]::UTF8).Replace("`r`n", "`n")
    $sha = [Security.Cryptography.SHA256]::Create()
    try { $octets = $sha.ComputeHash([Text.Encoding]::UTF8.GetBytes($texte)) }
    finally { $sha.Dispose() }
    return -join ($octets | ForEach-Object { $_.ToString('x2') })
}

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) { throw 'cargo est introuvable : installe Rust (rustup).' }
# Sous PowerShell 5.1, la sortie d'erreur redirigée d'un programme natif arrêterait le script : on lit son code.
$ErrorActionPreference = 'Continue'
$version = "$(cargo about --version 2>$null)".Trim()
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($code -ne 0 -or $version -notmatch "(^|\s)$([regex]::Escape($versionAttendue))$") {
    throw "cargo-about $versionAttendue est introuvable (trouvé : « $version »). Installe-le : cargo install cargo-about --version $versionAttendue --locked"
}

$travail = Join-Path $racine '.work'
New-Item -ItemType Directory -Path $travail -Force | Out-Null
$sortie = Join-Path $travail 'licences-rust.txt'
Push-Location -LiteralPath $racine
try {
    $ErrorActionPreference = 'Continue'
    cargo about generate --workspace --locked --output-file $sortie about.hbs
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
}
finally { Pop-Location }
if ($code -ne 0) { throw "cargo about a échoué (code $code) : voir les messages ci-dessus." }

$liste = ([IO.File]::ReadAllText($sortie, [Text.Encoding]::UTF8).Trim() -split "`r?`n") -join "`r`n"
$empreinte = Empreinte-Verrou $verrou
$partie = @($debut, "Cargo.lock (SHA-256, fins de ligne LF) : $empreinte", '', $liste, $fin) -join "`r`n"

$texte = [IO.File]::ReadAllText($licences, [Text.Encoding]::UTF8)
$i = $texte.IndexOf($debut)
$j = $texte.IndexOf($fin)
if ($i -lt 0 -or $j -lt $i) {
    throw "LICENCES-TIERS.txt n'a pas les lignes « $debut » et « $fin » : remets-les là où va la liste des crates Rust."
}
$nouveau = $texte.Substring(0, $i) + $partie + $texte.Substring($j + $fin.Length)

if ($PSCmdlet.ShouldProcess($licences, 'Remplacer la liste des crates Rust')) {
    [IO.File]::WriteAllText($licences, $nouveau, (New-Object Text.UTF8Encoding $false))
    Write-Host "LICENCES-TIERS.txt : liste des crates Rust à jour (Cargo.lock $($empreinte.Substring(0, 12))…)."
}
else {
    Write-Host $liste
}
