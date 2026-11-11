<#
Administre le serveur de Fight of Pions depuis ce PC, par SSH (docs/DEPLOIEMENT.md, E4i).

    .\outils\admin.ps1                                   la liste des commandes
    .\outils\admin.ps1 comptes
    .\outils\admin.ps1 donner 3 MassacreDivin Legendaire
    .\outils\admin.ps1 sauvegarder

Le serveur se règle une fois, dans la variable FIGHT_SERVEUR (utilisateur@adresse) :

    [Environment]::SetEnvironmentVariable("FIGHT_SERVEUR", "utilisateur@adresse-du-serveur", "User")

puis rouvrir le terminal. Sans clé SSH, le mot de passe est demandé à chaque commande.
Un compte se désigne ici par son seul numéro (3) : dans PowerShell, #3 commence un
commentaire.
Fichier enregistré en UTF-8 avec BOM : Windows PowerShell 5.1 lit ainsi ses accents.
#>

$serveur = $env:FIGHT_SERVEUR
if (-not $serveur) {
    Write-Error "Le serveur n'est pas réglé : voir l'en-tête de ce script (variable FIGHT_SERVEUR)."
    exit 1
}
# Chaque mot passe tel quel, même avec des espaces (un pseudo, « massacre divin ») : il est
# mis entre apostrophes pour le shell du serveur. Un guillemet est protégé à part :
# Windows PowerShell 5.1 ne le transmet pas tel quel à ssh.
$mots = $args | ForEach-Object {
    "'" + (([string]$_ -replace "'", "'\''") -replace '"', '\"') + "'"
}
$commande = "docker compose -f /opt/fight-of-pions/docker-compose.yml exec -T serveur serveur admin " + ($mots -join " ")
ssh $serveur $commande
exit $LASTEXITCODE
