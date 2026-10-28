param(
    [string]$AssetPath = "$PSScriptRoot\..",
    [switch]$Detailed
)

# Lister les assets existants

$assetFolders = Get-ChildItem -Path $AssetPath -Directory | 
    Where-Object { 
        (Test-Path "$($_.FullName)\.asset-meta.json") -or
        (Test-Path "$($_.FullName)\DCC") -or
        (Test-Path "$($_.FullName)\Output") -or
        (Test-Path "$($_.FullName)\Textures")
    }

if ($assetFolders.Count -eq 0) {
    Write-Host "[*] Aucun asset trouve dans $AssetPath"
    exit 0
}

Write-Host ""
Write-Host "Assets disponibles:"
Write-Host ""

foreach ($asset in $assetFolders) {
    Write-Host "  * $($asset.Name)"
    
    if ($Detailed) {
        $metaPath = "$($asset.FullName)\.asset-meta.json"
        
        if (Test-Path $metaPath) {
            $meta = Get-Content $metaPath -Encoding UTF8 | ConvertFrom-Json
            Write-Host "    Type: $($meta.type)"
            Write-Host "    Version: $($meta.version)"
            Write-Host "    Cree par: $($meta.creator)"
            Write-Host "    Cree le: $($meta.created)"
        }
        
        # Afficher la structure
        $dccFiles = Get-ChildItem "$($asset.FullName)\DCC" -File -ErrorAction SilentlyContinue
        if ($dccFiles) {
            Write-Host "    Fichiers DCC:"
            $dccFiles | ForEach-Object {
                Write-Host "      - $($_.Name)"
            }
        }
        
        Write-Host ""
    }
}

Write-Host "[*] Utilisez l'option -Detailed pour voir plus d'informations"

