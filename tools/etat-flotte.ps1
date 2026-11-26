<#
.SYNOPSIS
État de la flotte : pour chaque appli publique de la famille, retard sur le socle, PR Dependabot ouvertes, ruleset de
la branche par défaut, dernière analyse CodeQL, dernière version publiée.

.DESCRIPTION
Fichier du dépôt modèle seulement : init.ps1 le retire de l'appli créée.

Lecture seule : le script ne fait que des lectures (GET) de l'API de GitHub par gh, et n'écrit rien, ni sur GitHub ni
sur le disque. Les applis sont les dépôts du propriétaire du modèle dont le nom commence par « Aria- », publics et non
archivés, modèle exclu. Un dépôt privé est écarté par sa visibilité, sans être lu ni nommé.

Le socle de référence est celui du modèle sur GitHub, à la pointe de sa branche par défaut. Une valeur qui ne se lit
pas (droits, API indisponible) s'affiche « non lu ».

    .\tools\etat-flotte.ps1
    .\tools\etat-flotte.ps1 | Format-Table -AutoSize
    .\tools\etat-flotte.ps1 -Modele proprietaire/nom
#>
#Requires -Version 5.1
[CmdletBinding()]
param(
    # Le dépôt modèle, « propriétaire/nom » : son socle sert de référence, son propriétaire donne la liste des applis.
    [ValidatePattern('\A[\w.-]+/[\w.-]+\z')]
    [string]$Modele = 'ZedjeK/Aria-Modele'
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { throw 'gh est introuvable : ce script lit GitHub par gh.' }

# Une lecture de l'API de GitHub (GET seulement) ; les lignes de la réponse, ou $null si la lecture échoue. Sous
# PowerShell 5.1, une sortie d'erreur de gh redirigée arrêterait le script : on lit son code de retour à la place.
function Lire-GitHub([string]$adresse, [string]$filtre) {
    $arguments = @('api', '--method', 'GET', $adresse)
    if ($filtre) { $arguments += '--jq', $filtre }
    $ErrorActionPreference = 'Continue'
    $tout = @(& gh @arguments 2>&1)
    $code = $LASTEXITCODE
    $ErrorActionPreference = 'Stop'
    if ($code -ne 0) { return $null }
    return , @($tout | Where-Object { $_ -isnot [Management.Automation.ErrorRecord] } | ForEach-Object { "$_" })
}

# Le texte d'un fichier d'un dépôt, fins de ligne ramenées à LF ; $null s'il n'existe pas ou ne se lit pas. Le contenu
# arrive en base64 : aucun accent ne dépend de l'encodage de la console.
function Lire-Fichier([string]$depot, [string]$chemin) {
    $lignes = Lire-GitHub "repos/$depot/contents/$chemin" '.content'
    if ($null -eq $lignes) { return $null }
    $octets = [Convert]::FromBase64String(($lignes -join ''))
    return (New-Object Text.UTF8Encoding($false)).GetString($octets).Replace("`r`n", "`n")
}

# Le socle du modèle : la liste, puis le texte de chaque fichier.
$texteListe = Lire-Fichier $Modele 'tools/socle.txt'
if ($null -eq $texteListe) { throw "Le socle du modèle ne se lit pas : tools/socle.txt dans $Modele." }
$socle = [ordered]@{}
foreach ($chemin in ($texteListe.TrimStart([char]0xFEFF) -split "`n" | ForEach-Object { $_.Trim() } |
            Where-Object { $_ -and -not $_.StartsWith('#') })) {
    $texte = Lire-Fichier $Modele $chemin
    if ($null -eq $texte) { throw "Le modèle liste $chemin dans son socle, mais ne l'a pas." }
    $socle[$chemin] = $texte
}

# Les éléments d'une réponse en tableau, un objet par élément ; $null si la lecture échoue. Les filtres passés à gh
# n'ont jamais de guillemets doubles : Windows PowerShell 5.1 les perdrait en appelant un programme natif. Le tri se
# fait ici.
function Lire-Liste([string]$adresse) {
    $lignes = Lire-GitHub $adresse '.[]'
    if ($null -eq $lignes) { return $null }
    return , @($lignes | Where-Object { $_ } | ForEach-Object { $_ | ConvertFrom-Json })
}

# Les applis : dépôts publics et non archivés du propriétaire, « Aria-* », modèle exclu.
$proprietaire = $Modele.Split('/')[0]
$depots = Lire-Liste "users/$proprietaire/repos?type=owner&per_page=100"
if ($null -eq $depots) { throw "La liste des dépôts de $proprietaire ne se lit pas." }
$applis = @($depots | Where-Object {
        -not $_.private -and -not $_.archived -and $_.name.StartsWith('Aria-') -and $_.full_name -ne $Modele
    } | Sort-Object name)

foreach ($appli in $applis) {
    $depot = $appli.full_name

    # Socle : les fichiers absents ou différents de ceux du modèle.
    $retard = @($socle.Keys | Where-Object { (Lire-Fichier $depot $_) -cne $socle[$_] })
    $etatSocle = if ($retard.Count -eq 0) { 'à jour' } else { "$($retard.Count) en retard" }

    $demandes = Lire-Liste "repos/$depot/pulls?state=open&per_page=100"
    $dependabot = if ($null -eq $demandes) { 'non lu' }
    else { "$(@($demandes | Where-Object { $_.user.login -eq 'dependabot[bot]' }).Count) PR" }

    $regles = Lire-Liste "repos/$depot/rules/branches/$($appli.default_branch)"
    $exiges = @($regles | Where-Object { $_ -and $_.type -eq 'required_status_checks' } |
        ForEach-Object { $_.parameters.required_status_checks } | ForEach-Object { $_.context })
    $etatRuleset = if ($null -eq $regles) { 'non lu' }
    elseif (@($regles).Count -eq 0) { 'aucun' }
    elseif ($exiges -contains 'tests') { 'tests exigé' }
    else { 'tests non exigé' }

    $analyses = Lire-Liste "repos/$depot/code-scanning/analyses?per_page=1"
    $codeql = if ($null -eq $analyses) { 'non lu' }
    elseif (@($analyses).Count -eq 0) { 'aucune' }
    else { "$(@($analyses)[0].created_at)" -replace 'T.*$', '' }

    $version = Lire-GitHub "repos/$depot/releases/latest" '.tag_name'

    [pscustomobject]@{
        Appli      = $appli.name
        Stack      = if ($appli.language) { $appli.language } else { 'inconnue' }
        Version    = if ($null -eq $version) { 'aucune' } else { $version -join '' }
        Socle      = $etatSocle
        Dependabot = $dependabot
        Ruleset    = $etatRuleset
        CodeQL     = $codeql
    }
}
