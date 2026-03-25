use nalgebra::{Quaternion, UnitQuaternion, Vector3};
use std::sync::{Mutex, OnceLock};

static ORIENTATION_BUFFER: OnceLock<Mutex<PhoneOrientation>> = OnceLock::new();

fn get_buffer() -> &'static Mutex<PhoneOrientation> {
    ORIENTATION_BUFFER.get_or_init(|| Mutex::new(PhoneOrientation::new()))
}

#[derive(Clone)]
pub struct PhoneOrientation {
    current: UnitQuaternion<f32>,
    reference_forward: Vector3<f32>,
    reference_up: Vector3<f32>,
}

impl PhoneOrientation {
    pub fn new() -> Self {
        Self {
            current: UnitQuaternion::identity(),
            reference_forward: Vector3::new(1.0, 0.0, 0.0),
            reference_up: Vector3::new(0.0, 0.0, 1.0),
        }
    }

    pub fn update_from_quaternion(&mut self, w: f32, x: f32, y: f32, z: f32) {
        self.current = UnitQuaternion::new_normalize(Quaternion::new(w, x, y, z));
    }

    pub fn set_zero(&mut self) {
        self.reference_forward = (self.current * Vector3::new(1.0, 0.0, 0.0)).normalize();
        self.reference_up = (self.current * Vector3::new(0.0, 1.0, 0.0)).normalize();
    }

    /// Affiche les 3 axes locaux du téléphone dans le repère monde
    /// Appelle cette fonction dans tes 3 positions de test pour trouver le bon axe
    pub fn debug(&self) {
        let x = self.current * Vector3::new(1.0, 0.0, 0.0);
        let y = self.current * Vector3::new(0.0, 1.0, 0.0);
        let z = self.current * Vector3::new(0.0, 0.0, 1.0);
        println!("--- DEBUG AXES ---");
        println!("X local: [{:.3}, {:.3}, {:.3}]", x.x, x.y, x.z);
        println!("Y local: [{:.3}, {:.3}, {:.3}]", y.x, y.y, y.z);
        println!("Z local: [{:.3}, {:.3}, {:.3}]", z.x, z.y, z.z);
        println!("------------------");
    }

    pub fn steering(&self) -> f32 {
        let current_forward = self.current * Vector3::new(1.0, 0.0, 0.0);
        let up = self.reference_up;

        let ref_proj = self.reference_forward - up * up.dot(&self.reference_forward);
        let cur_proj = current_forward - up * up.dot(&current_forward);

        let ref_norm = ref_proj.norm();
        let cur_norm = cur_proj.norm();

        if ref_norm < 1e-4 || cur_norm < 1e-4 {
            return 0.0;
        }

        let ref_proj = ref_proj / ref_norm;
        let cur_proj = cur_proj / cur_norm;

        let cos_a = ref_proj.dot(&cur_proj).clamp(-1.0, 1.0);
        let cross = ref_proj.cross(&cur_proj);
        let sign = up.dot(&cross).signum();
        let angle = sign * cos_a.acos();

        (angle / std::f32::consts::PI).clamp(-1.0, 1.0)
    }
}

pub fn update_orientation(w: f32, x: f32, y: f32, z: f32) {
    if let Ok(mut o) = get_buffer().lock() {
        o.update_from_quaternion(w, x, y, z);
    }
}

pub fn set_zero() {
    if let Ok(mut o) = get_buffer().lock() {
        o.set_zero();
    }
}

pub fn get_steering() -> f32 {
    if let Ok(o) = get_buffer().lock() {
        o.steering()
    } else {
        0.0
    }
}

/// Appelle cette fonction dans tes 3 positions de test :
/// 1. Position neutre
/// 2. Tourné 90° droite (volant)  
/// 3. Incliné petit bord
/// → Les résultats vont révéler quel axe utiliser
pub fn debug() {
    if let Ok(o) = get_buffer().lock() {
        o.debug();
    }
}