mod network;
mod core;
mod ui;

use std::thread;
use network::input::listen;

fn main() {
    // Thread OSC : reçoit les quaternions et envoie le braquage immédiatement
    thread::spawn(|| { listen(); });

    // Interface graphique sur le thread principal
    ui::run();
}
