mod quaternion_to_stay_rotation;

pub use quaternion_to_stay_rotation::{
    update_orientation,
    set_zero,
    set_max_angle,
    set_steering_axis,
    set_accel_axis,
    apply_calibration,
    get_steering,
    get_accel,
    get_components,
    has_data,
    has_reference,
    AXIS_NONE,
};
