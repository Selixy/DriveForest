mod network;
mod core;

use std::thread;
use std::time::Duration;
use network::input::listen;
use core::{set_zero, debug};

fn main() {
    println!("🚀 Serveur DriveForest - Quaternion Receiver");
    println!("============================================\n");
    
    // Démarrer le récepteur OSC (tourne indéfiniment dans un thread)
    let _input_thread = std::thread::spawn(|| {
        listen();
    });
    
    // Calibrer la position neutre
    set_zero();
    
    // Afficher la rotation "volant" à ~60 FPS (toutes les ~16ms)
    loop {
        debug();
        thread::sleep(Duration::from_millis(16));
    }
}

