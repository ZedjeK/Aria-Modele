<#
.SYNOPSIS
Tests des squelettes de stack du dépôt modèle : init.ps1, compilation et zip, dans un dossier jetable.

.DESCRIPTION
Fichier du dépôt modèle seulement : init.ps1 le supprime dans l'appli créée, avec .github\workflows\squelettes.yml.

D'abord le socle partagé : la crate aria-coeur (socle\coeur : rustfmt, clippy et ses tests, avec sa fenêtre Tauri ;
ses fichiers de réglage de Rust sont ceux de chaque stack Rust) et le paquet aria-interface (socle\interface :
svelte-check et ses tests), à la même version.

Puis, pour chaque dossier de stacks\ qui a un _socle\stack.json (ou seulement celui donné par -Stack) :
  1. init.ps1 crée une appli « Essai » dans une copie jetable du dépôt, sur le socle de ce clone (-Socle), et lance
     les tests de la stack ;
  2. tools\icone.ps1, si la stack le fournit, tire l'icône Windows de docs\logo.svg ; build.ps1 compile l'exe ;
     tools\empaqueter.ps1 en fait le zip et son empreinte ;
  3. le zip contient l'exe, BUILDINFO.txt et LICENCES-TIERS.txt ; tools\version.ps1 répond X.Y.Z.

Il faut les outils de chaque stack (Rust et Node.js pour rust-tauri). Tous les squelettes tournent ; le code de
retour est non nul si un seul contrôle est manqué.

    .\tools\tester-squelettes.ps1
    .\tools\tester-squelettes.ps1 -Stack rust-tauri
    .\tools\tester-squelettes.ps1 -Stack rust-tauri -Garder    garde l'appli jetable pour la lancer à la main
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

# Lance un programme natif et dit s'il a réussi. Sous PowerShell 5.1, sa sortie d'erreur redirigée arrêterait le
# script : on lit son code de retour à la place.
function Reussit([scriptblock]$commande) {
    $ErrorActionPreference = 'Continue'
    & $commande | Out-Host
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    return ($code -eq 0)
}

# ----- Le socle partagé -----

$socle = Join-Path $racine 'socle\coeur'
Write-Host "Socle partagé (socle\coeur)"
foreach ($fichier in 'rust-toolchain.toml', 'clippy.toml', 'rustfmt.toml') {
    $attendu = [IO.File]::ReadAllText((Join-Path $socle $fichier))
    Get-ChildItem -LiteralPath $dossierStacks -Directory | Where-Object { Test-Path (Join-Path $_.FullName $fichier) } | ForEach-Object {
        $lu = [IO.File]::ReadAllText((Join-Path $_.FullName $fichier))
        Verifier ($lu -ceq $attendu) "socle : $fichier est celui de stacks\$($_.Name)"
    }
}
# La crate et le paquet de l'interface ont la version du dépôt modèle : une seule, pour un seul tag.
$versionCrate = ''
foreach ($ligne in (Get-Content -LiteralPath (Join-Path $socle 'Cargo.toml') -Encoding UTF8)) {
    if ($ligne -match '^version\s*=\s*"(\d+\.\d+\.\d+)"') { $versionCrate = $Matches[1]; break }
}
$interfaceSocle = Join-Path $racine 'socle\interface'
$versionPaquet = (Get-Content -LiteralPath (Join-Path $interfaceSocle 'package.json') -Raw -Encoding UTF8 | ConvertFrom-Json).version
Verifier ($versionCrate -and $versionCrate -eq $versionPaquet) "socle : la crate ($versionCrate) et le paquet de l'interface ($versionPaquet) ont la même version"
if (-not $Stack -or $Stack -eq 'rust-tauri') {
    Push-Location -LiteralPath $socle
    try {
        Verifier (Reussit { cargo fmt --check }) 'socle : mise en forme (rustfmt)'
        Verifier (Reussit { cargo clippy --all-targets --all-features --locked -- -D warnings }) 'socle : contrôles (clippy, fenêtre comprise)'
        Verifier (Reussit { cargo test --all-features --locked }) 'socle : tests, fenêtre comprise'
    }
    finally { Pop-Location }
    Push-Location -LiteralPath $interfaceSocle
    try {
        Verifier (Reussit { npm ci --no-audit --no-fund }) 'socle : paquets de l''interface'
        Verifier (Reussit { npm run check }) 'socle : types et composants de l''interface (svelte-check)'
        Verifier (Reussit { npm test }) 'socle : tests de l''interface (vitest)'
    }
    finally { Pop-Location }
}

# ----- Les squelettes -----

$stacks = @(Get-ChildItem -LiteralPath $dossierStacks -Directory |
    Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName '_socle\stack.json') } |
    Where-Object { -not $Stack -or $_.Name -eq $Stack })
if ($stacks.Count -eq 0) { throw "Aucun squelette à tester$(if ($Stack) { " pour « $Stack »" })." }

