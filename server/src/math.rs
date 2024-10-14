pub fn locked_with_y(quat: glam::Quat, order: glam::EulerRot) -> glam::Quat {
    let (_, yaw, _) = quat.to_euler(order);
    glam::Quat::from_euler(glam::EulerRot::XYZ, 0., yaw, 0.)
}

// Returns the euler angles orientation as a vector in degrees
pub fn to_euler_angles_vector(orientation: glam::Quat, order: glam::EulerRot) -> glam::Vec3A {
    let angles = orientation.to_euler(order);
    glam::Vec3A::new(
        angles.0.to_degrees(),
        angles.1.to_degrees(),
        angles.2.to_degrees(),
    )
}
