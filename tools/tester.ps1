<#
.SYNOPSIS
Tests du dépôt modèle : rejoue init.ps1 sur une stack d'essai, dans un dossier jetable.

.DESCRIPTION
Ce fichier est celui du dépôt modèle. Dans une appli, init.ps1 le remplace par le tester.ps1 de la stack choisie.

Onze scénarios, les huit premiers et le dernier chacun dans sa propre copie du dépôt :
  1. init.ps1 réussit : marqueurs remplis, blocs de stack fusionnés, fichiers du modèle retirés ;
  2. un port déjà pris est refusé, sans rien modifier ;
  3. un arbre de travail modifié est refusé, sans rien modifier ;
  4. une promesse qui casserait un tableau ou un script est refusée ;
  5. un dossier qui contient une jonction est refusé ;
  6. sans installation par l'exe, le README et les notes de version gardent le bloc portable ;
  7. verifier-release.ps1 refuse une version incomplète et accepte une version complète ;
  8. une stack refuse ce qu'elle ne fournit pas encore (clé nonFournis de stack.json) ;
  9. les actions recopiées dans les _socle\outils.txt gardent le SHA épinglé dans les workflows ;
 10. chaque script PowerShell du dépôt se lit sans erreur de syntaxe ;
 11. le socle commun (tools\socle.txt) : une appli fraîchement créée est à jour, une appli modifiée est signalée.

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
    Verifier (-not (Test-Path (Join-Path $c 'tools\tester-squelettes.ps1'))) "les tests des squelettes ne sont pas copiés dans l'appli"
    Verifier (-not (Test-Path (Join-Path $c '.github\workflows\squelettes.yml'))) "le workflow des squelettes n'est pas copié dans l'appli"
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
    Verifier (-not (Texte $c '.github\dependabot.yml').Contains('/stacks/')) "les blocs Dependabot des squelettes ne sont pas copiés dans l'appli"
    Verifier ((Texte $c '.github\workflows\tests.yml').Contains('Outils de la stack essai')) "les outils de la stack sont dans tests.yml"
    Verifier ((Texte $c '.github\workflows\tests.yml').Contains('runs-on: ubuntu-24.04')) "tests.yml tourne sur la machine demandée par la stack"
    Verifier ((Texte $c '.github\workflows\tests.yml').Contains('  tests: #')) "le job s'appelle toujours « tests »"
    Verifier ((Texte $c '.github\workflows\release.yml').Contains('Outils de la stack essai')) "les outils de la stack sont dans release.yml"
    Verifier ((Texte $c '.github\workflows\release.yml').Contains('release/Aria-Essai-$version.zip')) "release.yml nomme le zip de l'appli"
    Verifier ((Texte $c '.github\workflows\codeql.yml').Contains('language: essai')) "le langage de la stack est dans codeql.yml"
    Verifier ((Texte $c 'LICENCES-TIERS.txt').Contains("Composant d'essai")) "les composants de la stack sont dans LICENCES-TIERS.txt"
    Verifier (Test-Path (Join-Path $c '.github\pull_request_template.md')) "le modèle de PR est copié dans l'appli"
    Verifier ((Texte $c '.github\release.yml').Contains('title: Nouveautés')) "les catégories des notes générées sont copiées dans l'appli"
    Verifier ((Texte $c '.github\ISSUE_TEMPLATE\bug.yml').Contains("Version d'Aria Essai")) "le formulaire de bug porte le nom de l'appli"
    Verifier ((Texte $c '.github\ISSUE_TEMPLATE\config.yml').Contains('ZedjeK/Aria-Essai/security/advisories/new')) "les issues renvoient au signalement privé de l'appli"
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
        Verifier (($contenu -join ',') -eq 'Aria-Essai.txt,BUILDINFO.txt,fichiers.txt,LICENCES-TIERS.txt') "le zip contient l'exe, BUILDINFO.txt, fichiers.txt et LICENCES-TIERS.txt ($($contenu -join ','))"
        $liste = @(Get-Content -LiteralPath (Join-Path $extrait 'Aria-Essai-1.2.3\fichiers.txt') -Encoding UTF8)
        Verifier (($liste -join ',') -eq 'Aria-Essai.txt,BUILDINFO.txt,LICENCES-TIERS.txt') "fichiers.txt liste tout le zip, sauf lui-même ($($liste -join ','))"
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
    # La copie n'a pas bougé : on la réutilise pour une promesse qui casserait un fichier XML.
    $code = Lancer-Init $c 8123 'Rapide & sans souci'
    Verifier ($code -ne 0) "une esperluette est refusée"
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

    Write-Host "8. Une stack refuse ce qu'elle ne fournit pas encore"
    $c = Nouvelle-Copie
    $socle = Join-Path $c 'stacks\essai\_socle\stack.json'
    [IO.File]::WriteAllText($socle, '{ "nom": "Stack d''essai", "runsOn": "ubuntu-24.04", "nonFournis": ["installation", "misesAJour"] }')
    Push-Location -LiteralPath $c
    try {
        # Le changement est validé dans le dépôt jetable : init.ps1 exige un arbre de travail propre.
        $ErrorActionPreference = 'Continue'
        git add -A 2>&1 | Out-Null
        git -c user.name=essai -c user.email=essai@example.invalid -c commit.gpgsign=false commit --quiet -m nonFournis 2>&1 | Out-Null
    }
    finally {
        $ErrorActionPreference = 'Stop'
        Pop-Location
    }
    $code = Lancer-Init $c 8123
    Verifier ($code -ne 0) "installation demandée à une stack qui ne la fournit pas : refusé"
    Verifier (Test-Path (Join-Path $c 'stacks')) "rien n'a été modifié : stacks\ est toujours là"
    $code = Lancer-Init $c 8123 -installation non
    Verifier ($code -eq 0) "sans installation : init.ps1 se termine sans erreur"

    Write-Host "9. Les actions des blocs de stack gardent le SHA des workflows"
    # Dependabot ne lit que .github\workflows : un SHA recopié dans un _socle\outils.txt resterait en arrière.
    $motif = 'uses:\s*([^\s@]+)@([0-9a-f]{40})'
    $epingles = @{}
    Get-ChildItem -LiteralPath (Join-Path $racine '.github\workflows') -Filter '*.yml' | ForEach-Object {
        foreach ($m in [regex]::Matches([IO.File]::ReadAllText($_.FullName), $motif)) {
            $action = $m.Groups[1].Value
            if (-not $epingles.ContainsKey($action)) { $epingles[$action] = New-Object Collections.Generic.List[string] }
            if (-not $epingles[$action].Contains($m.Groups[2].Value)) { $epingles[$action].Add($m.Groups[2].Value) }
        }
    }
    $doubles = @($epingles.Keys | Where-Object { $epingles[$_].Count -gt 1 })
    Verifier ($doubles.Count -eq 0) "chaque action a un seul SHA dans les workflows ($($doubles -join ', '))"
    $fragments = @(Get-ChildItem -LiteralPath (Join-Path $racine 'stacks') -Recurse -File -Filter 'outils.txt' |
        Where-Object { $_.Directory.Name -eq '_socle' })
    foreach ($f in $fragments) {
        $relatif = $f.FullName.Substring($racine.Length + 1)
        foreach ($m in [regex]::Matches([IO.File]::ReadAllText($f.FullName), $motif)) {
            $action = $m.Groups[1].Value
            $sha = $m.Groups[2].Value
            Verifier ($epingles.ContainsKey($action) -and $epingles[$action].Contains($sha)) "${relatif} : $action garde le SHA des workflows"
        }
    }

    Write-Host "10. Chaque script PowerShell du dépôt se lit sans erreur de syntaxe"
    # Certains scripts ne tournent pas en CI (proteger-depot.ps1 écrit sur GitHub) : leur syntaxe, au moins, est relue.
    $scripts = @(Get-ChildItem -LiteralPath $racine -Recurse -File -Filter '*.ps1' |
        Where-Object { $_.FullName -notmatch '[\\/]\.(git|work)[\\/]' })
    foreach ($s in $scripts) {
        $jetons = $null
        $erreurs = $null
        [Management.Automation.Language.Parser]::ParseFile($s.FullName, [ref]$jetons, [ref]$erreurs) | Out-Null
        $detail = ''
        if ($erreurs.Count -gt 0) { $detail = " (ligne $($erreurs[0].Extent.StartLineNumber) : $($erreurs[0].Message))" }
        Verifier ($erreurs.Count -eq 0) "$($s.FullName.Substring($racine.Length + 1)) se lit sans erreur de syntaxe$detail"
    }

    Write-Host "11. Le socle commun : une appli fraîchement créée est à jour, une appli modifiée ne l'est pas"
    $socle = @([IO.File]::ReadAllLines((Join-Path $racine 'tools\socle.txt'), [Text.Encoding]::UTF8) |
        ForEach-Object { $_.Trim() } | Where-Object { $_ -and -not $_.StartsWith('#') })
    Verifier ($socle.Count -gt 0) "tools\socle.txt liste des fichiers"
    foreach ($f in $socle) {
        $relatif = $f -replace '/', '\'
        $existe = Test-Path -LiteralPath (Join-Path $racine $relatif) -PathType Leaf
        Verifier $existe "$f existe dans le modèle"
        # Tout ce que init.ps1 réécrit : marqueur, bloc de stack, bloc retiré, ou le nom du modèle lui-même. Un
        # fichier qui en porte ne peut pas rester identique au modèle.
        if ($existe) {
            Verifier ((Texte $racine $relatif) -notmatch '\{\{[A-Z_]+\}\}|<<STACK:[a-z-]+>>|<!-- [A-Z]+:(debut|fin) -->|Aria-Modele') "$f ne porte rien que init.ps1 réécrit"
        }
    }
    $c = Nouvelle-Copie
    $code = Lancer-Init $c 8123
    Verifier ($code -eq 0) "init.ps1 se termine sans erreur"
    $verif = Join-Path $c 'tools\verifier-socle.ps1'
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -Modele $racine | Out-Host
    Verifier ($LASTEXITCODE -eq 0) "une appli fraîchement créée a tout le socle du modèle"
    [IO.File]::AppendAllText((Join-Path $c '.github\release.yml'), "`n# une ligne ajoutée dans l'appli`n")
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -Modele $racine | Out-Host
    Verifier ($LASTEXITCODE -eq 1) "un fichier du socle modifié dans l'appli est signalé"
    Remove-Item -LiteralPath (Join-Path $c '.github\release.yml') -Force
    & $hote -NoProfile -ExecutionPolicy Bypass -File $verif -Modele $racine | Out-Host
    Verifier ($LASTEXITCODE -eq 1) "un fichier du socle absent de l'appli est signalé"
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
