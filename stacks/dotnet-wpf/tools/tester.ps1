<#
.SYNOPSIS
Lance les tests d'Aria {{NOM}} (xUnit, tests\Aria.Tests).

.DESCRIPTION
Code de retour non nul si un test échoue ou si la compilation casse.

    .\tools\tester.ps1
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
$env:DOTNET_NOLOGO = '1'

$racine = Split-Path -Parent $PSScriptRoot
if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) { throw "dotnet est introuvable : installe le SDK .NET 10." }

# Sous PowerShell 5.1, une sortie d'erreur de dotnet redirigée arrêterait le script : on lit son code de retour.
$ErrorActionPreference = 'Continue'
& dotnet test (Join-Path $racine 'tests\Aria.Tests\Aria.Tests.csproj') -c Release
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'

if ($code -ne 0) {
    Write-Host "Les tests ont échoué." -ForegroundColor Red
    exit 1
}
Write-Host "Tous les tests passent." -ForegroundColor Cyan
exit 0
