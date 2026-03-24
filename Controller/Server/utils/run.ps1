# Run script pour le serveur Rust
# Lance l'executable depuis le dossier dist (ou compile et lance)

param(
    [switch]$Build,
    [switch]$Release
)

$ErrorActionPreference = "Stop"

# Si -Build est specifie, compiler d'abord
if ($Build) {
    $config = if ($Release) { "release" } else { "debug" }
    Write-Host "[BUILD] Compilation en cours..." -ForegroundColor Cyan
    & "$PSScriptRoot\build.ps1" -Configuration $config
    if ($LASTEXITCODE -ne 0) {
        exit 1
    }
}

$exeName = "Server.exe"
$distFile = "dist\$exeName"

# Verifier que l'executable existe
if (-not (Test-Path $distFile)) {
    Write-Host "[ERROR] Executable non trouve: $distFile" -ForegroundColor Red
    Write-Host "[INFO] Lancez avec -Build pour compiler" -ForegroundColor Yellow
    exit 1
}

Write-Host "[RUN] Lancement de $distFile..." -ForegroundColor Cyan
& $distFile
