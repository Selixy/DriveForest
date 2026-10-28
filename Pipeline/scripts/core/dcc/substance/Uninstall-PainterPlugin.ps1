<#
.SYNOPSIS
    Desinstalle proprement le plugin DriveForest de Substance 3D Painter.
#>

function Find-PainterPluginsDir {
    $basePath = "$env:LOCALAPPDATA\Adobe\Adobe Substance 3D Painter"
    if (-not (Test-Path $basePath)) { return $null }

    $pluginsDir = Get-ChildItem -Path $basePath -Recurse -Filter "plugins" -Directory -ErrorAction SilentlyContinue |
                  Select-Object -First 1

    if ($pluginsDir) { return $pluginsDir.FullName }
    return $null
}

Write-Host "[*] Recherche du plugin DriveForest..."
$pluginsDir = Find-PainterPluginsDir

if (-not $pluginsDir) {
    Write-Host "[i] Substance 3D Painter introuvable, rien a nettoyer."
    exit 0
}

$pluginDest = Join-Path $pluginsDir "driveforest_addon.py"
$backupDest = Join-Path $pluginsDir "driveforest_addon.py.bak"

$removed = $false

if (Test-Path $pluginDest) {
    Remove-Item -Path $pluginDest -Force
    Write-Host "[+] Plugin supprime : $pluginDest"
    $removed = $true
}

if (Test-Path $backupDest) {
    Remove-Item -Path $backupDest -Force
    Write-Host "[+] Backup supprime : $backupDest"
    $removed = $true
}

if (-not $removed) {
    Write-Host "[i] Aucun plugin DriveForest trouve, rien a faire."
} else {
    Write-Host "[+] Desinstallation complete."
    Write-Host "[i] Redemarrez Substance Painter pour appliquer."
}