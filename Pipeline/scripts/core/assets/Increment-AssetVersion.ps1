param(
    [Parameter(Mandatory=$true)]
    [string]$AssetName,
    
    [string]$AssetPath = "$PSScriptRoot\..",
    [int]$NewVersion
)

# Incrementer la version d'un asset

$assetRoot = "$AssetPath\$AssetName"

if (-not (Test-Path $assetRoot)) {
    Write-Host "[!] L'asset '$AssetName' n'existe pas"
    exit 1
}

$metaPath = "$assetRoot\.asset-meta.json"

if (-not (Test-Path $metaPath)) {
    Write-Host "[!] Fichier metadonnees introuvable pour '$AssetName'"
    exit 1
}

# Lire les metadonnees actuelles
$meta = Get-Content $metaPath -Encoding UTF8 | ConvertFrom-Json
$currentVersion = [int]$meta.version

# Determiner la nouvelle version
if ($NewVersion) {
    $nextVersion = $NewVersion
} else {
    $nextVersion = $currentVersion + 1
}

$versionPad = "{0:D2}" -f $nextVersion
$currentVersionPad = "{0:D2}" -f $currentVersion

Write-Host "[*] Mise a jour de la version de l'asset '$AssetName'"
Write-Host "    V$currentVersionPad -> V$versionPad"
Write-Host ""

# Copier les fichiers DCC vers les versions precedentes
$dccPath = "$assetRoot\DCC"
$oldVersionPath = "$assetRoot\DCC\Versions"

if (-not (Test-Path $oldVersionPath)) {
    New-Item -Path $oldVersionPath -ItemType Directory -Force | Out-Null
}

$oldVersionDir = "$oldVersionPath\V$currentVersionPad"
New-Item -Path $oldVersionDir -ItemType Directory -Force | Out-Null

# Copier les fichiers actuels
$dccFiles = Get-ChildItem $dccPath -File | Where-Object { $_.Extension -in @(".blend", ".sbs") }

foreach ($file in $dccFiles) {
    Copy-Item -Path $file.FullName -Destination $oldVersionDir -Force
    Write-Host "[+] Archive: $($file.Name)"
}

# Creer les nouveaux fichiers versions
Write-Host ""
Write-Host "[*] Creation des nouveaux fichiers versions..."

$assetNameClean = $assetRoot | Split-Path -Leaf

foreach ($file in $dccFiles) {
    $extension = $file.Extension
    $fileName = $file.BaseName
    
    if ($fileName -match "(.+)_Blender_V\d+") {
        $dccName = "Blender"
        $newName = "$assetNameClean`_Blender_V$versionPad$extension"
    } elseif ($fileName -match "(.+)_Substance_V\d+") {
        $dccName = "Substance"
        $newName = "$assetNameClean`_Substance_V$versionPad$extension"
    } else {
        continue
    }
    
    $newFilePath = "$dccPath\$newName"
    New-Item -Path $newFilePath -ItemType File -Force | Out-Null
    Write-Host "[+] Cree: $newName"
}

# Mettre a jour les metadonnees
$meta.version = $versionPad
$meta.modified = (Get-Date -Format "yyyy-MM-dd HH:mm:ss")
$metaJson = $meta | ConvertTo-Json
Set-Content -Path $metaPath -Value $metaJson -Encoding UTF8

Write-Host ""
Write-Host "[OK] Version incrementee avec succes!"
Write-Host "[*] Anciennes versions archivees dans: $oldVersionDir"

