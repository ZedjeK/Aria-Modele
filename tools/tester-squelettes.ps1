<#
.SYNOPSIS
Tests des squelettes de stack du dépôt modèle : init.ps1, compilation et zip, dans un dossier jetable.

.DESCRIPTION
Fichier du dépôt modèle seulement : init.ps1 le supprime dans l'appli créée, avec .github\workflows\squelettes.yml.

Pour chaque dossier de stacks\ qui a un _socle\stack.json (ou seulement celui donné par -Stack) :
  1. init.ps1 crée une appli « Essai » dans une copie jetable du dépôt, et lance les tests de la stack ;
  2. build.ps1 compile l'exe ; tools\empaqueter.ps1 en fait le zip et son empreinte ;
  3. le zip contient l'exe, BUILDINFO.txt et LICENCES-TIERS.txt ; tools\version.ps1 répond X.Y.Z.

Il faut les outils de chaque stack (le SDK .NET 10 pour dotnet-wpf). Tous les squelettes tournent ; le code de
retour est non nul si un seul contrôle est manqué.

    .\tools\tester-squelettes.ps1
    .\tools\tester-squelettes.ps1 -Stack dotnet-wpf
    .\tools\tester-squelettes.ps1 -Stack dotnet-wpf -Garder    garde l'appli jetable pour la lancer à la main
#>
[CmdletBinding()]
param(
    [string]$Stack,
    [switch]$Garder
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = Split-Path -Parent $PSScriptRoot
$dossierStacks = Join-Path $racine 'stacks'
if (-not (Test-Path -LiteralPath $dossierStacks -PathType Container)) {
    throw "stacks\ est introuvable : ce script ne sert que dans le dépôt modèle."
}

$hote = (Get-Process -Id $PID).Path
$echecs = New-Object Collections.Generic.List[string]
$copies = New-Object Collections.Generic.List[string]

function Verifier([bool]$condition, [string]$message) {
    if ($condition) { Write-Host "  ok    $message" }
    else { Write-Host "  ÉCHEC $message" -ForegroundColor Red; $script:echecs.Add($message) }
}

# Copie le dépôt (sans .git, .work ni sorties de compilation) dans un dossier jetable, et en fait un dépôt git propre.
function Nouvelle-Copie {
    $copie = Join-Path ([IO.Path]::GetTempPath()) ('aria-modele-squelette-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
    New-Item -ItemType Directory -Path $copie | Out-Null
    $script:copies.Add($copie)
    Push-Location -LiteralPath $racine
    try {
        $ErrorActionPreference = 'Continue'
        $fichiers = @(git -c core.quotepath=false ls-files --cached --others --exclude-standard 2>$null)
        $ErrorActionPreference = 'Stop'
    }
    finally { Pop-Location }
    foreach ($relatif in $fichiers) {
        if ($relatif -match '(^|/)(bin|obj)/') { continue }
        $source = Join-Path $racine $relatif
        if (-not (Test-Path -LiteralPath $source -PathType Leaf)) { continue }
        $cible = Join-Path $copie $relatif
        New-Item -ItemType Directory -Path (Split-Path -Parent $cible) -Force | Out-Null
        Copy-Item -LiteralPath $source -Destination $cible -Force
    }
    Push-Location -LiteralPath $copie
    try {
        # Sous PowerShell 5.1, un simple avertissement de git sur sa sortie d'erreur arrêterait le script.
        $ErrorActionPreference = 'Continue'
        git init --quiet 2>&1 | Out-Null
        git add -A 2>&1 | Out-Null
        # Dépôt jetable : identité factice et pas de signature, rien de tout cela ne touche le vrai dépôt.
        git -c user.name=essai -c user.email=essai@example.invalid -c commit.gpgsign=false commit --quiet -m essai 2>&1 | Out-Null
        if ($LASTEXITCODE -ne 0) { throw "Impossible de préparer le dépôt d'essai dans $copie." }
    }
    finally { Pop-Location }
    return $copie
}

$stacks = @(Get-ChildItem -LiteralPath $dossierStacks -Directory |
    Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName '_socle\stack.json') } |
    Where-Object { -not $Stack -or $_.Name -eq $Stack })
