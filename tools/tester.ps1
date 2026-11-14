<#
.SYNOPSIS
Tests du dépôt modèle : rejoue init.ps1 sur une stack d'essai, dans un dossier jetable.

.DESCRIPTION
Ce fichier est celui du dépôt modèle. Dans une appli, init.ps1 le remplace par le tester.ps1 de la stack choisie.

Sept scénarios, chacun dans sa propre copie du dépôt :
  1. init.ps1 réussit : marqueurs remplis, blocs de stack fusionnés, fichiers du modèle retirés ;
  2. un port déjà pris est refusé, sans rien modifier ;
  3. un arbre de travail modifié est refusé, sans rien modifier ;
  4. une promesse qui casserait un tableau ou un script est refusée ;
  5. un dossier qui contient une jonction est refusé ;
  6. sans installation par l'exe, le README et les notes de version gardent le bloc portable ;
  7. verifier-release.ps1 refuse une version incomplète et accepte une version complète.

Tous les scénarios tournent ; le code de retour est non nul si un seul contrôle est manqué.
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

function Lancer-Init([string]$copie, [int]$port, [string]$promesse = 'Une appli pour essayer le modèle.', [string]$installation = 'oui') {
    & $hote -NoProfile -ExecutionPolicy Bypass -File (Join-Path $copie 'init.ps1') `
        -Nom Essai -Stack essai -Promesse $promesse -ElementOrange 'une clé' `
        -DemarrageAuto non -Installation $installation -Port $port | Out-Host
    return $LASTEXITCODE
}

function Texte([string]$copie, [string]$relatif) {
    return [IO.File]::ReadAllText((Join-Path $copie $relatif), [Text.Encoding]::UTF8)
}

