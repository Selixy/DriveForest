param(
    [Parameter(Mandatory=$true)]
    [string]$AssetName,
    
    [Parameter(Mandatory=$true)]
    [ValidateSet("Character", "Prop", "Vehicle")]
    [string]$AssetType,
    
    [string]$OutputPath = "$PSScriptRoot\..\..\$AssetName"
)

# Verifier si le dossier existe deja
if (Test-Path $OutputPath) {
    Write-Host "[*] Un dossier nomme '$AssetName' existe deja a l'emplacement: $OutputPath"
    $response = Read-Host "Voulez-vous continuer? (O/N)"
    if ($response -ne "O") {
        Write-Host "Creation annulee."
        exit 1
    }
}

# Creer la structure de base
Write-Host "[*] Creation de la structure pour l'asset '$AssetName' (Type: $AssetType)..."

# Creer les dossiers principaux (Structure V00/Lod0)
$dccPath = "$OutputPath\V00\Lod0\DCC"
$outPath = "$OutputPath\V00\Lod0\Output"

$mainFolders = @(
    $OutputPath,
    "$OutputPath\V00",
    "$OutputPath\V00\Lod0",
    $dccPath,
    $outPath
)

foreach ($folder in $mainFolders) {
    if (-not (Test-Path $folder)) {
        New-Item -Path $folder -ItemType Directory -Force | Out-Null
        $folderName = Split-Path $folder -Leaf
        Write-Host "[+] Dossier cree: $folderName"
    }
}

# Creer l'asset_meta.json
$metaData = @{
    is_static = ($AssetType -eq "Prop")
    rigged = ($AssetType -in @("Character", "Vehicle"))
    lods = @("Lod0")
    version = "V00"
}
$metaData | ConvertTo-Json | Set-Content -Path "$OutputPath\${AssetName}_meta.json" -Encoding UTF8
Write-Host "[+] Fichier ${AssetName}_meta.json cree"

# Creer les fichiers DCC (Blender et Substance)
$blenderFile = "${AssetName}_Blender_V00"
& "$PSScriptRoot\..\dcc\blender\Create-BlenderFile.ps1" -DCCPath $dccPath -FileName $blenderFile

$substanceFile = "${AssetName}_Substance_V00"
& "$PSScriptRoot\..\dcc\substance\Create-SubstanceFile.ps1" -DCCPath $dccPath -FileName $substanceFile

# Creer un fichier README
$readmePath = "$OutputPath\README.md"
$dateNow = Get-Date -Format "dd/MM/yyyy HH:mm"
$readmeContent = @"
# Asset: `$AssetName

Type: `$AssetType
Cree le: `$dateNow

## Structure

- V00/Lod0/DCC/    - Fichiers de travail (Blender, Substance)
- V00/Lod0/Output/ - Fichiers exportes finalises

"@

Set-Content -Path $readmePath -Value $readmeContent -Encoding UTF8
Write-Host "[+] Fichier README cree"

Write-Host ""
Write-Host "[OK] Template d'asset cree avec succes!"
Write-Host "[*] Chemin: $OutputPath"
Write-Host ""
Write-Host "Contenu genere:"
Get-ChildItem -Path $OutputPath -Recurse | ForEach-Object {
    $relativePath = $_.FullName -replace [regex]::Escape($OutputPath), "."
    Write-Host "  $relativePath"
}