if ($stacks.Count -eq 0) { throw "Aucun squelette à tester$(if ($Stack) { " pour « $Stack »" })." }

try {
    foreach ($s in $stacks) {
        Write-Host "Squelette « $($s.Name) »"
        $c = Nouvelle-Copie

        & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $c 'init.ps1') `
            -Nom Essai -Stack $s.Name -Promesse 'Une appli pour essayer le squelette.' -ElementOrange 'une clé' `
            -DemarrageAuto non -Installation oui -Port 0 | Out-Host
        Verifier ($LASTEXITCODE -eq 0) "$($s.Name) : init.ps1 crée l'appli et ses tests passent"
        if ($LASTEXITCODE -ne 0) { continue }

        Verifier (-not (Test-Path (Join-Path $c 'tools\tester-squelettes.ps1'))) "$($s.Name) : tester-squelettes.ps1 n'est pas copié dans l'appli"
        Verifier (-not (Test-Path (Join-Path $c '.github\workflows\squelettes.yml'))) "$($s.Name) : squelettes.yml n'est pas copié dans l'appli"

        $version = "$(& (Join-Path $c 'tools\version.ps1') | Select-Object -Last 1)".Trim()
        Verifier ($version -match '^\d+\.\d+\.\d+$') "$($s.Name) : tools\version.ps1 répond X.Y.Z ($version)"

        & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $c 'build.ps1') | Out-Host
        Verifier ($LASTEXITCODE -eq 0) "$($s.Name) : build.ps1 compile l'exe"
        Verifier (Test-Path (Join-Path $c '.work\sortie\Aria-Essai.exe')) "$($s.Name) : Aria-Essai.exe est dans .work\sortie\"

        & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $c 'tools\empaqueter.ps1') | Out-Host
        $zip = Join-Path $c ".work\release\Aria-Essai-$version.zip"
        Verifier (Test-Path $zip) "$($s.Name) : empaqueter.ps1 produit Aria-Essai-$version.zip"
        if (Test-Path $zip) {
            $extrait = Join-Path $c '.work\extrait'
            Expand-Archive -LiteralPath $zip -DestinationPath $extrait
            $contenu = @(Get-ChildItem -LiteralPath (Join-Path $extrait "Aria-Essai-$version") -Name)
            foreach ($attendu in 'Aria-Essai.exe', 'BUILDINFO.txt', 'LICENCES-TIERS.txt') {
                Verifier ($contenu -contains $attendu) "$($s.Name) : le zip contient $attendu"
            }
            $sortie = @(Get-ChildItem -LiteralPath (Join-Path $c '.work\sortie') -Name)
            $manquants = @($sortie | Where-Object { $contenu -notcontains $_ })
            Verifier ($manquants.Count -eq 0) "$($s.Name) : le zip contient tout ce que build.ps1 a produit ($($manquants -join ', '))"
            Verifier (@($contenu | Where-Object { $_ -like '*.pdb' }).Count -eq 0) "$($s.Name) : aucun fichier de débogage dans le zip"
            $exe = Get-Item -LiteralPath (Join-Path $extrait "Aria-Essai-$version\Aria-Essai.exe") -ErrorAction SilentlyContinue
            if ($exe) { Write-Host "        taille de l'exe : $([math]::Round($exe.Length / 1MB, 1)) Mo, zip : $([math]::Round((Get-Item $zip).Length / 1MB, 1)) Mo" }
        }
    }
}
finally {
    foreach ($copie in $copies) {
        if ($Garder) { Write-Host "Appli jetable gardée : $copie (à supprimer à la main)"; continue }
        # Seulement les dossiers jetables créés par ce script.
        if ($copie -like '*aria-modele-squelette-*' -and (Test-Path -LiteralPath $copie)) {
            Remove-Item -LiteralPath $copie -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

Write-Host ""
if ($echecs.Count -gt 0) {
    Write-Host "$($echecs.Count) contrôle(s) manqué(s)." -ForegroundColor Red
    exit 1
}
Write-Host "Tous les squelettes passent." -ForegroundColor Cyan
exit 0
