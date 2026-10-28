<#
.SYNOPSIS
    Cree un fichier .blend avec l'addon DriveForest embarque dedans.
#>

param(
    [Parameter(Mandatory=$true)]
    [string]$DCCPath,

    [Parameter(Mandatory=$true)]
    [string]$FileName
)

# --- Fonction : trouver Blender automatiquement ---
function Find-Blender {
    # 1. Dans le PATH
    $fromPath = Get-Command "blender" -ErrorAction SilentlyContinue | Select-Object -ExpandProperty Source
    if ($fromPath) { return $fromPath }

    # 2. Emplacements d'installation standards Windows
    $searchRoots = @(
        "$env:ProgramFiles\Blender Foundation",
        "${env:ProgramFiles(x86)}\Blender Foundation",
        "$env:LOCALAPPDATA\Programs\Blender Foundation",
        "C:\Blender",
        "D:\Blender",
        "D:\Program Files\Blender Foundation"
    )

    foreach ($root in $searchRoots) {
        if (Test-Path $root) {
            $found = Get-ChildItem -Path $root -Recurse -Filter "blender.exe" -ErrorAction SilentlyContinue |
                     Sort-Object FullName -Descending |
                     Select-Object -First 1
            if ($found) { return $found.FullName }
        }
    }

    # 3. Registre Windows (installation via installeur officiel)
    $regPaths = @(
        "HKLM:\SOFTWARE\BlenderFoundation",
        "HKCU:\SOFTWARE\BlenderFoundation"
    )
    foreach ($reg in $regPaths) {
        if (Test-Path $reg) {
            $installDir = (Get-ItemProperty $reg -ErrorAction SilentlyContinue).Install_Dir
            if ($installDir) {
                $exe = Join-Path $installDir "blender.exe"
                if (Test-Path $exe) { return $exe }
            }
        }
    }

    # 4. Steam
    $steamBlender = "$env:ProgramFiles\Steam\steamapps\common\Blender\blender.exe"
    if (Test-Path $steamBlender) { return $steamBlender }

    return $null
}

# --- Chemins ---
$blendFile = Join-Path $DCCPath "$FileName.blend"
$addonsDir = (Resolve-Path (Join-Path $PSScriptRoot "..\..\..\addons")).Path -replace '\\', '/'

# --- Creer le dossier de destination si necessaire ---
if (-not (Test-Path $DCCPath)) {
    New-Item -Path $DCCPath -ItemType Directory -Force | Out-Null
    Write-Host "[+] Dossier cree : $DCCPath"
}

# --- Trouver Blender ---
Write-Host "[*] Recherche de Blender..."
$blenderExe = Find-Blender

if (-not $blenderExe) {
    Write-Warning "[!] Blender introuvable sur cette machine."
    Write-Warning "    Installez Blender depuis https://www.blender.org/download/"
    Write-Warning "    Fichier .blend non genere."
    exit 1
}

Write-Host "[+] Blender trouve : $blenderExe"

$embedScript = @"
import bpy
import sys

# Ajouter le chemin de l'addon a sys.path
addons_dir = r'$addonsDir'
if addons_dir not in sys.path:
    sys.path.append(addons_dir)

# Creer un nouveau text block pour le loader
script = bpy.data.texts.new('df_loader.py')
script.write(f'''
import sys
if r"{addons_dir}" not in sys.path:
    sys.path.append(r"{addons_dir}")
import blender_addon_driveforest
'''.strip())

# Sauvegarder
bpy.ops.wm.save_as_mainfile(filepath=r'$blendFile')
"@.Replace("`r`n", "`n")

# --- Lancer Blender en background ---
Write-Host "[*] Creation de $FileName.blend..."

$blenderArgs = @(
    "-b"
    "--python-expr", $embedScript
)

try {
    $process = Start-Process `
        -FilePath $blenderExe `
        -ArgumentList $blenderArgs `
        -NoNewWindow `
        -Wait `
        -PassThru `
        -ErrorAction Stop

    if ($process.ExitCode -eq 0 -and (Test-Path $blendFile)) {
        Write-Host "[+] Succes : $blendFile"
    } else {
        Write-Warning "[!] Blender a termine avec le code $($process.ExitCode)."
        exit 1
    }

} catch {
    Write-Warning "[!] Erreur lors du lancement de Blender : $_"
    exit 1
}