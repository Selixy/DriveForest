use std::net::UdpSocket;
use std::sync::atomic::{AtomicBool, AtomicU32, Ordering};
use std::time::{Duration, Instant};
use crate::core::update_orientation;
use crate::network::output::{send_steering_now, send_honk, UNITY_IP, UNITY_PORT};

/// Seuil sur le delta d'accélération entre deux trames (m/s²).
/// Valeur par défaut : 8.0 — un coup rapide suffit, le repos ne déclenche pas.
static SHAKE_THRESHOLD: AtomicU32 = AtomicU32::new(0x41000000); // 8.0f32 en bits
static HONK_FLASH:      AtomicBool = AtomicBool::new(false);

// Accélération précédente (pour calcul du delta)
static PREV_AX: AtomicU32 = AtomicU32::new(0);
static PREV_AY: AtomicU32 = AtomicU32::new(0);
static PREV_AZ: AtomicU32 = AtomicU32::new(0);

pub fn take_honk_flash() -> bool {
    HONK_FLASH.swap(false, Ordering::Relaxed)
}

pub fn set_shake_threshold(value: f32) {
    SHAKE_THRESHOLD.store(value.to_bits(), Ordering::Relaxed);
}

fn get_shake_threshold() -> f32 {
    f32::from_bits(SHAKE_THRESHOLD.load(Ordering::Relaxed))
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

/// Retourne true si un shake a été détecté dans ce bundle.
fn parse_osc_bundle(data: &[u8]) -> bool {
    if !data.starts_with(b"#bundle\0") || data.len() < 16 {
        return false;
    }

    let mut shake = false;
    let mut offset = 16;
    while offset + 4 <= data.len() {
        let size = i32::from_be_bytes(data[offset..offset + 4].try_into().unwrap()) as usize;
        offset += 4;

        if offset + size > data.len() {
            break;
        }

        let element = &data[offset..offset + size];
        offset += size;

        if let Some((address, args)) = parse_osc(element) {
            handle_osc_message(&address, &args, &mut shake);
        }
    }
    shake
}

pub fn listen() {
    let socket = UdpSocket::bind("0.0.0.0:50000")
        .expect("Impossible de binder le socket sur le port 50000");

    let out_socket = UdpSocket::bind("0.0.0.0:0")
        .expect("Impossible de créer le socket de sortie");
    let target = format!("{}:{}", UNITY_IP, UNITY_PORT);

    // Cooldown entre deux klaxons (évite le spam)
    let mut last_honk = Instant::now() - Duration::from_secs(2);

    println!("Récepteur Quaternion OSC démarré");
    println!("Écoute sur UDP 0.0.0.0:50000");

    let mut buf = [0u8; 4096];

    loop {
        let (len, _addr) = socket.recv_from(&mut buf)
            .expect("Erreur de réception UDP");

        let data = &buf[..len];

        if data.starts_with(b"#bundle\0") {
            let shake = parse_osc_bundle(data);
            send_steering_now(&out_socket, &target);

            if shake && last_honk.elapsed() >= Duration::from_millis(500) {
                send_honk(&out_socket, &target);
                HONK_FLASH.store(true, Ordering::Relaxed);
                last_honk = Instant::now();
            }
        }
    }
}

fn handle_osc_message(address: &str, args: &[OscArg], shake: &mut bool) {
    // Quaternion: /ZIGSIM/[device_id]/quaternion (w, x, y, z)
    if address.ends_with("/quaternion") && args.len() >= 4 {
        if let (Some(OscArg::Float(w)), Some(OscArg::Float(x)),
                Some(OscArg::Float(y)), Some(OscArg::Float(z))) =
            (args.get(0), args.get(1), args.get(2), args.get(3))
        {
            update_orientation(*w, *x, *y, *z);
        }
    }

    // Accéléromètre: /ZIGSIM/[device_id]/accel (x, y, z) en m/s²
    // On mesure le DELTA entre deux trames pour s'affranchir de la gravité.
    // Un seul coup brusque dans n'importe quelle direction suffit à déclencher.
    if address.ends_with("/accel") && args.len() >= 3 {
        if let (Some(OscArg::Float(x)), Some(OscArg::Float(y)), Some(OscArg::Float(z))) =
            (args.get(0), args.get(1), args.get(2))
        {
            let px = f32::from_bits(PREV_AX.load(Ordering::Relaxed));
            let py = f32::from_bits(PREV_AY.load(Ordering::Relaxed));
            let pz = f32::from_bits(PREV_AZ.load(Ordering::Relaxed));

            let dx = x - px;
            let dy = y - py;
            let dz = z - pz;
            let delta = (dx * dx + dy * dy + dz * dz).sqrt();

            // Ne détecter que si on a déjà au moins une trame précédente (évite le faux pic au démarrage)
            if (px != 0.0 || py != 0.0 || pz != 0.0) && delta > get_shake_threshold() {
                *shake = true;
            }

            PREV_AX.store(x.to_bits(), Ordering::Relaxed);
            PREV_AY.store(y.to_bits(), Ordering::Relaxed);
            PREV_AZ.store(z.to_bits(), Ordering::Relaxed);
        }
    }
}
