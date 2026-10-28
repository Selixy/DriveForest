@echo off
REM =========================================================
REM  DriveForest - Gestionnaire d'Assets
REM  Outil unique pour creer, lister et gerer les assets
REM =========================================================

setlocal enabledelayedexpansion
cd /d "%~dp0"

:menu
cls
echo.
echo =========================================================
echo   DriveForest - Gestionnaire d'Assets
echo =========================================================
echo.
echo.
echo  Que voulez-vous faire?
echo  ---------------------------------------------------------
echo.
echo    1. Creer un nouvel asset
echo    2. Lister les assets existants
echo    3. Incrementer la version d'un asset
echo    4. Afficher les types disponibles
echo    5. Gerer le plugin Substance Painter
echo    6. Afficher l'aide
echo    0. Quitter
echo.

set /p choice="Votre choix (0-6): "

REM Valider le choix
if "%choice%"=="" goto :error_choice
if "%choice%"=="0" exit /b 0
if "%choice%"=="1" goto :create_asset
if "%choice%"=="2" goto :list_assets
if "%choice%"=="3" goto :increment_version
if "%choice%"=="4" goto :show_types
if "%choice%"=="5" goto :manage_painter
if "%choice%"=="6" goto :show_help

:error_choice
echo.
echo [!] Choix invalide!
timeout /t 2 /nobreak
goto :menu

:create_asset
cls
echo.
echo =========================================================
echo   CREER UN NOUVEL ASSET
echo =========================================================
echo.

REM Demander le nom de l'asset
set /p "AssetName=Entrez le nom de l'asset: "

if "!AssetName!"=="" (
    echo.
    echo [!] Le nom ne peut pas etre vide!
    timeout /t 2 /nobreak
    goto :menu
)

REM Vérifier si l'asset existe déjà
if exist "!AssetName!" (
    echo.
    echo [*] Un asset nomme '!AssetName!' existe deja.
    set /p "Confirm=Voulez-vous continuer? (O/N): "
    if /i not "!Confirm!"=="O" goto :menu
)

REM Afficher les types disponibles
echo.
echo Types d'assets disponibles:
echo.
echo    1. Character    - Personnages, PNJ, creatures
echo    2. Prop         - Objets, meubles
echo    3. Vehicle      - Vehicules, engins
echo.

set /p "TypeNum=Selectionnez le type (1-3): "

REM Mapper le numéro au type
if "!TypeNum!"=="1" set "AssetType=Character"
if "!TypeNum!"=="2" set "AssetType=Prop"
if "!TypeNum!"=="3" set "AssetType=Vehicle"

if not defined AssetType (
    echo.
    echo [!] Type invalide!
    timeout /t 2 /nobreak
    goto :menu
)

echo.
echo [*] Creation de l'asset '!AssetName!' (Type: !AssetType!)...
echo.

REM Appeler le script PowerShell
powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\assets\Create-AssetTemplate.ps1' -AssetName '!AssetName!' -AssetType '!AssetType!' -OutputPath '!CD!\!AssetName!'"

if !errorlevel! equ 0 (
    echo.
    echo [+] Asset cree avec succes!
) else (
    echo.
    echo [!] Erreur lors de la creation
)

echo.
pause
goto :menu

:list_assets
cls
echo.
echo =========================================================
echo   ASSETS EXISTANTS
echo =========================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\assets\List-Assets.ps1' -AssetPath '.' -Detailed"

if !errorlevel! neq 0 (
    echo [*] Aucun asset trouve
)

echo.
pause
goto :menu

:increment_version
cls
echo.
echo =========================================================
echo   INCREMENTER LA VERSION D'UN ASSET
echo =========================================================
echo.

set /p "AssetName=Entrez le nom de l'asset: "

if "!AssetName!"=="" (
    echo.
    echo [!] Le nom ne peut pas etre vide!
    timeout /t 2 /nobreak
    goto :menu
)

if not exist "!AssetName!" (
    echo.
    echo [!] L'asset '!AssetName!' n'existe pas!
    timeout /t 2 /nobreak
    goto :menu
)

echo.
echo [*] Incrementation en cours...
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\assets\Increment-AssetVersion.ps1' -AssetName '!AssetName!' -AssetPath '.' "

echo.
pause
goto :menu

:show_types
cls
echo.
echo =========================================================
echo   TYPES D'ASSETS DISPONIBLES
echo =========================================================
echo.

powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\config\List-AssetTypes.ps1'"

echo.
pause
goto :menu

:manage_painter
cls
echo.
echo =========================================================
echo   GESTION DU PLUGIN SUBSTANCE PAINTER
echo =========================================================
echo.
echo   1. Installer le plugin
echo   2. Mettre a jour le plugin
echo   3. Desinstaller le plugin
echo   0. Retour au menu principal
echo.

set /p "PainterChoice=Votre choix (0-3): "

if "%PainterChoice%"=="1" (
    echo.
    powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\dcc\substance\Install-PainterPlugin.ps1'"
) else if "%PainterChoice%"=="2" (
    echo.
    powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\dcc\substance\Update-PainterPlugin.ps1'"
) else if "%PainterChoice%"=="3" (
    echo.
    powershell -NoProfile -ExecutionPolicy Bypass -Command "& '.\scripts\core\dcc\substance\Uninstall-PainterPlugin.ps1'"
) else if "%PainterChoice%"=="0" (
    goto :menu
) else (
    echo.
    echo [!] Choix invalide!
)

echo.
pause
goto :manage_painter

:show_help
cls
echo.
echo =========================================================
echo   AIDE ET DOCUMENTATION
echo =========================================================
echo.
echo  STRUCTURE D'UN ASSET:
echo  ---------------------------------------------------------
echo.
echo    MonAsset/
echo    +-- DCC/
echo    !   +-- MonAsset_Blender_V00.blend
echo    !   +-- MonAsset_Substance_V00.sbs
echo    +-- Output/
echo    +-- Textures/
echo    +-- README.md
echo    +-- .asset-meta.json
echo.
echo.
echo  NOMENCLATURE:
echo  ---------------------------------------------------------
echo.
echo    Format:  {AssetName}_{DCC}_V{Version}.{ext}
echo.
echo    Exemples:
echo    - MonAsset_Blender_V00.blend
echo    - MonAsset_Substance_V00.sbs
echo    - MonAsset_Blender_V01.blend  (apres mise a jour)
echo.
echo.
echo  WORKFLOW RAPIDE:
echo  ---------------------------------------------------------
echo.
echo    1. Lancer Asset.bat
echo    2. Choisir "1. Creer un nouvel asset"
echo    3. Entrer le nom de l'asset
echo    4. Selectionner le type
echo    5. Structure creee automatiquement
echo    6. Ouvrir les fichiers dans DCC/
echo    7. Exporter dans Output/
echo    8. Incrementer la version quand termine
echo.
echo.
echo  TYPES D'ASSETS:
echo  ---------------------------------------------------------
echo.
echo    * Character    - Personnages, PNJ, creatures
echo    * Environment  - Decors, batiments, terrains
echo    * Prop         - Objets interactifs, meubles
echo    * Vehicle      - Vehicules, velos, engins
echo    * Material     - Materiaux, shaders, textures
echo.
echo.
pause
goto :menu
