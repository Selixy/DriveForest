"""
Addon Substance Designer pour DriveForest
Ajoute un bouton Hello World dans la partie export
"""

import substance

def hello_world():
    """Fonction Hello World"""
    print("=" * 50)
    print("Hello World from DriveForest!")
    print("=" * 50)
    substance.context.log.info("Hello World from DriveForest!")

# Enregistrer la fonction comme commande accessible
substance.context.registerFunction(hello_world)
