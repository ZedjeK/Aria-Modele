<#
.SYNOPSIS
Transforme ce dépôt modèle en appli Aria, puis se supprime.

.DESCRIPTION
À lancer une seule fois, dans le dossier du clone :

    .\init.ps1
    .\init.ps1 -Nom Atlas -Stack go-wails -Promesse "..." -ElementOrange "une boussole" -DemarrageAuto non -Installation oui -Port 8123

Le script choisit la stack, remplit le nom et la promesse, fusionne les blocs propres à la stack, lance les tests,
puis se supprime. Il ne fait aucun git add, aucun commit, aucun push et aucun appel à GitHub.

Il refuse de tourner si l'arbre de travail n'est pas propre. Pour tout annuler ensuite :

    git restore .
    git clean -fd

.PARAMETER Installation
oui : l'exe propose de se ranger dans %USERPROFILE%\.Aria-<Nom>\programme (appli de bureau). non : exe portable
seulement. Le README et les notes de version gardent les blocs INSTALLATION ou PORTABLE selon la réponse.

.PARAMETER Port
Port local de l'interface web. 0 : l'appli n'a pas d'interface web.
#>
[CmdletBinding()]
param(
    [string]$Nom,
    [string]$Stack,
    [string]$Promesse,
    [string]$ElementOrange,
    [ValidateSet('oui', 'non')][string]$DemarrageAuto,
    [ValidateSet('oui', 'non')][string]$Installation,
    [int]$Port
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = $PSScriptRoot
Set-Location -LiteralPath $racine

function Arreter([string]$message) {
    Write-Host "Arrêt : $message" -ForegroundColor Red
    exit 1
}

function Demander([string]$question) {
    return (Read-Host $question).Trim()
}

# La promesse et l'élément orange sont recopiés dans du Markdown, du HTML, du XML, du XAML et des scripts : une ligne
# de texte simple. « & » casserait un fichier XML (Directory.Build.props), une accolade ouvrirait une extension XAML.
function Verifier-Texte([string]$quoi, [string]$valeur) {
    if (-not $valeur) { Arreter "$quoi est vide." }
    if ($valeur.Length -gt 200) { Arreter "$quoi dépasse 200 caractères." }
    if ($valeur -match '[\x00-\x1F\x7F"|<>$`\\&{}]') {
        Arreter "$quoi contient un caractère refusé : saut de ligne, guillemet double, barre verticale, chevron, dollar, accent grave, barre oblique inverse, esperluette ou accolade."
    }
    if ($valeur -match '\{\{|\}\}|Aria-Modele') { Arreter "$quoi contient un marqueur du modèle." }
}

# Lit un fichier texte en gardant la trace de son BOM, pour le réécrire à l'identique.
function Lire-Texte([string]$chemin) {
    $octets = [IO.File]::ReadAllBytes($chemin)
    if ($octets -contains 0) { return $null } # binaire : on n'y touche pas
    $bom = ($octets.Length -ge 3 -and $octets[0] -eq 0xEF -and $octets[1] -eq 0xBB -and $octets[2] -eq 0xBF)
    return [pscustomobject]@{ Texte = [Text.Encoding]::UTF8.GetString($octets).TrimStart([char]0xFEFF); Bom = $bom }
}

function Ecrire-Texte([string]$chemin, [string]$texte, [bool]$bom) {
    [IO.File]::WriteAllText($chemin, $texte, (New-Object Text.UTF8Encoding($bom)))
}

# ----- Conditions de départ -----

$dossierStacks = Join-Path $racine 'stacks'
if (-not (Test-Path -LiteralPath $dossierStacks -PathType Container)) {
    Arreter "le dossier stacks\ n'existe plus : ce dépôt a déjà été initialisé."
}
if (-not (Get-Command git -ErrorAction SilentlyContinue)) {
    Arreter "git est introuvable."
}
$etat = @(git status --porcelain)
if ($LASTEXITCODE -ne 0) { Arreter "ce dossier n'est pas un dépôt git." }
if ($etat.Count -gt 0) {
    Arreter "l'arbre de travail n'est pas propre. Valide ou annule tes changements, puis relance."
}
# Un lien ou une jonction mènerait les réécritures et les suppressions hors de ce dossier.
$liens = @(Get-ChildItem -LiteralPath $racine -Force | Where-Object Name -ne '.git' | ForEach-Object {
        if ($_.Attributes -band [IO.FileAttributes]::ReparsePoint) { $_ }
        elseif ($_.PSIsContainer) { Get-ChildItem -LiteralPath $_.FullName -Recurse -Force -Attributes ReparsePoint }
    })
if ($liens.Count -gt 0) {
    Arreter "ce dossier contient un lien ou une jonction ($($liens[0].FullName)). Retire-le, puis relance."
}

$stacks = @(Get-ChildItem -LiteralPath $dossierStacks -Directory |
    Where-Object { Test-Path -LiteralPath (Join-Path $_.FullName '_socle\stack.json') })
if ($stacks.Count -eq 0) {
    Arreter "aucun squelette de stack n'est disponible dans stacks\."
}

# ----- Questions -----

if (-not $Nom) { $Nom = Demander "Nom de l'appli, sans « Aria » (exemple : Atlas)" }
if ($Nom -cnotmatch '\A[A-Z][A-Za-z0-9]{1,30}\z') {
    Arreter "le nom « $Nom » n'est pas valide : une majuscule, puis des lettres ou des chiffres, sans espace ni accent."
}

if (-not $Stack) {
    Write-Host "Stacks disponibles : $(($stacks | ForEach-Object Name) -join ', ')"
    $Stack = Demander "Stack"
}
$source = Join-Path $dossierStacks $Stack
if (-not ($stacks | Where-Object Name -eq $Stack)) {
    Arreter "la stack « $Stack » n'existe pas. Disponibles : $(($stacks | ForEach-Object Name) -join ', ')."
}

if (-not $Promesse) { $Promesse = Demander "Promesse de l'appli, en une phrase" }
Verifier-Texte "la promesse" $Promesse

if (-not $ElementOrange) { $ElementOrange = Demander "Élément orange du logo (exemple : une boussole)" }
Verifier-Texte "l'élément orange" $ElementOrange

if (-not $DemarrageAuto) {
    $reponse = Demander "L'appli sert-elle en arrière-plan, avec démarrage automatique ? (o/N)"
    if ($reponse -match '^[oOyY]') { $DemarrageAuto = 'oui' } else { $DemarrageAuto = 'non' }
}

if (-not $Installation) {
    $reponse = Demander "L'exe propose-t-il de s'installer dans le dossier de données (appli de bureau) ? (O/n)"
    if ($reponse -match '^[nN]') { $Installation = 'non' } else { $Installation = 'oui' }
}

if (-not $PSBoundParameters.ContainsKey('Port')) {
    $reponse = Demander "Port local de l'interface web (vide si l'appli n'en a pas)"
    if ($reponse) {
        if ($reponse -notmatch '^\d+$') { Arreter "le port « $reponse » n'est pas un nombre." }
        $Port = [int]$reponse
    }
}
if ($Port -ne 0) {
    if ($Port -lt 1024 -or $Port -gt 65535) { Arreter "le port $Port est hors de la plage 1024 à 65535." }
    $registre = Join-Path $racine 'docs\PORTS.md'
    if (Test-Path -LiteralPath $registre) {
        $pris = @(Get-Content -LiteralPath $registre -Encoding UTF8 |
            ForEach-Object { if ($_ -match '^\|\s*(\d+)\s*\|') { [int]$Matches[1] } })
        if ($pris -contains $Port) {
            Arreter "le port $Port est déjà pris par une autre appli Aria (voir docs\PORTS.md)."
        }
    }
}

# ----- 1. La stack choisie passe à la racine -----

$reglages = Get-Content -LiteralPath (Join-Path $source '_socle\stack.json') -Raw -Encoding UTF8 | ConvertFrom-Json
$nomStack = $Stack
if ($reglages.PSObject.Properties['nom']) { $nomStack = [string]$reglages.nom }

$fragments = @{}
Get-ChildItem -LiteralPath (Join-Path $source '_socle') -Filter '*.txt' | ForEach-Object {
    $fragments[$_.BaseName] = (Get-Content -LiteralPath $_.FullName -Raw -Encoding UTF8).TrimEnd("`r", "`n")
}

Get-ChildItem -LiteralPath $source -Force | Where-Object Name -ne '_socle' | ForEach-Object {
    Copy-Item -LiteralPath $_.FullName -Destination $racine -Recurse -Force
}
Remove-Item -LiteralPath $dossierStacks -Recurse -Force
$essais = Join-Path $racine 'tools\essais'
if (Test-Path -LiteralPath $essais) { Remove-Item -LiteralPath $essais -Recurse -Force }
# Les tests des squelettes ne servent qu'au dépôt modèle.
foreach ($relatif in 'tools\tester-squelettes.ps1', '.github\workflows\squelettes.yml') {
    $chemin = Join-Path $racine $relatif
    if (Test-Path -LiteralPath $chemin) { Remove-Item -LiteralPath $chemin -Force }
}

# ----- 2. Blocs propres à la stack -----

$fichiersSocle = @(
    '.gitignore', '.gitattributes', '.github\dependabot.yml',
    '.github\workflows\tests.yml', '.github\workflows\release.yml', '.github\workflows\codeql.yml'
)
foreach ($relatif in $fichiersSocle) {
    $chemin = Join-Path $racine $relatif
    if (-not (Test-Path -LiteralPath $chemin)) { continue }
    $lu = Lire-Texte $chemin
    $finDeLigne = "`n"
    if ($lu.Texte.Contains("`r`n")) { $finDeLigne = "`r`n" }
    $lignes = New-Object Collections.Generic.List[string]
    foreach ($ligne in ($lu.Texte -split "`r?`n")) {
        if ($ligne -match '<<STACK:([a-z-]+)>>') {
            if ($fragments.ContainsKey($Matches[1])) {
                foreach ($l in ($fragments[$Matches[1]] -split "`r?`n")) { $lignes.Add($l) }
            }
        }
        else { $lignes.Add($ligne) }
    }
    $texte = $lignes -join $finDeLigne
    if ($relatif -like '*tests.yml' -and $reglages.PSObject.Properties['runsOn'] -and $reglages.runsOn) {
        $texte = $texte.Replace('runs-on: windows-latest', "runs-on: $($reglages.runsOn)")
    }
    Ecrire-Texte $chemin $texte $lu.Bom
}

# ----- 3. Marqueurs, dans tous les fichiers texte -----

# PowerShell ne distingue pas la casse : ce nom ne doit pas être « port », qui écraserait le paramètre.
$portTexte = 'aucun'
if ($Port -ne 0) { $portTexte = [string]$Port }
$demarrage = 'non'
if ($DemarrageAuto -eq 'oui') { $demarrage = 'oui, désactivé par défaut' }
# Blocs du README et des notes de version : on garde celui qui correspond à la réponse, on retire l'autre.
$installationTexte = 'non : exe portable seulement'
$blocGarde = 'PORTABLE'
$blocRetire = 'INSTALLATION'
if ($Installation -eq 'oui') {
    $installationTexte = "oui : l'exe propose de se ranger dans le dossier ``programme\`` du dossier de données"
    $blocGarde = 'INSTALLATION'
    $blocRetire = 'PORTABLE'
}

$valeurs = [ordered]@{
    '{{NOM_MIN}}'        = $Nom.ToLowerInvariant()
    '{{NOM}}'            = $Nom
    '{{PROMESSE}}'       = $Promesse
    '{{ELEMENT_ORANGE}}' = $ElementOrange
    '{{PORT}}'           = $portTexte
    '{{ANNEE}}'          = [string](Get-Date).Year
    '{{STACK}}'          = $nomStack
    '{{DEMARRAGE_AUTO}}' = $demarrage
    '{{INSTALLATION}}'   = $installationTexte
    'Aria-Modele'        = "Aria-$Nom"
}
$binaires = '.png', '.ico', '.gz', '.dll', '.ttf', '.woff2', '.zip', '.exe', '.jpg', '.jpeg', '.syso', '.pdf'
$moi = $PSCommandPath

# Seulement les fichiers que git suit ou suivrait : ce que .gitignore écarte (.work, dépendances, sorties de build)
# n'appartient pas au modèle et n'est pas réécrit.
$suivis = @(git -c core.quotepath=false ls-files --cached --others --exclude-standard)
if ($LASTEXITCODE -ne 0) { Arreter "git n'a pas pu lister les fichiers du dépôt." }

$suivis | ForEach-Object { Join-Path $racine $_ } |
    Where-Object { Test-Path -LiteralPath $_ -PathType Leaf } |
    ForEach-Object { Get-Item -LiteralPath $_ -Force } |
    Where-Object { $binaires -notcontains $_.Extension.ToLowerInvariant() -and $_.FullName -ne $moi } |
    ForEach-Object {
        $lu = Lire-Texte $_.FullName
        if ($null -eq $lu) { return }
        $texte = $lu.Texte
        # Blocs réservés au dépôt modèle (README.md)
        $texte = [regex]::Replace($texte, '(?s)<!-- MODELE:debut -->.*?<!-- MODELE:fin -->\r?\n?', '')
        # Installation par l'exe (README.md, notes de version) : un bloc retiré en entier, l'autre perd ses balises.
        $texte = [regex]::Replace($texte, "(?s)<!-- ${blocRetire}:debut -->.*?<!-- ${blocRetire}:fin -->\r?\n?", '')
        $texte = [regex]::Replace($texte, "<!-- ${blocGarde}:(debut|fin) -->\r?\n?", '')
        foreach ($cle in $valeurs.Keys) { $texte = $texte.Replace($cle, $valeurs[$cle]) }
        if ($texte -cne $lu.Texte) { Ecrire-Texte $_.FullName $texte $lu.Bom }
    }

# ----- 4. Fichiers et dossiers qui portent « Aria-Modele » -----

Get-ChildItem -LiteralPath $racine -Recurse -Force |
    Where-Object { $_.FullName -notmatch '\\\.git(\\|$)' -and $_.Name -like '*Aria-Modele*' } |
    Sort-Object { $_.FullName.Length } -Descending |
    ForEach-Object { Rename-Item -LiteralPath $_.FullName -NewName ($_.Name.Replace('Aria-Modele', "Aria-$Nom")) }

# Le registre des ports fait foi dans le dépôt modèle : il n'est pas copié dans l'appli.
$registre = Join-Path $racine 'docs\PORTS.md'
if (Test-Path -LiteralPath $registre) { Remove-Item -LiteralPath $registre -Force }

# ----- 5. Tests de la stack -----

Write-Host "Tests de la stack « $nomStack »..."
& (Join-Path $racine 'tools\tester.ps1')
if ($LASTEXITCODE -ne 0) {
    Arreter "les tests ont échoué. init.ps1 n'a pas été supprimé : corrige, ou annule avec « git restore . » puis « git clean -fd »."
}

# ----- 6. Le script se supprime et dit la suite -----

Remove-Item -LiteralPath $moi -Force

Write-Host ""
Write-Host "Aria $Nom est prête." -ForegroundColor Cyan
Write-Host "La suite :"
Write-Host "  1. Relis ce qui a changé : git status"
Write-Host "  2. Remplace docs\logo.png, docs\logo.svg et l'icône de la stack."
Write-Host "  3. Complète les lignes « à renseigner » de la fiche projet, dans docs\FICHE.md."
Write-Host "  4. Fais le premier commit sur une branche init/$($Nom.ToLowerInvariant()) et amène-le par une PR."
Write-Host "  5. Après le premier passage vert de la CI sur main : .\tools\proteger-depot.ps1"
if ($Port -ne 0) {
    Write-Host "  6. Ajoute le port $Port au registre docs\PORTS.md du dépôt modèle."
}