try {
    Write-Host "1. init.ps1 réussit"
    $c = Nouvelle-Copie
    # Un fichier écarté par .gitignore : init.ps1 ne doit pas y toucher.
    New-Item -ItemType Directory -Path (Join-Path $c '.work') | Out-Null
    [IO.File]::WriteAllText((Join-Path $c '.work\note.txt'), 'Aria-Modele {{NOM}}')
    $code = Lancer-Init $c 8123
    Verifier ($code -eq 0) "init.ps1 se termine sans erreur"
    Verifier ((Texte $c '.work\note.txt') -ceq 'Aria-Modele {{NOM}}') "un fichier ignoré par git n'est pas réécrit"
    Verifier (-not (Test-Path (Join-Path $c 'init.ps1'))) "init.ps1 s'est supprimé"
    Verifier (-not (Test-Path (Join-Path $c 'stacks'))) "stacks\ est supprimé"
    Verifier (-not (Test-Path (Join-Path $c 'tools\essais'))) "tools\essais\ est supprimé"
    Verifier (-not (Test-Path (Join-Path $c 'docs\PORTS.md'))) "le registre des ports n'est pas copié dans l'appli"
    Verifier (-not (Test-Path (Join-Path $c 'CLAUDE.md'))) "aucun CLAUDE.md dans l'appli"

    $restes = @(Get-ChildItem -LiteralPath $c -Recurse -File -Force |
        Where-Object { $_.FullName -notmatch '\\\.(git|work)\\' -and $_.Extension -notin '.png', '.ico' } |
        Where-Object { [IO.File]::ReadAllText($_.FullName) -match '\{\{[A-Z_]+\}\}|<<STACK:|MODELE:debut|<!-- (INSTALLATION|PORTABLE):|Aria-Modele' } |
        ForEach-Object { $_.FullName.Substring($c.Length + 1) })
    Verifier ($restes.Count -eq 0) "aucun marqueur ne reste ($($restes -join ', '))"

    Verifier ((Texte $c 'README.md').Contains('<h1 align="center">Aria Essai</h1>')) "le README porte le nom de l'appli"
    Verifier ((Texte $c 'README.md').Contains('Une appli pour essayer le modèle.')) "le README porte la promesse"
    Verifier ((Texte $c 'docs\FICHE.md').Contains("| Stack | Stack d'essai |")) "la fiche projet porte le nom de la stack"
    Verifier ((Texte $c 'docs\FICHE.md').Contains('| Port local (interface web) | 8123 |')) "la fiche projet porte le port"
    Verifier ((Texte $c 'docs\FICHE.md').Contains("| Installation par l'exe | oui :")) "la fiche projet porte l'installation par l'exe"
    Verifier ((Texte $c 'README.md').Contains('.Aria-Essai\programme')) "le README garde le bloc d'installation"
    Verifier (-not (Texte $c 'README.md').Contains('Choisis un dossier stable')) "le README retire le bloc portable"
    Verifier ((Texte $c '.github\release-notes.md').Contains('choisis « Installer »')) "les notes de version gardent le bloc d'installation"
    Verifier (-not (Texte $c '.github\release-notes.md').Contains('Choisis un dossier stable')) "les notes de version retirent le bloc portable"
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

    Write-Host "4. Une promesse qui casserait un tableau ou un script est refusée"
    $c = Nouvelle-Copie
    $code = Lancer-Init $c 8123 'Une promesse | avec une barre verticale'
    Verifier ($code -ne 0) "init.ps1 s'arrête en erreur"
    Verifier (Test-Path (Join-Path $c 'stacks')) "rien n'a été modifié : stacks\ est toujours là"

    Write-Host "5. Un dossier qui contient une jonction est refusé"
    $c = Nouvelle-Copie
    $cible = Join-Path ([IO.Path]::GetTempPath()) ('aria-modele-essai-cible-' + [guid]::NewGuid().ToString('N').Substring(0, 8))
    New-Item -ItemType Directory -Path $cible | Out-Null
    $copies.Add($cible)
    [IO.File]::WriteAllText((Join-Path $cible 'temoin.txt'), 'Aria-Modele')
    New-Item -ItemType Directory -Path (Join-Path $c '.work') | Out-Null
    New-Item -ItemType Junction -Path (Join-Path $c '.work\lien') -Target $cible | Out-Null
    $code = Lancer-Init $c 8123
    Verifier ($code -ne 0) "init.ps1 s'arrête en erreur"
    Verifier (Test-Path (Join-Path $c 'stacks')) "rien n'a été modifié : stacks\ est toujours là"
    Verifier ((Texte $cible 'temoin.txt') -ceq 'Aria-Modele') "la cible de la jonction est intacte"
    # La jonction est retirée avant le ménage, pour que la suppression de la copie ne la traverse pas.
    [IO.Directory]::Delete((Join-Path $c '.work\lien'))

    Write-Host "6. Une appli sans installation par l'exe garde seulement le bloc portable"
    $c = Nouvelle-Copie
    $code = Lancer-Init $c 8123 -installation non
    Verifier ($code -eq 0) "init.ps1 se termine sans erreur"
    Verifier ((Texte $c 'docs\FICHE.md').Contains("| Installation par l'exe | non :")) "la fiche projet dit que l'exe ne s'installe pas"
    Verifier (-not (Texte $c 'README.md').Contains('programme')) "le README retire le bloc d'installation"
    Verifier ((Texte $c 'README.md').Contains('Choisis un dossier stable')) "le README garde le bloc portable"
    Verifier (-not (Texte $c '.github\release-notes.md').Contains('Installer')) "les notes de version retirent le bloc d'installation"
    Verifier ((Texte $c '.github\release-notes.md').Contains('Choisis un dossier stable')) "les notes de version gardent le bloc portable"
    $restes = @('README.md', '.github\release-notes.md' | Where-Object { (Texte $c $_) -match '<!-- (INSTALLATION|PORTABLE):' })
    Verifier ($restes.Count -eq 0) "aucune balise d'installation ne reste ($($restes -join ', '))"

    Write-Host "7. verifier-release.ps1 refuse une version incomplète"
    $c = Nouvelle-Copie
    $code = Lancer-Init $c 8123
    Verifier ($code -eq 0) "init.ps1 se termine sans erreur"
    $verif = Join-Path $c 'tools\verifier-release.ps1'
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -Tag v1.2.3 | Out-Host
    Verifier ($LASTEXITCODE -ne 0) "sans notes, avec fiche et licences à renseigner : refusé"
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -EssaiABlanc | Out-Host
    Verifier ($LASTEXITCODE -eq 0) "l'essai à blanc liste les trous sans échouer"

    # On complète tout, comme avant une vraie version.
    $notes = (Texte $c '.github\release-notes.md').Replace('{VERSION}', '1.2.3').Replace('{REPO_URL}', 'https://github.com/ZedjeK/Aria-Essai')
    $notes = [regex]::Replace($notes, '(?s)<!--.*?-->\r?\n?', '')
    New-Item -ItemType Directory -Path (Join-Path $c 'docs\releases') -Force | Out-Null
    [IO.File]::WriteAllText((Join-Path $c 'docs\releases\v1.2.3.md'), $notes)
    [IO.File]::WriteAllText((Join-Path $c 'docs\FICHE.md'), (Texte $c 'docs\FICHE.md').Replace(': à renseigner', ': renseigné').Replace('| à renseigner', '| renseigné'))
    $licences = [regex]::Replace((Texte $c 'LICENCES-TIERS.txt'), '(?m)^\(à compléter.*\r?\n?', '').Replace('à renseigner', 'renseigné')
    [IO.File]::WriteAllText((Join-Path $c 'LICENCES-TIERS.txt'), $licences)
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -Tag v1.2.3 | Out-Host
    Verifier ($LASTEXITCODE -eq 0) "notes, fiche et licences complètes : accepté"

    [IO.File]::WriteAllText((Join-Path $c 'docs\releases\v1.2.3.md'), $notes.Replace("Décompresse d'abord", 'Décompresse'))
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -Tag v1.2.3 | Out-Host
    Verifier ($LASTEXITCODE -ne 0) "des notes sans « décompresse d'abord » : refusé"
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
