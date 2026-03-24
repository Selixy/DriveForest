# Build script pour le serveur Rust
# Compile le projet et copie l'executable dans dist/

param(
    [ValidateSet("debug", "release")]
    [string]$Configuration = "debug"
)

$ErrorActionPreference = "Stop"

Write-Host "[BUILD] Compilation du serveur (Configuration: $Configuration)..." -ForegroundColor Cyan

# Build avec Cargo
if ($Configuration -eq "release") {
    cargo build --release
} else {
    cargo build
}

if ($LASTEXITCODE -ne 0) {
    Write-Host "[ERROR] La compilation a echoue!" -ForegroundColor Red
    exit 1
}

# Determiner le nom et le chemin de l'executable
$exeName = "Server.exe"
$targetPath = "target\$Configuration\$exeName"
$distPath = "dist"
$distFile = "$distPath\$exeName"

# Creer le dossier dist s'il n'existe pas
if (-not (Test-Path $distPath)) {
    New-Item -ItemType Directory -Path $distPath | Out-Null
    Write-Host "[INFO] Dossier dist cree" -ForegroundColor Gray
}

# Copier l'executable
if (Test-Path $targetPath) {
    Copy-Item -Path $targetPath -Destination $distFile -Force
    Write-Host "[SUCCESS] Executable copie vers $distFile" -ForegroundColor Green
} else {
    Write-Host "[ERROR] Executable non trouve: $targetPath" -ForegroundColor Red
    exit 1
}
