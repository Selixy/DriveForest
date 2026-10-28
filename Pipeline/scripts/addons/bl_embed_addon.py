"""
bl_embed_addon.py
Script intermediaire execute par Blender en mode background.
Recoit deux arguments via -- :
    argv[0] : chemin absolu vers l'addon .py a embarquer
    argv[1] : chemin absolu du .blend de destination

Usage depuis PowerShell :
    blender -b --python bl_embed_addon.py -- "C:/path/addon.py" "C:/path/asset.blend"
"""

import bpy
import sys
import os

# --- Recuperer les arguments passes apres "--" ---
argv = sys.argv
separator = argv.index("--") if "--" in argv else None

if separator is None or len(argv) < separator + 3:
    print("[!] Usage: blender -b --python bl_embed_addon.py -- <addon_path> <blend_path>")
    sys.exit(1)

addon_path = argv[separator + 1]
blend_path = argv[separator + 2]

print(f"[*] Addon source : {addon_path}")
print(f"[*] Destination  : {blend_path}")

# --- Validation ---
if not os.path.isfile(addon_path):
    print(f"[!] Addon introuvable : {addon_path}")
    sys.exit(1)

# --- Lire le source de l'addon ---
with open(addon_path, "r", encoding="utf-8") as f:
    addon_source = f.read()

# --- Nettoyer un eventuel ancien texte du meme nom ---
text_name = "driveforest_addon.py"
if text_name in bpy.data.texts:
    bpy.data.texts.remove(bpy.data.texts[text_name])

# --- Creer le text-block et l'activer comme module ---
text_block = bpy.data.texts.new(text_name)
text_block.write(addon_source)
text_block.use_module = True

print(f"[+] Text-block '{text_name}' embarque et marque use_module=True")

# --- Creer le dossier de destination si necessaire ---
dest_dir = os.path.dirname(blend_path)
if dest_dir and not os.path.exists(dest_dir):
    os.makedirs(dest_dir, exist_ok=True)

# --- Sauvegarder le .blend ---
blend_path_normalized = blend_path.replace("\\", "/")
bpy.ops.wm.save_as_mainfile(filepath=blend_path_normalized)

print(f"[+] Fichier .blend sauvegarde : {blend_path_normalized}")