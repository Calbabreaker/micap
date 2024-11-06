pub fn locked_with_yaw(quat: glam::Quat) -> glam::Quat {
    let xyz = quat.xyz().dot(glam::Vec3::Y) * glam::Vec3::Y;
    glam::Quat::from_vec4(xyz.extend(quat.w)).normalize()
}

/// Returns the euler angles orientation as a vector in degrees
pub fn to_euler_angles_vector(orientation: glam::Quat, order: glam::EulerRot) -> glam::Vec3A {
    let angles = orientation.to_euler(order);
    glam::Vec3A::new(
        angles.0.to_degrees(),
        angles.1.to_degrees(),
        angles.2.to_degrees(),
    )
}
