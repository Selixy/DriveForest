use std::net::UdpSocket;
use std::sync::{Mutex, OnceLock};

// Static pour stocker les données du gyroscope de manière thread-safe
static GYROSCOPE_DATA: OnceLock<Mutex<(f32, f32, f32)>> = OnceLock::new();

fn get_gyroscope_mut() -> &'static Mutex<(f32, f32, f32)> {
    GYROSCOPE_DATA.get_or_init(|| Mutex::new((0.0, 0.0, 0.0)))
}

/// Retourne les 3 dernières valeurs du gyroscope (x, y, z) en rad/s
pub fn get_gyroscope() -> (f32, f32, f32) {
    let data = get_gyroscope_mut();
    if let Ok(gyro) = data.lock() {
        *gyro
    } else {
        (0.0, 0.0, 0.0)
    }
}

#[derive(Debug, Clone)]
pub enum OscArg {
    Float(f32),
    Int(i32),
    Str(String),
    Unknown,
}

impl std::fmt::Display for OscArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            OscArg::Float(v) => write!(f, "{:.6}", v),
            OscArg::Int(v) => write!(f, "{}", v),
            OscArg::Str(v) => write!(f, "\"{}\"", v),
            OscArg::Unknown => write!(f, "?"),
        }
    }
}

fn parse_osc(data: &[u8]) -> Option<(String, Vec<OscArg>)> {
    // Vérifier si c'est un bundle ou un message simple
    if data.starts_with(b"#bundle\0") {
        return None; // Les bundles sont traités séparément
    }

    let addr_end = data.iter().position(|&b| b == 0)?;
    let address = String::from_utf8(data[..addr_end].to_vec()).ok()?;

    let type_tag_start = (addr_end + 4) & !3;
    if type_tag_start >= data.len() || data[type_tag_start] != b',' {
        return Some((address, vec![]));
    }

    let tag_end = data[type_tag_start..]
        .iter()
        .position(|&b| b == 0)
        .map(|p| type_tag_start + p)?;

    let type_tags = &data[type_tag_start + 1..tag_end];
    let mut offset = (tag_end + 4) & !3;
    let mut args = Vec::new();

    for &tag in type_tags {
        match tag {
            b'f' if offset + 4 <= data.len() => {
                let val = f32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
                args.push(OscArg::Float(val));
                offset += 4;
            }
            b'i' if offset + 4 <= data.len() => {
                let val = i32::from_be_bytes(data[offset..offset + 4].try_into().unwrap());
                args.push(OscArg::Int(val));
                offset += 4;
            }
            b's' => {
                if let Some(end) = data[offset..].iter().position(|&b| b == 0) {
                    let s = String::from_utf8_lossy(&data[offset..offset + end]).to_string();
                    args.push(OscArg::Str(s));
                    offset = (offset + end + 4) & !3;
                }
            }
            _ => args.push(OscArg::Unknown),
        }
    }

    Some((address, args))
}

fn parse_osc_bundle(data: &[u8]) {
    // OSC Bundle format:
    // - "#bundle\0" (8 bytes)
    // - Timetag (8 bytes)
    // - Ensuite: répétition de (size + data)
    
    if !data.starts_with(b"#bundle\0") || data.len() < 16 {
        return;
    }

    let mut offset = 16; // Sauter "#bundle\0" + timetag

    while offset + 4 <= data.len() {
        // Lire la taille du prochain élément (big-endian)
        let size = i32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + size > data.len() {
            break;
        }

        let element = &data[offset..offset + size];
        offset += size;

        // Parser l'élément comme un message OSC
        if let Some((address, args)) = parse_osc(element) {
            handle_gyroscope_data(&address, &args);
        }
    }
}

pub fn listen() {
    let socket = UdpSocket::bind("0.0.0.0:50000")
        .expect("Impossible de binder le socket sur le port 50000");
    
    println!("Récepteur Gyroscope OSC démarré");
    println!("Écoute sur UDP 0.0.0.0:50000");
    println!("Fréquence: ~60 FPS\n");

    let mut buf = [0u8; 4096];

    loop {
        let (len, _addr) = socket.recv_from(&mut buf)
            .expect("Erreur de réception UDP");
        
        let data = &buf[..len];

        if data.starts_with(b"#bundle\0") {
            parse_osc_bundle(data);
        }
    }
}

fn handle_gyroscope_data(address: &str, args: &[OscArg]) {
    if address.ends_with("/gyro") && args.len() >= 3 {
        // Format: /ZIGSIM/device_id/gyro with 3 floats
        match (args.get(0), args.get(1), args.get(2)) {
            (Some(OscArg::Float(x)), Some(OscArg::Float(y)), Some(OscArg::Float(z))) => {
                // Stocker les données
                if let Ok(mut gyro) = get_gyroscope_mut().lock() {
                    *gyro = (*x, *y, *z);
                }
                // Afficher uniquement XYZ
                println!("{:.6} {:.6} {:.6}", x, y, z);
            }
            _ => {}
        }
    }
}
