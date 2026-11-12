# Stack d'essai : « compile » un fichier texte, pour éprouver la chaîne sans aucun outil.
$sortie = Join-Path $PSScriptRoot '.work\sortie'
New-Item -ItemType Directory -Path $sortie -Force | Out-Null
Set-Content -LiteralPath (Join-Path $sortie 'Aria-{{NOM}}.txt') -Value 'Aria {{NOM}}' -Encoding UTF8
