#pragma once

#include <bmi160_defs.h>

#include "tracker.h"
#include "trackers/sensor_fusion.h"

constexpr size_t BMI160_CALIBRATION_SAMPLES = 70;
constexpr size_t BMI160_FIFO_BUFFER_SIZE = 128;

// Change both HZ/RANGE and FLAG when changing config
constexpr uint8_t BMI160_GYRO_ODR_FLAG = BMI160_GYRO_ODR_400HZ;
constexpr float BMI160_GYRO_ODR_HZ = 400.;
constexpr uint8_t BMI160_ACCEL_ODR_FLAG = BMI160_ACCEL_ODR_100HZ;
constexpr float BMI160_ACCEL_ODR_HZ = 100.;

constexpr uint8_t BMI160_GYRO_RANGE_FLAG = BMI160_GYRO_RANGE_1000_DPS;
constexpr float BMI160_GYRO_RANGE = 1000.f;
constexpr uint8_t BMI160_ACCEL_RANGE_FLAG = BMI160_ACCEL_RANGE_4G;
constexpr float BMI160_ACCEL_RANGE = 4.;

// Converts raw gyro output to radians based on sensitivity from datasheet
// LSB/°/s -> degrees/s
const float BMI160_GYRO_CONVERSION =
    1.f / (16.4f * (1 << BMI160_GYRO_RANGE_FLAG)); // 16.4 * 2^index

// Makes accel output scale from -4g to +4g
// LSB/g -> g/s
const float BMI160_ACCEL_CONVERSION = 1.f / ((float)0x8000 / BMI160_ACCEL_RANGE);

class TrackerBMI160 : public Tracker {
public:
    TrackerBMI160(uint8_t index, uint8_t address)
        : Tracker(TrackerKind::BMI160, index, address),
          m_sensor_fusion(BMI160_GYRO_ODR_HZ, BMI160_GYRO_RANGE) {}

    void setup() override final;
    void update() override final;

private:
    bool calibrate();
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
