mod network;

use network::start_input;

fn main() {
    println!("🚀 Démarrage du serveur DriveForest...\n");
    
    // Démarrer le récepteur OSC (input)
    start_input();
}