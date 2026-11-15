<#
.SYNOPSIS
Compile Aria {{NOM}} en un exe autonome, dans .work\sortie\.

.DESCRIPTION
PyInstaller (pyinstaller==6.22.3) empaquète l'appli, son interface, le logo et pyproject.toml (la version) en un seul exe,
sans console : rien d'autre à installer. L'exe se décompresse dans cache\ du dossier de données
(--runtime-tmpdir), jamais dans %TEMP%. Ne demande pas les droits administrateur et ne lance rien. Les versions des outils
vont dans .work\outils.txt, que tools\empaqueter.ps1 recopie dans BUILDINFO.txt.

    .\build.ps1
#>
[CmdletBinding()]
param()

Set-StrictMode -Version Latest
$ErrorActionPreference = 'Stop'

$racine = $PSScriptRoot
$python = Join-Path $racine '.venv\Scripts\python.exe'
if (-not (Test-Path -LiteralPath $python)) {
    $commande = Get-Command python -ErrorAction SilentlyContinue
    if (-not $commande) { throw "Python est introuvable : installe Python 3.11 ou plus (voir requirements.txt)." }
    $python = $commande.Source
}
$icone = Join-Path $racine 'Aria-Modele.ico'
if (-not (Test-Path -LiteralPath $icone)) { throw "L'icône Aria-Modele.ico est introuvable à la racine." }

$work = Join-Path $racine '.work'
$sortie = Join-Path $work 'sortie'
$pyinstaller = Join-Path $work 'pyinstaller'
# Une jonction à la place de .work ou de ses sous-dossiers mènerait la suppression qui suit hors du dépôt.
foreach ($d in $work, $sortie, $pyinstaller) {
    if ((Test-Path -LiteralPath $d) -and ((Get-Item -LiteralPath $d -Force).Attributes -band [IO.FileAttributes]::ReparsePoint)) {
        throw "$d est un lien ou une jonction : rien n'est supprimé. Retire-le, puis relance."
    }
}
foreach ($d in $sortie, $pyinstaller) {
    if (Test-Path -LiteralPath $d) { Remove-Item -LiteralPath $d -Recurse -Force }
}
New-Item -ItemType Directory -Path $sortie -Force | Out-Null

# Sous PowerShell 5.1, une sortie d'erreur d'un programme redirigée arrêterait le script : on lit son code de retour.
$ErrorActionPreference = 'Continue'
$versions = @(
    "$(& $python --version 2>&1)".Trim(),
    "PyInstaller : $("$(& $python -m PyInstaller --version 2>&1)".Trim())",
    "FastAPI : $("$(& $python -c 'import fastapi; print(fastapi.__version__)' 2>&1)".Trim())",
    "uvicorn : $("$(& $python -c 'import uvicorn; print(uvicorn.__version__)' 2>&1)".Trim())"
)
& $python -m PyInstaller --noconfirm --clean --onefile --noconsole `
    --name Aria-{{NOM}} `
    --icon $icone `
    --paths $racine `
    --runtime-tmpdir '%USERPROFILE%\.Aria-{{NOM}}\cache' `
    --collect-submodules uvicorn `
    --specpath $pyinstaller --workpath $pyinstaller --distpath $sortie `
    --add-data "$racine\appli\statique;appli\statique" `
    --add-data "$racine\docs\logo.svg;docs" `
    --add-data "$racine\pyproject.toml;." `
    (Join-Path $racine 'appli\__main__.py')
$code = $LASTEXITCODE
$ErrorActionPreference = 'Stop'
if ($code -ne 0) { throw "La construction d'Aria {{NOM}} a échoué." }

$exe = Join-Path $sortie 'Aria-{{NOM}}.exe'
if (-not (Test-Path -LiteralPath $exe)) { throw "Aria-{{NOM}}.exe est absent de .work\sortie\ après la construction." }

[IO.File]::WriteAllLines((Join-Path $work 'outils.txt'), [string[]]$versions, (New-Object Text.UTF8Encoding($false)))
Write-Host "Prêt : $exe ($([math]::Round((Get-Item -LiteralPath $exe).Length / 1MB, 1)) Mo)"
