<#
.SYNOPSIS
    Cree un fichier .spp vierge et installe le plugin DriveForest si necessaire.
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$DCCPath,

    [Parameter(Mandatory=$true)]
    [string]$FileName
)

# --- Fonction : trouver le dossier plugins de Painter ---
function Find-PainterPluginsDir {
    $basePath = "$env:LOCALAPPDATA\Adobe\Adobe Substance 3D Painter"
    if (-not (Test-Path $basePath)) { return $null }

    # Chercher le dossier plugins dans les versions installees
    $pluginsDir = Get-ChildItem -Path $basePath -Recurse -Filter "plugins" -Directory -ErrorAction SilentlyContinue |
                  Select-Object -First 1

    if ($pluginsDir) { return $pluginsDir.FullName }

    # Fallback : creer le dossier plugins dans le chemin standard
    $fallback = "$basePath\plugins"
    New-Item -Path $fallback -ItemType Directory -Force | Out-Null
    return $fallback
}

# --- Chemins ---
$sppFile      = Join-Path $DCCPath "$FileName.spp"
$pluginSource = Join-Path $PSScriptRoot "..\..\..\addons\substance_addon_driveforest.py"

# --- Validation ---
if (-not (Test-Path $pluginSource)) {
    Write-Warning "[!] Plugin introuvable : $pluginSource"
    Write-Warning "    Le fichier .spp sera cree sans plugin."
}

# --- Creer le dossier de destination ---
if (-not (Test-Path $DCCPath)) {
    New-Item -Path $DCCPath -ItemType Directory -Force | Out-Null
}

# --- Creer le .spp vierge (format zip minimal valide pour Painter) ---
Write-Host "[*] Creation de $FileName.spp..."
Add-Type -AssemblyName System.IO.Compression.FileSystem

$tempDir = "$env:TEMP\spp_$([guid]::NewGuid())"
New-Item -Path $tempDir -ItemType Directory -Force | Out-Null

# Metadata minimale reconnue par Painter
$metadata = @{
    version = "10.0.0"
    name    = $FileName
} | ConvertTo-Json
Set-Content -Path "$tempDir\metadata.json" -Value $metadata -Encoding UTF8

[System.IO.Compression.ZipFile]::CreateFromDirectory($tempDir, $sppFile)
Remove-Item -Path $tempDir -Recurse -Force

Write-Host "[+] Fichier .spp cree : $sppFile"

# --- Installer le plugin si pas deja present ---
if (Test-Path $pluginSource) {
    $pluginsDir = Find-PainterPluginsDir
    if ($pluginsDir) {
        $pluginDest = Join-Path $pluginsDir "driveforest_addon.py"
        if (-not (Test-Path $pluginDest)) {
            Copy-Item -Path $pluginSource -Destination $pluginDest -Force
            Write-Host "[+] Plugin installe : $pluginDest"
            Write-Host "[i] Redemarrez Substance Painter pour activer le plugin."
        } else {
            Write-Host "[i] Plugin deja installe, aucune action requise."
        }
    } else {
        Write-Warning "[!] Dossier plugins Painter introuvable. Painter est-il installe via Creative Cloud ?"
    }
}