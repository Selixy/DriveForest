# Menu interactif pour la création d'assets

function Show-Menu {
    param(
        [string]$Title = 'Menu Création d''Assets',
        [array]$Options
    )
    
    Clear-Host
    Write-Host ""
    Write-Host "╔════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║  $Title" -ForegroundColor Cyan
    Write-Host "╚════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
    
    for ($i = 0; $i -lt $Options.Count; $i++) {
        Write-Host "  $($i + 1). $($Options[$i])" -ForegroundColor Yellow
    }
    Write-Host "  0. Quitter" -ForegroundColor Red
    Write-Host ""
}

function Get-AssetTypeMenu {
    $assetTypes = @("Character", "Environment", "Prop", "Vehicle", "Material")
    
    do {
        Show-Menu -Title "Sélectionner le Type d'Asset" -Options $assetTypes
        $selection = Read-Host "Choix"
        
        if ($selection -eq "0") {
            return $null
        }
        
        if ([int]$selection -gt 0 -and [int]$selection -le $assetTypes.Count) {
            return $assetTypes[[int]$selection - 1]
        }
        
        Write-Host "Sélection invalide. Appuyez sur une touche..." -ForegroundColor Red
        Read-Host
    } while ($true)
}

# Menu principal
$scriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$projectRoot = Split-Path -Parent $scriptDir

do {
    Clear-Host
    Write-Host ""
    Write-Host "╔════════════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "║    🎨 DriveForest - Générateur d'Assets  ║" -ForegroundColor Cyan
    Write-Host "╚════════════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
    Write-Host "  1. Créer un nouvel asset" -ForegroundColor Green
    Write-Host "  2. Afficher les types disponibles" -ForegroundColor Green
    Write-Host "  0. Quitter" -ForegroundColor Red
    Write-Host ""
    
    $choice = Read-Host "Votre choix"
    
    switch ($choice) {
        "1" {
            Write-Host ""
            $assetName = Read-Host "Entrez le nom de l'asset"
            
            if ([string]::IsNullOrWhiteSpace($assetName)) {
                Write-Host "❌ Le nom de l'asset ne peut pas être vide!" -ForegroundColor Red
                Read-Host "Appuyez sur une touche"
                continue
            }
            
            $assetType = Get-AssetTypeMenu
            
            if ($assetType) {
                Write-Host ""
                Write-Host "⏳ Génération en cours..." -ForegroundColor Cyan
                & "$scriptDir\Create-AssetTemplate.ps1" -AssetName $assetName -AssetType $assetType -OutputPath "$projectRoot\$assetName"
                Write-Host ""
                Read-Host "Appuyez sur une touche pour continuer"
            }
        }
        "2" {
            & "$scriptDir\List-AssetTypes.ps1"
            Read-Host "Appuyez sur une touche pour continuer"
        }
        "0" {
            Write-Host "Au revoir!" -ForegroundColor Yellow
            exit 0
        }
        default {
            Write-Host "Choix invalide. Appuyez sur une touche..." -ForegroundColor Red
            Read-Host
        }
    }
} while ($true)
