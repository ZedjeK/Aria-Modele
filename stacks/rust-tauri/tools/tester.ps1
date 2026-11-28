<#
.SYNOPSIS
Lance les tests d'Aria {{NOM}} : runtime Visual C++ lié dans l'exe, interface (svelte-check, vitest), puis Rust
(cargo deny, rustfmt, clippy, cargo test).

.DESCRIPTION
L'interface est compilée avant le Rust : l'exe l'embarque, et clippy compile l'appli telle qu'elle sera livrée.
Code de retour non nul au premier contrôle qui échoue.

    .\tools\tester.ps1
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = Split-Path -Parent $PSScriptRoot

# Le runtime Visual C++ est lié dans l'exe (build > windows > staticVCRuntime, vrai par défaut dans Tauri) : l'appli
# ne demande rien à installer. Ce contrôle refuse tout ce qui le désactiverait. Il ne lit que des fichiers : il passe
# avant la recherche des outils.
Write-Host '== Runtime Visual C++ lié dans l''exe'
$refus = New-Object Collections.Generic.List[string]
$conf = [IO.File]::ReadAllText((Join-Path $racine 'appli\tauri.conf.json'), [Text.Encoding]::UTF8) | ConvertFrom-Json
$windows = $null
$build = $conf.PSObject.Properties['build']
if ($build -and $null -ne $build.Value) {
    $propriete = $build.Value.PSObject.Properties['windows']
    if ($propriete) { $windows = $propriete.Value }
}
if ($null -ne $windows) {
    # Le nom de la configuration et ses deux autres écritures acceptées par Tauri.
    foreach ($nom in 'staticVCRuntime', 'static-vc-runtime', 'staticVcRuntime') {
        $valeur = $windows.PSObject.Properties[$nom]
        if ($valeur -and $valeur.Value -eq $false) { $refus.Add("appli\tauri.conf.json : build > windows > $nom vaut false") }
    }
}
if ([IO.File]::ReadAllText((Join-Path $racine 'appli\build.rs')) -match 'static_vc_runtime\s*\(\s*false\s*\)') {
    $refus.Add('appli\build.rs : static_vc_runtime(false)')
}
# La variable d'environnement STATIC_VCRUNTIME (dépréciée par Tauri, mais encore lue) : ni dans un script, ni dans
# un workflow, ni dans la configuration de cargo.
$extensions = '.ps1', '.cmd', '.bat', '.toml', '.yml', '.yaml', '.json'
$fichiers = @(foreach ($dossier in $racine, (Join-Path $racine 'tools'), (Join-Path $racine '.cargo')) {
        if (Test-Path -LiteralPath $dossier -PathType Container) { Get-ChildItem -LiteralPath $dossier -File }
    })
$github = Join-Path $racine '.github'
if (Test-Path -LiteralPath $github -PathType Container) { $fichiers += @(Get-ChildItem -LiteralPath $github -File -Recurse) }
foreach ($fichier in $fichiers) {
    # Ce script nomme la variable pour la chercher : il ne compte pas.
    if ($fichier.Extension -notin $extensions -or $fichier.FullName -eq $PSCommandPath) { continue }
    if ([IO.File]::ReadAllText($fichier.FullName) -match 'STATIC_VCRUNTIME') {
        $refus.Add("$($fichier.FullName.Substring($racine.Length + 1)) : variable STATIC_VCRUNTIME")
    }
}
if ($refus.Count -gt 0) {
    foreach ($ligne in $refus) { Write-Host "  $ligne" -ForegroundColor Red }
    Write-Host 'Échec : le runtime Visual C++ doit rester lié dans l''exe, pour que l''appli ne demande rien à installer.' -ForegroundColor Red
    exit 1
}

foreach ($outil in 'cargo', 'npm') {
    if (-not (Get-Command $outil -ErrorAction SilentlyContinue)) {
        throw "$outil est introuvable : installe Rust (rustup) et Node.js."
    }
}

# Les applis Aria se compilent pour Windows x64 avec MSVC (décision 0010). Un Rust installé en GNU échoue plus loin,
# sur un outil absent (dlltool) : on le dit tout de suite, avec la réparation.
if ($env:OS -eq 'Windows_NT') {
    $hoteRust = ((& rustc -vV) | Select-String '^host:\s*(\S+)').Matches | ForEach-Object { $_.Groups[1].Value }
    if ($hoteRust -and $hoteRust -notlike '*-windows-msvc') {
        throw ("Rust est installé pour $hoteRust ; les applis Aria demandent MSVC. Installe les Build Tools de " +
            "Visual Studio (composants VC.Tools.x86.x64 et un SDK Windows 11), puis : " +
            'rustup set default-host x86_64-pc-windows-msvc')
    }
}

# Lance un contrôle ; au premier échec, le dit et s'arrête avec un code non nul. Sous PowerShell 5.1, la sortie
# d'erreur redirigée d'un programme natif arrêterait le script : on lit son code de retour à la place.
function Controler([string]$quoi, [scriptblock]$commande) {
    Write-Host "== $quoi"
    $ErrorActionPreference = 'Continue'
    & $commande
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    if ($code -ne 0) {
        Write-Host "Échec : $quoi (code $code)." -ForegroundColor Red
        exit 1
    }
}

Push-Location -LiteralPath (Join-Path $racine 'interface')
try {
    Controler 'Paquets de l''interface' { npm ci --no-audit --no-fund }
    Controler 'Types et composants (svelte-check)' { npm run check }
    Controler 'Tests de l''interface (vitest)' { npm test }
    Controler 'Compilation de l''interface' { npm run build }
}
finally { Pop-Location }

Push-Location -LiteralPath $racine
try {
    # Licences, failles connues (RustSec), crates interdites, origine et doublons : règles dans deny.toml. La CI
    # installe cargo-deny à la version de ses workflows ; en local, sans lui, le contrôle est sauté et le dit.
    if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
        Controler 'Dépendances Rust (cargo deny)' { cargo deny --locked check }
    }
    elseif ($env:GITHUB_ACTIONS -eq 'true') {
        Write-Host 'Échec : cargo-deny est introuvable dans la CI (taiki-e/install-action, dans les workflows).' -ForegroundColor Red
        exit 1
    }
    else {
        Write-Warning 'cargo-deny est introuvable : contrôle des dépendances sauté. Installe-le : cargo install cargo-deny --version 0.20.2 --locked'
    }
    Controler 'Mise en forme du Rust (rustfmt)' { cargo fmt --all --check }
    Controler 'Contrôles du Rust (clippy)' { cargo clippy --workspace --all-targets --all-features --locked -- -D warnings }
    Controler 'Tests du Rust' { cargo test --workspace --all-features --locked }
}
finally { Pop-Location }

Write-Host "Tous les tests passent." -ForegroundColor Cyan
exit 0
