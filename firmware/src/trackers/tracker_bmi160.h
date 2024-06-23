#pragma once

#include <bmi160_defs.h>
#include <vqf.h>

#include "globals.h"
#include "tracker.h"
#include "trackers/sensor_fusion.h"

const uint32_t BMI160_CALIBRATION_SAMPLES = 200;
const size_t BMI160_FIFO_BUFFER_SIZE = 128;

// Change both HZ and FLAG when changing config
const uint8_t BMI160_GYRO_ODR_FLAG = BMI160_GYRO_ODR_400HZ;
const float BMI160_GYRO_ODR_HZ = 400.;
const uint8_t BMI160_ACCEL_ODR_FLAG = BMI160_ACCEL_ODR_100HZ;
const float BMI160_ACCEL_ODR_HZ = 100.;

const uint8_t BMI160_GYRO_RANGE_FLAG = BMI160_GYRO_RANGE_1000_DPS;
const float BMI160_GYRO_SENSITIVITY = 16.4f * (1 << BMI160_GYRO_RANGE_FLAG);
const uint8_t BMI160_ACCEL_RANGE_FLAG = BMI160_ACCEL_RANGE_4G;
const float BMI160_ACCEL_RANGE = 4.;

// Makes gyro output scale from -500 to +500 °/s (sensor outputs a signed 16-bit integer)
// LSB/°/s -> radians/s/LSB
const float BMI160_GYRO_CONVERSION = ((PI / 180.f) / BMI160_GYRO_SENSITIVITY);

// Makes accel output scale from -4g to +4g
// LSB/g -> m/s^2
const float BMI160_ACCEL_CONVERSION = EARTH_GRAVITY / ((float)0x8000 / BMI160_ACCEL_RANGE);

class TrackerBMI160 : public Tracker {

public:
    TrackerBMI160(uint8_t index, uint8_t address)
        : Tracker(TrackerKind::BMI160, index, address),
          m_sensor_fusion(BMI160_GYRO_ODR_HZ, BMI160_ACCEL_ODR_HZ) {}

    void setup() override final;
    void update() override final;
    void calibrate() override final;

private:
    bool read_fifo();
    bool fifo_unpack_i16(size_t* index, size_t count, int16_t* out);

    float get_temperature();
    void handle_raw_accel(int16_t accel[3]);
    void handle_raw_gyro(int16_t gyro[3]);

private:
    bmi160_dev m_device;
    bmi160_fifo_frame m_fifo;

    float m_accel_offsets[3];
    float m_gyro_offsets[3];

    SensorFusion m_sensor_fusion;
};
