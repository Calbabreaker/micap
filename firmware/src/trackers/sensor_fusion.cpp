#include "sensor_fusion.h"
#include "globals.h"
#include "maths.h"

SensorFusion::SensorFusion(float gyro_hz, float gyro_range) {
    // Setup Fusion
    FusionOffsetInitialise(&offset, gyro_hz);
    FusionAhrsInitialise(&m_ahrs);
    const FusionAhrsSettings settings = {
        .convention = FusionConventionNwu,
        .gain = 0.5f,
        .gyroscopeRange = gyro_range,
        .accelerationRejection = 10.0f,
        .magneticRejection = 0.0f,
        .recoveryTriggerPeriod = 5,
    };
    FusionAhrsSetSettings(&m_ahrs, &settings);
}

// TODO check if order of gyro /accel matters
void SensorFusion::update_gyro(float gyro_xyz[3], float deltatime) {
    FusionVector new_gyro = FusionOffsetUpdate(&offset, *(FusionVector*)gyro_xyz);
    FusionAhrsUpdateNoMagnetometer(&m_ahrs, new_gyro, m_proper_accel, deltatime);
};

// Updates with direct acceleration  from the accelerometer (in g)
void SensorFusion::update_accel(float accel_xyz[3]) {
    m_proper_accel = *(FusionVector*)accel_xyz;
}

Quaternion SensorFusion::get_orientation() {
    FusionQuaternion quat = FusionAhrsGetQuaternion(&m_ahrs);
    return Quaternion(quat.element.x, quat.element.y, quat.element.z, quat.element.w);
}

// Gets the acceleration relative to surface of earth
Vector3 SensorFusion::get_acceleration() const {
    FusionVector earth = FusionAhrsGetEarthAcceleration(&m_ahrs);
    Vector3 acceleration(earth.axis.x, earth.axis.y, earth.axis.z);
    return acceleration * EARTH_GRAVITY;
}
