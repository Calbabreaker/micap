#include "tracker_bmi160.h"
#include "bmi160.h"
#include "bmi160_defs.h"
#include "log.h"
#include <Wire.h>

// float from_raw(int raw, float range) {
//     // (LSB/°/s or LSB/m/s^2)
//     float sensitivity = 0x8000 / range;
//     return (float)raw / sensitivity;
// }

int8_t i2c_read(uint8_t dev_addr, uint8_t reg_addr, uint8_t* data, uint16_t len) {
    Wire.beginTransmission(dev_addr);
    Wire.write(reg_addr);
    if (Wire.endTransmission() != 0) {
        LOG_ERROR("Wire.endTransmission() failed for address 0x%02x while reading", dev_addr);
        return BMI160_E_COM_FAIL;
    }

    if (len > 0) {
        Wire.requestFrom(dev_addr, (uint8_t)len);

        uint8_t i = 0;
        while (i < len && Wire.available()) {
            data[i] = Wire.read();
            i++;
        }

        if (i != len) {
            LOG_ERROR("Expected length %d but got %d", len, i);
            return BMI160_READ_WRITE_LENGHT_INVALID;
        }
    }

    return BMI160_OK;
}

int8_t i2c_write(uint8_t dev_addr, uint8_t reg_addr, uint8_t* data, uint16_t len) {
    Wire.beginTransmission(dev_addr);
    Wire.write(reg_addr);

    for (int i = 0; i < len; i++) {
        Wire.write(data[i]);
    }

    if (Wire.endTransmission() != 0) {
        LOG_ERROR("Wire.endTransmission() failed for address 0x%02x while writing", dev_addr);
        return BMI160_E_COM_FAIL;
    }

    return BMI160_OK;
}

void delay_ms(uint32_t ms) {
    delay(ms);
}

void TrackerBMI160::setup() {
    m_device.id = m_address;
    m_device.interface = BMI160_I2C_INTF;
    m_device.read = i2c_read;
    m_device.write = i2c_write;
    m_device.delay_ms = delay_ms;
    int8_t result = bmi160_init(&m_device);

    // Set the imu config (values good for general motion capture)
    m_device.accel_cfg.odr = BMI160_ACCEL_ODR_200HZ;
    m_device.accel_cfg.range = BMI160_ACCEL_RANGE_4G;
    m_device.accel_cfg.bw = BMI160_ACCEL_BW_NORMAL_AVG4;
    m_device.accel_cfg.power = BMI160_ACCEL_NORMAL_MODE;
    m_device.gyro_cfg.odr = BMI160_GYRO_ODR_400HZ;
    m_device.gyro_cfg.range = BMI160_GYRO_RANGE_500_DPS;
    m_device.gyro_cfg.bw = BMI160_GYRO_BW_NORMAL_MODE;
    m_device.gyro_cfg.power = BMI160_GYRO_NORMAL_MODE;
    result = bmi160_set_sens_conf(&m_device);

    if (result != BMI160_OK) {
        LOG_ERROR("Failed to initialize BMI160 with address 0x%02x", m_address);
        this->status = TrackerStatus::Error;
    }
}

void TrackerBMI160::update() {
    struct bmi160_sensor_data raw_accel;
    struct bmi160_sensor_data raw_gyro;
    int8_t result =
        bmi160_get_sensor_data(BMI160_BOTH_ACCEL_AND_GYRO, &raw_accel, &raw_gyro, &m_device);

    this->acceleration = Vector3(raw_accel.x, raw_accel.y, raw_accel.z);
    // Definitely not correct
    this->orientation = Quaternion(raw_gyro.x, raw_gyro.y, raw_gyro.z, 1.0);

    if (result != BMI160_OK) {
        LOG_ERROR("BMI160 tracker error %d", result);
        this->status = TrackerStatus::Error;
    }
}
