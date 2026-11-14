<#
.SYNOPSIS
Compile Aria {{NOM}} en un seul exe autonome, dans .work\sortie\.

.DESCRIPTION
Produit Aria-{{NOM}}.exe, sans rien d'autre à installer (le runtime .NET est embarqué). Ne demande pas les droits
administrateur et ne lance rien. Les versions des outils vont dans .work\outils.txt, que tools\empaqueter.ps1 recopie
dans BUILDINFO.txt.

    .\build.ps1
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'
$env:DOTNET_CLI_TELEMETRY_OPTOUT = '1'
$env:DOTNET_NOLOGO = '1'

$racine = $PSScriptRoot
if (-not (Get-Command dotnet -ErrorAction SilentlyContinue)) { throw "dotnet est introuvable : installe le SDK .NET 10." }

$work = Join-Path $racine '.work'
$sortie = Join-Path $work 'sortie'
# Une jonction à la place de .work ou de .work\sortie mènerait la suppression qui suit hors du dépôt.
foreach ($d in $work, $sortie) {
    if ((Test-Path -LiteralPath $d) -and ((Get-Item -LiteralPath $d -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "$d est un lien ou une jonction : rien n'est supprimé. Retire-le, puis relance."
    }
}
if (Test-Path -LiteralPath $sortie) { Remove-Item -LiteralPath $sortie -Recurse -Force }
New-Item -ItemType Directory -Path $sortie -Force | Out-Null

# Sous PowerShell 5.1, une sortie d'erreur de dotnet redirigée arrêterait le script : on lit son code de retour.
$ErrorActionPreference = 'Continue'
$sdk = "$(& dotnet --version 2>$null)".Trim()
& dotnet publish (Join-Path $racine 'src\Aria.Gui\Aria.Gui.csproj') -c Release -o $sortie
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($code -ne 0) { throw "La compilation d'Aria {{NOM}} a échoué." }

$exe = Join-Path $sortie 'Aria-{{NOM}}.exe'
if (-not (Test-Path -LiteralPath $exe)) { throw "Aria-{{NOM}}.exe est absent de .work\sortie\ après la compilation." }
# Seul l'exe est livré : les fichiers de débogage restent hors du zip.
Get-ChildItem -LiteralPath $sortie -Filter '*.pdb' | Remove-Item -Force

[IO.File]::WriteAllLines((Join-Path $work 'outils.txt'), [string[]]@("SDK .NET : $sdk"), (New-Object Text.UTF8Encoding($false)))
Write-Host "Prêt : $exe ($([math]::Round((Get-Item -LiteralPath $exe).Length / 1MB, 1)) Mo)"
