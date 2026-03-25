use nalgebra::{Quaternion, UnitQuaternion};
use std::sync::{Mutex, OnceLock};

static ORIENTATION_BUFFER: OnceLock<Mutex<PhoneOrientation>> = OnceLock::new();

fn get_buffer() -> &'static Mutex<PhoneOrientation> {
    ORIENTATION_BUFFER.get_or_init(|| Mutex::new(PhoneOrientation::new()))
}

#[derive(Clone)]
pub struct PhoneOrientation {
    current:       UnitQuaternion<f32>,
    reference:     UnitQuaternion<f32>,
    has_data:      bool,
    has_reference: bool,
    /// Angle maximal de braquage (en radians). Normalement PI (180°).
    max_angle:     f32,
    /// Axe de décomposition : 0 = i (X), 1 = j (Y), 2 = k (Z).
    axis:          u8,
}

impl PhoneOrientation {
    pub fn new() -> Self {
        Self {
            current:       UnitQuaternion::identity(),
            reference:     UnitQuaternion::identity(),
            has_data:      false,
            has_reference: false,
            max_angle:     std::f32::consts::PI,
            axis:          2,
        }
    }

    pub fn update_from_quaternion(&mut self, w: f32, x: f32, y: f32, z: f32) {
        self.current = UnitQuaternion::new_normalize(Quaternion::new(w, x, y, z));
        self.has_data = true;
    }

    /// Enregistre la position actuelle comme position neutre du volant.
    pub fn set_zero(&mut self) {
        self.reference = self.current;
        self.has_reference = true;
    }

    pub fn set_max_angle(&mut self, radians: f32) {
        self.max_angle = radians.max(std::f32::consts::FRAC_PI_8);
    }

    pub fn set_axis(&mut self, axis: u8) {
        self.axis = axis.min(2);
    }

    /// Calcule l'angle de braquage normalisé dans [-1.0, 1.0].
    pub fn steering(&self) -> f32 {
        if !self.has_reference {
            return 0.0;
        }

        let q_rel = self.reference.inverse() * self.current;
        let q = q_rel.into_inner();

        // Correction double-cover : q et -q représentent la même rotation.
        let (w, i, j, k) = if q.w >= 0.0 {
            (q.w, q.i, q.j, q.k)
        } else {
            (-q.w, -q.i, -q.j, -q.k)
        };

        let component = match self.axis {
            0 => i,
            1 => j,
            _ => k,
        };

        let angle = 2.0 * component.atan2(w);
        (angle / self.max_angle).clamp(-1.0, 1.0)
    }

    /// Retourne (i, j, k) du quaternion relatif après correction double-cover.
    pub fn components(&self) -> (f32, f32, f32) {
        if !self.has_reference {
            return (0.0, 0.0, 0.0);
        }
        let q_rel = self.reference.inverse() * self.current;
        let q = q_rel.into_inner();
        if q.w >= 0.0 {
            (q.i, q.j, q.k)
        } else {
            (-q.i, -q.j, -q.k)
        }
    }
}

// ─── API publique ─────────────────────────────────────────────────────────────

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

pub fn set_max_angle(radians: f32) {
    if let Ok(mut o) = get_buffer().lock() {
        o.set_max_angle(radians);
    }
}

pub fn set_axis(axis: u8) {
    if let Ok(mut o) = get_buffer().lock() {
        o.set_axis(axis);
    }
}

pub fn get_steering() -> f32 {
    if let Ok(o) = get_buffer().lock() { o.steering() } else { 0.0 }
}

pub fn get_components() -> (f32, f32, f32) {
    if let Ok(o) = get_buffer().lock() { o.components() } else { (0.0, 0.0, 0.0) }
}

pub fn has_data() -> bool {
    if let Ok(o) = get_buffer().lock() { o.has_data } else { false }
}

pub fn has_reference() -> bool {
    if let Ok(o) = get_buffer().lock() { o.has_reference } else { false }
}
