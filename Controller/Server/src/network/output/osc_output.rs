use std::net::UdpSocket;
use std::thread;
use std::time::Duration;

use crate::core::get_steering;

pub const UNITY_IP:    &str = "127.0.0.1";
pub const UNITY_PORT:  u16  = 9000;
pub const OSC_ADDRESS: &str = "/driveforest/steering";

/// Encode un message OSC minimal : une adresse + un float.
///
/// Format OSC :
///   - Adresse : chaîne null-terminée, paddée au multiple de 4 octets
///   - Type tag : ",f" null-terminé, paddé au multiple de 4 octets
///   - Valeur : f32 big-endian (4 octets)
fn encode_float(address: &str, value: f32) -> Vec<u8> {
    let mut buf = Vec::with_capacity(32);

    // Adresse
    buf.extend_from_slice(address.as_bytes());
    buf.push(0); // null-terminator
    while buf.len() % 4 != 0 { buf.push(0); }

    // Type tag ",f"
    buf.extend_from_slice(b",f\0\0");

    // Valeur float big-endian
    buf.extend_from_slice(&value.to_be_bytes());

    buf
}

/// Boucle d'envoi : lit le braquage calculé et l'envoie en OSC UDP ~60 fps.
pub fn run_sender() {
    let socket = UdpSocket::bind("0.0.0.0:0")
        .expect("OSC output: impossible de créer le socket");

    let target = format!("{}:{}", UNITY_IP, UNITY_PORT);

    println!("OSC output -> udp://{}  addr: {}", target, OSC_ADDRESS);

    loop {
        let steering = get_steering();
        let packet   = encode_float(OSC_ADDRESS, steering);
        socket.send_to(&packet, &target).ok();
        thread::sleep(Duration::from_millis(16)); // ~60 fps
    }
}
