use std::net::UdpSocket;
use crate::core::update_orientation;

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
        return None;
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
    // Vérifier si c'est un bundle
    if !data.starts_with(b"#bundle\0") || data.len() < 16 {
        return;
    }

    let mut offset = 16;
    while offset + 4 <= data.len() {
        let size = i32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + size > data.len() {
            break;
        }

        let element = &data[offset..offset + size];
        offset += size;

        // Parser l'élément comme un message OSC
        if let Some((address, args)) = parse_osc(element) {
            handle_quaternion_data(&address, &args);
        }
    }
}

pub fn listen() {
    let socket = UdpSocket::bind("0.0.0.0:50000")
        .expect("Impossible de binder le socket sur le port 50000");
    
    println!("🎧 Récepteur Quaternion OSC démarré");
    println!("📡 Écoute sur UDP 0.0.0.0:50000");
    println!("⏱️  Fréquence: ~60 FPS\n");

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

fn handle_quaternion_data(address: &str, args: &[OscArg]) {
    // Quaternion: /ZIGSIM/[device_id]/quaternion (w, x, y, z)
    if address.ends_with("/quaternion") && args.len() >= 4 {
        match (args.get(0), args.get(1), args.get(2), args.get(3)) {
            (Some(OscArg::Float(w)), Some(OscArg::Float(x)), Some(OscArg::Float(y)), Some(OscArg::Float(z))) => {
                // Mettre à jour le buffer du quaternion dans le module core
                update_orientation(*w, *x, *y, *z);
            }
            _ => {}
        }
    }
}
