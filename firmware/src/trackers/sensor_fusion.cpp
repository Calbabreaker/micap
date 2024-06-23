#include "sensor_fusion.h"
#include "globals.h"

void SensorFusion::update_gyro(float gyro_xyz[3]) {
    m_vqf.updateGyr(gyro_xyz);
};

// Updates with the proper acceleration  from the accelerometer
void SensorFusion::update_accel(float accel_xyz[3]) {
    m_vqf.updateAcc(accel_xyz);
    m_proper_accel = Vector3(accel_xyz[0], accel_xyz[1], accel_xyz[2]);
}

Quaternion SensorFusion::get_orientation() {

    float quat_wxyz[4];
    m_vqf.getQuat6D(quat_wxyz);
    m_quat = Quaternion(quat_wxyz[1], quat_wxyz[2], quat_wxyz[3], quat_wxyz[0]);
    return m_quat;
}

// Gets the acceleration relative to surface of earth
Vector3 SensorFusion::get_acceleration() const {
    // Taken from:
    // https://github.com/jrowberg/i2cdevlib/blob/master/Arduino/MPU6050/MPU6050_6Axis_MotionApps20.cpp#L522
    Vector3 gravity_unit(
        2 * (m_quat.x * m_quat.z - m_quat.w * m_quat.y),
        2 * (m_quat.w * m_quat.x + m_quat.y * m_quat.z),
        m_quat.w * m_quat.w - m_quat.x * m_quat.x - m_quat.y * m_quat.y + m_quat.z * m_quat.z
    );

    return m_proper_accel - gravity_unit * EARTH_GRAVITY;
}
