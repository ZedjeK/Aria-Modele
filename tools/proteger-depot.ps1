<#
.SYNOPSIS
Pose les protections GitHub d'une appli Aria : fusion en squash, rulesets, signalement privé, alertes.

.DESCRIPTION
À lancer après le premier passage vert de la CI sur main : on n'exige jamais un check qui n'a pas encore tourné.

Le script lit l'état du dépôt, affiche ce qu'il changerait, et n'écrit qu'après ton accord. L'état d'avant est
enregistré dans .work\protection-avant.json, pour pouvoir revenir en arrière.

    .\tools\proteger-depot.ps1              affiche le plan et demande confirmation
    .\tools\proteger-depot.ps1 -Depot ZedjeK/Aria-Atlas

Il a besoin de gh, connecté avec un compte qui administre le dépôt.
#>
[CmdletBinding()]
param(
    [string]$Depot
)

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = Split-Path -Parent $PSScriptRoot

if (-not (Get-Command gh -ErrorAction SilentlyContinue)) { throw "gh est introuvable." }
if (-not $Depot) { $Depot = "$(gh repo view --json nameWithOwner --jq .nameWithOwner)".Trim() }
if ($Depot -notmatch '^[\w.-]+/[\w.-]+$') { throw "Dépôt introuvable. Précise-le : -Depot proprietaire/nom" }

# Appel en lecture : renvoie l'objet JSON, ou $null si GitHub répond par une erreur (404 compris).
function Lire([string]$chemin) {
    # Sous PowerShell 5.1, une sortie d'erreur de gh redirigée arrêterait le script : ici, un refus est une réponse.
    $ErrorActionPreference = 'Continue'
    $sortie = gh api $chemin 2>$null
    if ($LASTEXITCODE -ne 0) { return $null }
    if (-not $sortie) { return $true } # réponse vide (204) : la ressource existe
    return ($sortie | ConvertFrom-Json)
}

# Appel en écriture, avec un corps JSON passé par fichier.
function Ecrire([string]$methode, [string]$chemin, $corps) {
    if ($null -eq $corps) {
        gh api -X $methode $chemin | Out-Null
    }
    else {
        $fichier = Join-Path $racine '.work\protection-corps.json'
        [IO.File]::WriteAllText($fichier, ($corps | ConvertTo-Json -Depth 10), (New-Object Text.UTF8Encoding($false)))
        gh api -X $methode $chemin --input $fichier | Out-Null
        Remove-Item -LiteralPath $fichier -Force
    }
    if ($LASTEXITCODE -ne 0) { throw "GitHub a refusé : $methode $chemin" }
}

# ----- État des lieux, en lecture seule -----

Write-Host "Dépôt : $Depot"
$depotInfo = Lire "repos/$Depot"
if ($null -eq $depotInfo) { throw "Impossible de lire le dépôt $Depot." }
$branche = [string]$depotInfo.default_branch
$rulesets = @(Lire "repos/$Depot/rulesets")
$reglesBranche = @(Lire "repos/$Depot/rules/branches/$branche")

New-Item -ItemType Directory -Path (Join-Path $racine '.work') -Force | Out-Null
$avant = [ordered]@{ date = (Get-Date).ToString('s'); depot = $depotInfo; rulesets = $rulesets; regles = $reglesBranche }
[IO.File]::WriteAllText((Join-Path $racine '.work\protection-avant.json'), ($avant | ConvertTo-Json -Depth 12), (New-Object Text.UTF8Encoding($false)))

# Le check « tests » a-t-il réussi sur la branche par défaut ?
$checks = Lire "repos/$Depot/commits/$branche/check-runs"
$testsVerts = $false
if ($checks -and $checks.PSObject.Properties['check_runs']) {
    $testsVerts = [bool](@($checks.check_runs | Where-Object { $_.name -eq 'tests' -and $_.conclusion -eq 'success' }).Count)
}

$plan = New-Object Collections.Generic.List[object]
function Prevoir([string]$titre, [scriptblock]$action) {
    $script:plan.Add([pscustomobject]@{ Titre = $titre; Action = $action })
}

# 1. Méthode de fusion
$fusionOk = $depotInfo.allow_squash_merge -and -not $depotInfo.allow_merge_commit -and -not $depotInfo.allow_rebase_merge -and
    $depotInfo.delete_branch_on_merge -and $depotInfo.allow_auto_merge
if ($fusionOk) { Write-Host "  déjà en place : fusion en squash seulement, branche supprimée après fusion" }
else {
    Prevoir "Fusion : squash seulement, branche supprimée après fusion, fusion automatique autorisée" {
        Ecrire 'PATCH' "repos/$Depot" @{
            allow_squash_merge = $true; allow_merge_commit = $false; allow_rebase_merge = $false
            delete_branch_on_merge = $true; allow_auto_merge = $true
        }
    }
}

