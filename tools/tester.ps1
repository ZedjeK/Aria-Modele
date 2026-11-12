<#
.SYNOPSIS
Tests du dépôt modèle : rejoue init.ps1 sur une stack d'essai, dans un dossier jetable.

.DESCRIPTION
Ce fichier est celui du dépôt modèle. Dans une appli, init.ps1 le remplace par le tester.ps1 de la stack choisie.

Trois scénarios, chacun dans sa propre copie du dépôt :
  1. init.ps1 réussit : marqueurs remplis, blocs de stack fusionnés, fichiers du modèle retirés ;
  2. un port déjà pris est refusé, sans rien modifier ;
  3. un arbre de travail modifié est refusé, sans rien modifier.

Code de retour non nul au premier contrôle manqué.
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = Split-Path -Parent $PSScriptRoot
if (-not (Test-Path -LiteralPath (Join-Path $racine 'stacks') -PathType Container)) {
    throw "Ce tester.ps1 est celui du dépôt modèle, et stacks\ n'existe plus : la stack choisie aurait dû fournir le sien."
}

$hote = (Get-Process -Id $PID).Path
$echecs = New-Object Collections.Generic.List[string]
$copies = New-Object Collections.Generic.List[string]

function Verifier([bool]$condition, [string]$message) {
    if ($condition) { Write-Host "  ok    $message" }
    else { Write-Host "  ÉCHEC $message" -ForegroundColor Red; $script:echecs.Add($message) }
}

