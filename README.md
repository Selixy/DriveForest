# DriveForest

Simulateur de conduite en VR où un téléphone sert de volant. Projet d'école de 2026, fait à deux : Alice Bert a modélisé l'environnement, j'ai pris le code et les outils. Le détail est sur la [fiche du portfolio](https://selixy.dev).

## Comment ça marche

Le téléphone fait tourner ZIG SIM, qui envoie son orientation et son accéléromètre en OSC sur UDP. Un serveur Rust les reçoit, transforme le quaternion en braquage utilisable quelle que soit la façon dont on tient le téléphone, et renvoie le braquage, l'accélération et le klaxon en OSC au jeu Unity.

## Ce que contient ce dépôt

- `Controller/Server/` : le serveur Rust (`nalgebra`, `eframe`). Il écoute sur le port 50000, envoie à Unity sur `127.0.0.1:9000`, et propose une interface de calibration. Les scripts `utils/build.ps1` et `utils/run.ps1` le compilent et le lancent.
- `Unity_DriveSim/` : les réglages du projet Unity (Unity 6000.4, OpenXR).

Les scripts C# du jeu, le pipeline d'assets en PowerShell et les addons Blender et Substance Painter ne sont pas encore rangés dans ce dépôt.

## État

Prototype. La boucle jouable fonctionne, le reste n'est pas finalisé. Le build Unity a été retiré du dépôt (il reste dans l'historique).
