<#
.SYNOPSIS
    Installe le plugin DriveForest dans Substance 3D Painter.
    A utiliser si le plugin n'a pas ete installe automatiquement.
#>

function Find-PainterPluginsDir {
    $basePath = "$env:LOCALAPPDATA\Adobe\Adobe Substance 3D Painter"
    if (-not (Test-Path $basePath)) { return $null }

    $pluginsDir = Get-ChildItem -Path $basePath -Recurse -Filter "plugins" -Directory -ErrorAction SilentlyContinue |
                  Select-Object -First 1

    if ($pluginsDir) { return $pluginsDir.FullName }

    $fallback = "$basePath\plugins"
    New-Item -Path $fallback -ItemType Directory -Force | Out-Null
    return $fallback
}

$pluginSource = Join-Path $PSScriptRoot "..\..\..\addons\substance_addon_driveforest.py"

if (-not (Test-Path $pluginSource)) {
    Write-Error "[!] Plugin source introuvable : $pluginSource"
    exit 1
}

Write-Host "[*] Recherche du dossier plugins Substance Painter..."
$pluginsDir = Find-PainterPluginsDir

if (-not $pluginsDir) {
    Write-Error "[!] Substance 3D Painter introuvable. Verifiez votre installation Creative Cloud."
    exit 1
}

Write-Host "[+] Dossier plugins : $pluginsDir"

$pluginDest = Join-Path $pluginsDir "driveforest_addon.py"

if (Test-Path $pluginDest) {
    Write-Host "[i] Plugin deja installe. Utilisez Update-PainterPlugin.ps1 pour le mettre a jour."
    exit 0
}

Copy-Item -Path $pluginSource -Destination $pluginDest -Force
Write-Host "[+] Plugin installe avec succes : $pluginDest"
Write-Host "[i] Redemarrez Substance Painter pour activer le plugin."
