#pragma once

#include <Fusion/Fusion.h>

#include "maths.h"

class SensorFusion {
public:
    // Gyro hz muse be greater than accel hz
    SensorFusion(float gyro_hz, float gyro_range);

    void update_gyro(float gyro_xyz[3], float deltatime);

    // Updates with the proper acceleration from the accelerometer
    void update_accel(float accel_xyz[3]);

    // Gets the acceleration relative to the surface of earth
    Vector3 get_acceleration() const;
    Quaternion get_orientation();

private:
    FusionVector m_proper_accel = {.array = {0, 0, 0}};
    Quaternion m_quat;
    FusionOffset offset;
    FusionAhrs m_ahrs;
};
