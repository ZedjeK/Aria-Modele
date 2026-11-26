# Réglages de PSScriptAnalyzer pour les scripts du dépôt : l'étape « Analyse des scripts PowerShell » du job tests
# les applique à tous les .ps1. Fichier du socle commun : le même dans toutes les applis.
#
# Une remarque se corrige. Si elle ne peut pas l'être, elle s'exclut ligne par ligne dans le script
# ([Diagnostics.CodeAnalysis.SuppressMessageAttribute()] avec la raison), jamais ici sans raison écrite.
@{
    # Les remarques de niveau Information ne bloquent pas : la seule qui sort sur ce dépôt,
    # PSAvoidUsingPositionalParameters, vise aussi les appels à des programmes natifs comme npm, qui n'ont pas de
    # paramètres nommés.
    Severity     = @('Error', 'Warning')

    ExcludeRules = @(
        # Les fonctions internes d'un script gardent leurs noms français (Verifier, Nouvelle-Copie…) : les verbes
        # approuvés ne s'imposent qu'aux fonctions exportées d'un module, et le dépôt n'en a pas.
        'PSUseApprovedVerbs'
        # Les scripts parlent à la personne qui les lance : Write-Host est voulu pour ces messages, jamais pour
        # renvoyer une donnée.
        'PSAvoidUsingWriteHost'
    )
}
