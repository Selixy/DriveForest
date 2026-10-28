<#
.SYNOPSIS
    Met a jour le plugin DriveForest dans Substance 3D Painter.
    Remplace l'ancienne version par la version actuelle du depot.
#>

function Find-PainterPluginsDir {
    $basePath = "$env:LOCALAPPDATA\Adobe\Adobe Substance 3D Painter"
    if (-not (Test-Path $basePath)) { return $null }

    $pluginsDir = Get-ChildItem -Path $basePath -Recurse -Filter "plugins" -Directory -ErrorAction SilentlyContinue |
                  Select-Object -First 1

    if ($pluginsDir) { return $pluginsDir.FullName }
    return $null
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

$pluginDest = Join-Path $pluginsDir "driveforest_addon.py"

if (-not (Test-Path $pluginDest)) {
    Write-Warning "[!] Plugin non installe. Utilisez Install-PainterPlugin.ps1 d'abord."
    exit 1
}

# Lire les versions pour afficher un diff
$oldContent = Get-Content $pluginDest -Raw
$newContent  = Get-Content $pluginSource -Raw

if ($oldContent -eq $newContent) {
    Write-Host "[i] Le plugin est deja a jour, aucune action requise."
    exit 0
}

# Backup de l'ancienne version
$backupPath = "$pluginDest.bak"
Copy-Item -Path $pluginDest -Destination $backupPath -Force
Write-Host "[i] Ancienne version sauvegardee : $backupPath"

# Remplacer
Copy-Item -Path $pluginSource -Destination $pluginDest -Force
Write-Host "[+] Plugin mis a jour avec succes : $pluginDest"
Write-Host "[i] Redemarrez Substance Painter pour appliquer la mise a jour."
