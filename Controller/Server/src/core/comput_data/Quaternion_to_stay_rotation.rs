use nalgebra::{Quaternion, UnitQuaternion};
use std::sync::{Mutex, OnceLock};

static ORIENTATION_BUFFER: OnceLock<Mutex<PhoneOrientation>> = OnceLock::new();

fn get_buffer() -> &'static Mutex<PhoneOrientation> {
    ORIENTATION_BUFFER.get_or_init(|| Mutex::new(PhoneOrientation::new()))
}

/// Valeur sentinelle pour "axe désactivé"
pub const AXIS_NONE: u8 = 255;

#[derive(Clone)]
pub struct PhoneOrientation {
    current:       UnitQuaternion<f32>,
    reference:     UnitQuaternion<f32>,
    has_data:      bool,
    has_reference: bool,
    max_angle:     f32,
    /// Axe braquage  : 0=X 1=Y 2=Z  255=désactivé
    steering_axis: u8,
    /// Axe accélération : 0=X 1=Y 2=Z  255=désactivé
    accel_axis:    u8,
    /// Signe braquage : 1.0 normal, -1.0 inversé (déterminé à la calibration)
    steering_sign: f32,
    /// Signe accélération : 1.0 normal, -1.0 inversé
    accel_sign:    f32,
}

impl PhoneOrientation {
    pub fn new() -> Self {
        Self {
            current:       UnitQuaternion::identity(),
            reference:     UnitQuaternion::identity(),
            has_data:      false,
            has_reference: false,
            max_angle:     std::f32::consts::PI,
            steering_axis: 2,
            accel_axis:    AXIS_NONE,
            steering_sign: 1.0,
            accel_sign:    1.0,
        }
    }

    pub fn apply_calibration(&mut self, steer_axis: u8, steer_sign: f32, accel_axis: u8, accel_sign: f32) {
        self.steering_axis = steer_axis;
        self.steering_sign = steer_sign;
        self.accel_axis    = accel_axis;
        self.accel_sign    = accel_sign;
    }

    pub fn update_from_quaternion(&mut self, w: f32, x: f32, y: f32, z: f32) {
        self.current = UnitQuaternion::new_normalize(Quaternion::new(w, x, y, z));
        self.has_data = true;
    }

    pub fn set_zero(&mut self) {
        self.reference     = self.current;
        self.has_reference = true;
    }

    pub fn set_max_angle(&mut self, radians: f32) {
        self.max_angle = radians.max(std::f32::consts::FRAC_PI_8);
    }

    pub fn set_steering_axis(&mut self, axis: u8) { self.steering_axis = axis; }
    pub fn set_accel_axis(&mut self, axis: u8)    { self.accel_axis    = axis; }

    fn relative_components(&self) -> Option<(f32, f32, f32, f32)> {
        if !self.has_reference { return None; }
        let q = (self.reference.inverse() * self.current).into_inner();
        if q.w >= 0.0 {
            Some((q.w, q.i, q.j, q.k))
        } else {
            Some((-q.w, -q.i, -q.j, -q.k))
        }
    }

    fn angle_for_axis(&self, axis: u8, w: f32, i: f32, j: f32, k: f32) -> f32 {
        let component = match axis {
            0 => i,
            1 => j,
            2 => k,
            _ => return 0.0,
        };
        2.0 * component.atan2(w)
    }

    pub fn steering(&self) -> f32 {
        if self.steering_axis == AXIS_NONE { return 0.0; }
        let Some((w, i, j, k)) = self.relative_components() else { return 0.0; };
        let angle = self.angle_for_axis(self.steering_axis, w, i, j, k);
        self.steering_sign * (-(angle / self.max_angle).clamp(-1.0, 1.0))
    }

    pub fn accel(&self) -> f32 {
        if self.accel_axis == AXIS_NONE { return 0.0; }
        let Some((w, i, j, k)) = self.relative_components() else { return 0.0; };
        let angle = self.angle_for_axis(self.accel_axis, w, i, j, k);
        self.accel_sign * (-(angle / self.max_angle).clamp(-1.0, 1.0))
    }

    pub fn components(&self) -> (f32, f32, f32) {
        match self.relative_components() {
            Some((_, i, j, k)) => (i, j, k),
            None               => (0.0, 0.0, 0.0),
        }
    }
}

// ─── API publique ──────────────────────────────────────────────────────────────

pub fn update_orientation(w: f32, x: f32, y: f32, z: f32) {
    if let Ok(mut o) = get_buffer().lock() { o.update_from_quaternion(w, x, y, z); }
}

pub fn set_zero() {
    if let Ok(mut o) = get_buffer().lock() { o.set_zero(); }
}

pub fn set_max_angle(radians: f32) {
    if let Ok(mut o) = get_buffer().lock() { o.set_max_angle(radians); }
}

pub fn set_steering_axis(axis: u8) {
    if let Ok(mut o) = get_buffer().lock() { o.set_steering_axis(axis); }
}

pub fn set_accel_axis(axis: u8) {
    if let Ok(mut o) = get_buffer().lock() { o.set_accel_axis(axis); }
}

pub fn apply_calibration(steer_axis: u8, steer_sign: f32, accel_axis: u8, accel_sign: f32) {
    if let Ok(mut o) = get_buffer().lock() {
        o.apply_calibration(steer_axis, steer_sign, accel_axis, accel_sign);
    }
}

pub fn get_steering() -> f32 {
    if let Ok(o) = get_buffer().lock() { o.steering() } else { 0.0 }
}

pub fn get_accel() -> f32 {
    if let Ok(o) = get_buffer().lock() { o.accel() } else { 0.0 }
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
