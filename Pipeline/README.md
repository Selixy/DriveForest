# Pipeline d'assets

Des scripts PowerShell pour ranger les assets du jeu et créer leurs fichiers de travail, pour que l'artiste n'ait pas à monter les dossiers à la main. Tout se lance depuis un menu console : `Asset.bat`.

## Ce que ça fait

1. **Créer un asset** : un dossier nommé d'après l'asset, de type Character, Prop ou Vehicle, avec sa structure `V00/Lod0/DCC` (fichiers de travail) et `V00/Lod0/Output` (exports), un fichier de métadonnées `.asset-meta.json` et un README de départ.
2. **Lister les assets** existants.
3. **Incrémenter la version** d'un asset : les fichiers de travail de la version courante sont archivés dans `DCC/Versions/Vxx`, puis les fichiers de la version suivante sont créés.
4. **Créer le fichier Blender** de l'asset. Le fichier `.blend` est fabriqué avec l'addon embarqué directement dedans : Blender est lancé en arrière-plan avec `bl_embed_addon.py`, qui y enregistre l'addon.
5. **Créer le fichier Substance Painter** de l'asset, et installer, mettre à jour ou retirer le plugin dans le dossier des plugins de Painter.

## Les addons

Les deux addons (`scripts/addons/`) sont une base : un panneau « DriveForest » dans la barre latérale de Blender (touche N) avec un bouton d'essai, et l'équivalent pour Substance Painter. Je voulais y brancher la création et le versionnage des assets pour qu'on n'ouvre plus de console, mais je n'ai pas été plus loin.

Dans Blender, l'addon peut vivre dans le fichier `.blend`, donc il suit l'asset. Substance Painter ne permet pas de glisser l'addon dans le fichier de projet : il faut passer par le dossier de plugins du logiciel, d'où les scripts d'installation et de mise à jour. C'est la raison pour laquelle l'intégration s'est arrêtée à la création du fichier et du dossier.

## Utilisation

Windows, PowerShell. Blender doit être installé (il est cherché dans le PATH et dans les dossiers d'installation habituels). Lancer `Asset.bat` et suivre le menu.

## Rangement

```
Pipeline/
  Asset.bat                  menu console
  scripts/core/              les scripts PowerShell (assets, config, dcc, utils)
  scripts/addons/            les addons Blender et Substance Painter
  exemple-asset/             un exemple de ce que le gabarit produit
```

Les scripts de plugin Painter cherchent les addons dans `scripts/addons/`, à côté de `scripts/core/`.
