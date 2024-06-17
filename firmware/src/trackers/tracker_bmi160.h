#pragma once

#include <bmi160_defs.h>

#include "globals.h"
#include "tracker.h"

// Makes gyro output scale from -500 to +500 °/s (sensor outputs a signed 16-bit integer)
const uint8_t BMI160_GYRO_RANGE_FLAG = BMI160_GYRO_RANGE_500_DPS;
const float BMI160_GYRO_SCALE = 500. / 0x8000; // LSB/°/s -> °/s

// Makes accel output scale from -4 to +4 g
const uint8_t BMI160_ACCEL_RANGE_FLAG = BMI160_ACCEL_RANGE_4G;
const float BMI160_ACCEL_SCALE = EARTH_GRAVITY * (4. / 0x8000); // LSB/g -> m/s^2

class TrackerBMI160 : public Tracker {

public:
    TrackerBMI160(uint8_t index, uint8_t address) : Tracker(TrackerKind::BMI160, index, address) {}

    void setup() override final;
    void update() override final;

private:
    bmi160_dev m_device;
};
