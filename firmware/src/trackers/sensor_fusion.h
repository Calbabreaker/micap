#pragma once

#include <vqf.h>

#include "math.h"

class SensorFusion {
public:
    SensorFusion(float gyro_hz, float accel_hz) : m_vqf(1. / gyro_hz, 1. / accel_hz) {}

    void update_gyro(float gyro_xyz[3]);

    // Updates with the proper acceleration from the accelerometer
    void update_accel(float accel_xyz[3]);

    // Gets the acceleration relative to the surface of earth
    Vector3 get_acceleration() const;
    Quaternion get_orientation();

private:
    VQF m_vqf;

    Vector3 m_proper_accel;
    Quaternion m_quat;
};