# Copie le dépôt (sans .git ni .work) dans un dossier jetable, y pose la stack d'essai et en fait un dépôt git propre.
function Nouvelle-Copie {
    $copie = Join-Path ([IO.Path]::GetTempPath()) ('aria-modele-essai-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
    New-Item -ItemType Directory -Path $copie | Out-Null
    $script:copies.Add($copie)
    Get-ChildItem -LiteralPath $racine -Force | Where-Object { $_.Name -notin '.git', '.work' } | ForEach-Object {
        Copy-Item -LiteralPath $_.FullName -Destination $copie -Recurse -Force
    }
    Copy-Item -LiteralPath (Join-Path $racine 'tools\essais\stack-essai') -Destination (Join-Path $copie 'stacks\essai') -Recurse -Force
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

function Lancer-Init([string]$copie, [int]$port) {
    & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $copie 'init.ps1') `
        -Nom Essai -Stack essai -Promesse 'Une appli pour essayer le modèle.' -ElementOrange 'une clé' `
        -DemarrageAuto non -Port $port | Out-Host
    return $LASTEXITCODE
}

function Texte([string]$copie, [string]$relatif) {
    return [IO.File]::ReadAllText((Join-Path $copie $relatif), [Text.Encoding]::UTF8)
}

try {
    Write-Host "1. init.ps1 réussit"
    $c = Nouvelle-Copie
    $code = Lancer-Init $c 8123
    Verifier ($code -eq 0) "init.ps1 se termine sans erreur"
    Verifier (-not (Test-Path (Join-Path $c 'init.ps1'))) "init.ps1 s'est supprimé"
    Verifier (-not (Test-Path (Join-Path $c 'stacks'))) "stacks\ est supprimé"
    Verifier (-not (Test-Path (Join-Path $c 'tools\essais'))) "tools\essais\ est supprimé"
    Verifier (-not (Test-Path (Join-Path $c 'docs\PORTS.md'))) "le registre des ports n'est pas copié dans l'appli"

    $restes = @(Get-ChildItem -LiteralPath $c -Recurse -File -Force |
        Where-Object { $_.FullName -notmatch '\\\.git\\' -and $_.Extension -notin '.png', '.ico' } |
        Where-Object { [IO.File]::ReadAllText($_.FullName) -match '\{\{[A-Z_]+\}\}|<<STACK:|MODELE:debut|Aria-Modele' } |
        ForEach-Object { $_.FullName.Substring($c.Length + 1) })
    Verifier ($restes.Count -eq 0) "aucun marqueur ne reste ($($restes -join ', '))"

    Verifier ((Texte $c 'README.md').Contains('<h1 align="center">Aria Essai</h1>')) "le README porte le nom de l'appli"
    Verifier ((Texte $c 'README.md').Contains('Une appli pour essayer le modèle.')) "le README porte la promesse"
    Verifier ((Texte $c 'CLAUDE.md').Contains("| Stack | Stack d'essai |")) "la fiche projet porte le nom de la stack"
    Verifier ((Texte $c 'CLAUDE.md').Contains('| Port local (interface web) | 8123 |')) "la fiche projet porte le port"
    Verifier ((Texte $c 'LICENSE').Contains("Copyright (c) $((Get-Date).Year) ZedjeK")) "la licence porte l'année en cours"
    Verifier ((Texte $c '.gitignore').Contains('essai-ignore/')) "le bloc de la stack est dans .gitignore"
    Verifier ((Texte $c '.gitattributes').Contains('*.essai text')) "le bloc de la stack est dans .gitattributes"
    Verifier ((Texte $c '.github\dependabot.yml').Contains('essai-ecosysteme')) "le bloc de la stack est dans dependabot.yml"
    Verifier ((Texte $c '.github\workflows\tests.yml').Contains('Outils de la stack essai')) "les outils de la stack sont dans tests.yml"
    Verifier ((Texte $c '.github\workflows\tests.yml').Contains('runs-on: ubuntu-24.04')) "tests.yml tourne sur la machine demandée par la stack"
    Verifier ((Texte $c '.github\workflows\tests.yml').Contains('  tests: #')) "le job s'appelle toujours « tests »"
    Verifier ((Texte $c '.github\workflows\release.yml').Contains('Outils de la stack essai')) "les outils de la stack sont dans release.yml"
    Verifier ((Texte $c '.github\workflows\release.yml').Contains('release/Aria-Essai-$version.zip')) "release.yml nomme le zip de l'appli"
    Verifier ((Texte $c '.github\workflows\codeql.yml').Contains('language: essai')) "le langage de la stack est dans codeql.yml"
    Verifier (Test-Path (Join-Path $c 'Aria-Essai.txt')) "un fichier nommé Aria-Modele est renommé"
    Verifier ((Texte $c 'Aria-Essai.txt').Contains('Aria-Essai : Une appli pour essayer le modèle. Port 8123.')) "son contenu est rempli"
    $version = "$(& (Join-Path $c 'tools\version.ps1') | Select-Object -Last 1)".Trim()
    Verifier ($version -eq '1.2.3') "tools\version.ps1 est celui de la stack"

    # La chaîne de release, sans GitHub : build, puis zip et empreinte.
    & (Join-Path $c 'build.ps1') | Out-Null
    & (Join-Path $c 'tools\empaqueter.ps1') | Out-Null
    $zip = Join-Path $c '.work\release\Aria-Essai-1.2.3.zip'
    Verifier (Test-Path $zip) "empaqueter.ps1 produit Aria-Essai-1.2.3.zip"
    if (Test-Path $zip) {
        $attendu = (Get-FileHash -LiteralPath $zip -Algorithm SHA256).Hash.ToLowerInvariant() + '  Aria-Essai-1.2.3.zip'
        Verifier ((Texte $c '.work\release\Aria-Essai-1.2.3.zip.sha256').Trim() -eq $attendu) "le fichier .sha256 porte l'empreinte du zip"
        $extrait = Join-Path $c '.work\extrait'
        Expand-Archive -LiteralPath $zip -DestinationPath $extrait
        $contenu = @(Get-ChildItem -LiteralPath (Join-Path $extrait 'Aria-Essai-1.2.3') -Name | Sort-Object)
        Verifier (($contenu -join ',') -eq 'Aria-Essai.txt,BUILDINFO.txt,LICENCES-TIERS.txt') "le zip contient l'exe, BUILDINFO.txt et LICENCES-TIERS.txt ($($contenu -join ','))"
    }

    Write-Host "2. Un port déjà pris est refusé"
    $c = Nouvelle-Copie
    $code = Lancer-Init $c 8075
    Verifier ($code -ne 0) "init.ps1 s'arrête en erreur"
    Verifier (Test-Path (Join-Path $c 'stacks')) "rien n'a été modifié : stacks\ est toujours là"
    Verifier (Test-Path (Join-Path $c 'init.ps1')) "rien n'a été modifié : init.ps1 est toujours là"

    Write-Host "3. Un arbre de travail modifié est refusé"
    $c = Nouvelle-Copie
    Add-Content -LiteralPath (Join-Path $c 'CHANGELOG.md') -Value 'modification non validée'
    $code = Lancer-Init $c 8123
    Verifier ($code -ne 0) "init.ps1 s'arrête en erreur"
    Verifier (Test-Path (Join-Path $c 'stacks')) "rien n'a été modifié : stacks\ est toujours là"
}
finally {
    foreach ($copie in $copies) {
        # Seulement les dossiers jetables créés par ce script.
        if ($copie -like '*aria-modele-essai-*' -and (Test-Path -LiteralPath $copie)) {
            Remove-Item -LiteralPath $copie -Recurse -Force -ErrorAction SilentlyContinue
        }
    }
}

Write-Host ""
if ($echecs.Count -gt 0) {
    Write-Host "$($echecs.Count) contrôle(s) manqué(s)." -ForegroundColor Red
    exit 1
}
Write-Host "Tous les contrôles passent." -ForegroundColor Cyan
exit 0
