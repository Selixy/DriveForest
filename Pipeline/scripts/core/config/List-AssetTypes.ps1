# Script pour afficher les types d'assets disponibles

Write-Host ""
Write-Host "========================================================"
Write-Host "  Types d'Assets Disponibles"
Write-Host "========================================================"
Write-Host ""

$types = @(
    @{
        Name = "Character"
        Description = "Personnages jouables, PNJ, creatures"
    },
    @{
        Name = "Prop"
        Description = "Objets interactifs, meubles, elements decoratifs"
    },
    @{
        Name = "Vehicle"
        Description = "Vehicules, velos, engins"
    }
)

$types | ForEach-Object {
    Write-Host "  * $($_.Name)"
    Write-Host "    $($_.Description)"
}

Write-Host ""

