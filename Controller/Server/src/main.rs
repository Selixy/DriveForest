mod network;
mod core;
mod ui;

use std::thread;
use network::input::listen;
use network::output::run_sender;

fn main() {
    // Thread OSC entrée : reçoit les quaternions du téléphone
    thread::spawn(|| { listen(); });

    // Thread OSC sortie : envoie le braquage vers Unity en local
    thread::spawn(|| { run_sender(); });

    // Interface graphique sur le thread principal
    ui::run();
}