# 2. Ruleset de la branche par défaut
$types = @($reglesBranche | Where-Object { $_ -and $_.PSObject.Properties['type'] } | ForEach-Object { $_.type })
$checkExige = @($reglesBranche | Where-Object { $_ -and $_.PSObject.Properties['type'] -and $_.type -eq 'required_status_checks' } |
    ForEach-Object { $_.parameters.required_status_checks.context }) -contains 'tests'
if ($types -contains 'deletion' -and $types -contains 'non_fast_forward' -and $checkExige) {
    Write-Host "  déjà en place : $branche protégée, check « tests » exigé"
}
elseif ($types.Count -gt 0) {
    Write-Host "  À FAIRE À LA MAIN : $branche a déjà des règles ($($types -join ', ')). Complète le ruleset existant plutôt que d'en créer un second." -ForegroundColor Yellow
}
elseif (-not $testsVerts) {
    Write-Host "  PAS ENCORE : le check « tests » n'a pas réussi sur $branche. Relance ce script après le premier passage vert." -ForegroundColor Yellow
}
else {
    Prevoir "Ruleset « Protection de $branche » : suppression et réécriture interdites, check « tests » exigé" {
        Ecrire 'POST' "repos/$Depot/rulesets" @{
            name = "Protection de $branche"; target = 'branch'; enforcement = 'active'
            conditions = @{ ref_name = @{ include = @('~DEFAULT_BRANCH'); exclude = @() } }
            rules = @(
                @{ type = 'deletion' },
                @{ type = 'non_fast_forward' },
                @{ type = 'required_status_checks'; parameters = @{
                        strict_required_status_checks_policy = $false
                        required_status_checks = @(@{ context = 'tests' })
                    }
                }
            )
        }
    }
}

# 3. Ruleset des tags de version
if (@($rulesets | Where-Object { $_ -and $_.PSObject.Properties['target'] -and $_.target -eq 'tag' }).Count -gt 0) {
    Write-Host "  déjà en place : un ruleset de tags existe"
}
else {
    Prevoir "Ruleset « Tags de version » : un tag v* ne peut être ni supprimé ni déplacé" {
        Ecrire 'POST' "repos/$Depot/rulesets" @{
            name = 'Tags de version'; target = 'tag'; enforcement = 'active'
            conditions = @{ ref_name = @{ include = @('refs/tags/v*'); exclude = @() } }
            rules = @(@{ type = 'deletion' }, @{ type = 'update' })
        }
    }
}

# 4. Signalement privé des failles
$signalement = Lire "repos/$Depot/private-vulnerability-reporting"
if ($signalement -and $signalement.PSObject.Properties['enabled'] -and $signalement.enabled) {
    Write-Host "  déjà en place : signalement privé des failles"
}
else {
    Prevoir "Signalement privé des failles : activé" { Ecrire 'PUT' "repos/$Depot/private-vulnerability-reporting" $null }
}

# 5. Alertes Dependabot
if (Lire "repos/$Depot/vulnerability-alerts") { Write-Host "  déjà en place : alertes Dependabot" }
else { Prevoir "Alertes Dependabot : activées" { Ecrire 'PUT' "repos/$Depot/vulnerability-alerts" $null } }

# 6. Secret scanning et push protection
$analyse = $null
if ($depotInfo.PSObject.Properties['security_and_analysis']) { $analyse = $depotInfo.security_and_analysis }
$secretsOk = $analyse -and $analyse.PSObject.Properties['secret_scanning'] -and $analyse.secret_scanning.status -eq 'enabled' -and
    $analyse.PSObject.Properties['secret_scanning_push_protection'] -and $analyse.secret_scanning_push_protection.status -eq 'enabled'
if ($secretsOk) { Write-Host "  déjà en place : secret scanning et push protection" }
else {
    Prevoir "Secret scanning et push protection : activés" {
        Ecrire 'PATCH' "repos/$Depot" @{ security_and_analysis = @{
                secret_scanning = @{ status = 'enabled' }
                secret_scanning_push_protection = @{ status = 'enabled' }
            }
        }
    }
}

# ----- Plan, accord, application -----

Write-Host ""
Write-Host "À faire à la main, dans les réglages du dépôt : activer les releases immuables."
Write-Host "État d'avant enregistré dans .work\protection-avant.json."
Write-Host ""

if ($plan.Count -eq 0) {
    Write-Host "Rien à changer." -ForegroundColor Cyan
    exit 0
}

Write-Host "Ce script va faire $($plan.Count) changement(s) sur $Depot :"
$n = 0
foreach ($etape in $plan) { $n++; Write-Host "  $n. $($etape.Titre)" }
Write-Host ""
$reponse = (Read-Host "Appliquer ? (o/N)").Trim()
if ($reponse -notmatch '^[oO]') {
    Write-Host "Rien n'a été modifié."
    exit 0
}

foreach ($etape in $plan) {
    Write-Host "  $($etape.Titre)..."
    & $etape.Action
}
Write-Host "Terminé. Relance le script pour vérifier : il doit répondre « Rien à changer »." -ForegroundColor Cyan
