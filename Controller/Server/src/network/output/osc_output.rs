use std::net::UdpSocket;

use crate::core::{get_steering, get_accel};

pub const UNITY_IP:       &str = "127.0.0.1";
pub const UNITY_PORT:     u16  = 9000;
pub const OSC_ADDRESS:    &str = "/driveforest/steering";
pub const OSC_ACCEL:      &str = "/driveforest/accel";
pub const OSC_HONK:       &str = "/driveforest/honk";

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

/// Envoie braquage + accélération vers Unity.
pub fn send_steering_now(socket: &UdpSocket, target: &str) {
    socket.send_to(&encode_float(OSC_ADDRESS, get_steering()), target).ok();
    socket.send_to(&encode_float(OSC_ACCEL,   get_accel()),    target).ok();
}

/// Envoie un trigger de klaxon vers Unity.
pub fn send_honk(socket: &UdpSocket, target: &str) {
    let packet = encode_float(OSC_HONK, 1.0);
    socket.send_to(&packet, target).ok();
}