try {
    foreach ($s in $stacks) {
        Write-Host "Squelette « $($s.Name) »"
        $c = Nouvelle-Copie

        # Une promesse plus longue que la largeur des lignes de code (110 caractères pour rustfmt) : elle est recopiée
        # dans le code de la stack, dont la mise en forme est contrôlée par ses tests.
        $promesse = "Une appli pour essayer le squelette, avec une promesse assez longue pour dépasser la largeur qu'un formateur de code accepte sur une ligne."
        & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $c 'init.ps1') `
            -Nom Essai -Stack $s.Name -Promesse $promesse -ElementOrange 'une clé' `
            -DemarrageAuto oui -Port 0 -Socle $racine | Out-Host
        Verifier ($LASTEXITCODE -eq 0) "$($s.Name) : init.ps1 crée l'appli et ses tests passent"
        if ($LASTEXITCODE -ne 0) { continue }

        Verifier (-not (Test-Path (Join-Path $c 'socle'))) "$($s.Name) : le code du socle n'est pas copié dans l'appli"

        Verifier (-not (Test-Path (Join-Path $c 'tools\tester-squelettes.ps1'))) "$($s.Name) : tester-squelettes.ps1 n'est pas copié dans l'appli"
        Verifier (-not (Test-Path (Join-Path $c '.github\workflows\squelettes.yml'))) "$($s.Name) : squelettes.yml n'est pas copié dans l'appli"
        Verifier (-not (Test-Path (Join-Path $c '.github\workflows\socle-release.yml'))) "$($s.Name) : socle-release.yml n'est pas copié dans l'appli"
        $monter = Join-Path $c '.github\workflows\monter-socle.yml'
        Verifier ((Test-Path $monter) -and ([IO.File]::ReadAllText($monter) -cnotmatch 'MODELE:|Aria-Essai')) "$($s.Name) : monter-socle.yml est dans l'appli, sans bloc du modèle, et vise le dépôt modèle"

        # La liste des crates Rust de LICENCES-TIERS.txt, générée par cargo-about, si la stack fournit le script.
        $outilLicences = Join-Path $c 'tools\licences-rust.ps1'
        if (Test-Path $outilLicences) {
            & $hote -NoProfile -ExecutionPolicy Bypass -File $outilLicences | Out-Host
            Verifier ($LASTEXITCODE -eq 0) "$($s.Name) : tools\licences-rust.ps1 génère la liste des crates Rust"
            $licencesApp = [IO.File]::ReadAllText((Join-Path $c 'LICENCES-TIERS.txt'), [Text.Encoding]::UTF8)
            Verifier ($licencesApp -match 'Cargo\.lock \(SHA-256, fins de ligne LF\) : [0-9a-f]{64}') "$($s.Name) : LICENCES-TIERS.txt porte l'empreinte de Cargo.lock"
            Verifier ($licencesApp -match '(?m)^tauri \d+\.\d+\.\d+') "$($s.Name) : la liste des crates Rust nomme tauri"
        }

        $version = "$(& (Join-Path $c 'tools\version.ps1') | Select-Object -Last 1)".Trim()
        Verifier ($version -match '^\d+\.\d+\.\d+$') "$($s.Name) : tools\version.ps1 répond X.Y.Z ($version)"

        # L'icône Windows tirée de docs\logo.svg (rendu par Edge), avant la compilation qui l'embarque dans l'exe.
        $outilIcone = Join-Path $c 'tools\icone.ps1'
        if (Test-Path $outilIcone) {
            & $hote -NoProfile -ExecutionPolicy Bypass -File $outilIcone | Out-Host
            Verifier ($LASTEXITCODE -eq 0) "$($s.Name) : tools\icone.ps1 tire l'icône de docs\logo.svg"
        }

        & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $c 'build.ps1') | Out-Host
        Verifier ($LASTEXITCODE -eq 0) "$($s.Name) : build.ps1 compile l'exe"
        $exeSortie = Join-Path $c '.work\sortie\Aria-Essai.exe'
        Verifier (Test-Path $exeSortie) "$($s.Name) : Aria-Essai.exe est dans .work\sortie\"

        # L'exe se lance et tient : un plantage au démarrage (capacité refusée, panique) l'arrêterait aussitôt. Compilé
        # sans fichiers.txt à côté de lui, il ouvre sa fenêtre et attend ; on l'arrête ensuite.
        if ((Test-Path $exeSortie) -and $env:OS -eq 'Windows_NT') {
            $journal = Join-Path $env:USERPROFILE '.Aria-Essai\journal.json'
            $avant = if (Test-Path $journal) { (Get-Item $journal).LastWriteTimeUtc } else { [datetime]::MinValue }
            $processus = Start-Process -FilePath $exeSortie -PassThru
            Start-Sleep -Seconds 15
            $tient = -not $processus.HasExited
            Verifier $tient "$($s.Name) : l'exe se lance et tient 15 secondes (code de sortie : $(if ($tient) { 'aucun' } else { $processus.ExitCode }))"
            $ecrit = (Test-Path $journal) -and (Get-Item $journal).LastWriteTimeUtc -gt $avant -and
                ([IO.File]::ReadAllText($journal) -match '"lancement"')
            Verifier $ecrit "$($s.Name) : l'exe note son lancement dans son journal"
            if ($tient) { Stop-Process -Id $processus.Id -Force -ErrorAction SilentlyContinue }
        }

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
            $infos = [IO.File]::ReadAllText((Join-Path $extrait "Aria-Essai-$version\BUILDINFO.txt"), [Text.Encoding]::UTF8)
            Verifier ($infos -match "Taille de l'exe : \d+(\.\d)? Mo \(\d+ octets\)") "$($s.Name) : BUILDINFO.txt relève la taille de l'exe"
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
