# Serveur OSC Gyroscope - ZIG SIM

## 🎯 À propos
Ce serveur Rust écoute sur le port UDP **5000** pour recevoir des messages OSC provenant de l'application iOS **ZIG SIM**. Il traite les données de gyroscope et d'accélération en temps réel.

## 🚀 Démarrage rapide

### Lancer le serveur
```powershell
cd Controller/Server
cargo run --release
```

L'output devrait afficher :
```
🎧 Serveur écoute sur UDP 0.0.0.0:5000 (OSC)
```

### Configuration de ZIG SIM (iOS)
1. Télécharger l'app **ZIG SIM** depuis l'App Store
2. Ouvrir l'app et aller dans **Settings/Preferences**
3. Configurer l'adresse cible :
   - **IP**: L'adresse IP de votre ordinateur (ex: 192.168.x.x)
   - **Port**: `5000`
   - **Format**: `OSC` 
4. Activer le gyroscope/accéléromètre
5. Lancer l'envoi des données

## 📡 Adresses OSC supportées

| Adresse | Type | Description |
|---------|------|-------------|
| `/gyro/x` | Float | Rotation autour axe X (rad/s) |
| `/gyro/y` | Float | Rotation autour axe Y (rad/s) |
| `/gyro/z` | Float | Rotation autour axe Z (rad/s) |
| `/accel/x` | Float | Accélération axe X (m/s²) |
| `/accel/y` | Float | Accélération axe Y (m/s²) |
| `/accel/z` | Float | Accélération axe Z (m/s²) |

## 🔍 Logs en détail

Pour voir plus de détails des messages reçus, lancer avec :
```powershell
$env:RUST_LOG="info"
cargo run --release
```

### Exemple de log
```
🔄 /gyro/x: 0.125
🔄 /gyro/y: -0.089
⬆️ /accel/x: 9.81
```

## 🛠️ Architecture
```
p:\DriveForest\
├── Controller\
│   └── Server\              # Serveur Rust OSC/UDP
│       ├── Cargo.toml       # Dépendances
│       ├── src\
│       │   └── main.rs      # Code serveur
│       ├── README_OSC.md    # Ce fichier
│       └── target\          # Build output
```

## 📦 Dépendances
- **rosc**: Parser/encodeur OSC
- **tokio**: Runtime async pour UDP
- **log/env_logger**: Logging

## 🔧 Améliorations futures
- [ ] Websockets pour envoyer les données au client web
- [ ] Stockage des données en base
- [ ] Filtres/calibration gyroscope
- [ ] Interface web de monitoring
- [ ] Enregistrement de sessions

## ⚠️ Dépannage

**Le serveur ne reçoit aucune données**
- Vérifier que ZIG SIM envoie sur le bon port (5000)
- Vérifier que le firewall Windows n'est pas bloquant
- Tester localement avec: `netcat -u -l 127.0.0.1 5000`

**Erreur "Adresse déjà utilisée"**
- Attendre quelques secondes et relancer
- Tuer les anciens processus: `Get-Process | ? {$_.Name -match "Server"} | Stop-Process`

