#include "tracker_bmi160.h"
#include "log.h"
#include "math.h"

#include <Wire.h>
#include <bmi160.h>

// Buffer to store temporary fifo data
static uint8_t FIFO_BUFFER[BMI160_FIFO_BUFFER_SIZE];

// Implementations for i2c read and write functions using Wire.h
int8_t i2c_read(uint8_t dev_addr, uint8_t reg_addr, uint8_t* data, uint16_t len) {
    Wire.beginTransmission(dev_addr);
    Wire.write(reg_addr);
    if (Wire.endTransmission() != 0) {
        LOG_ERROR("Wire.endTransmission() failed for address 0x%02x while reading", dev_addr);
        return BMI160_E_COM_FAIL;
    }

    if (len > 0) {
        Wire.requestFrom(dev_addr, static_cast<uint8_t>(len));

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
    delay(static_cast<uint64_t>(ms));
}

void TrackerBMI160::setup() {
    m_device.id = m_address;
    m_device.interface = BMI160_I2C_INTF;
    m_device.read = i2c_read;
    m_device.write = i2c_write;
    m_device.delay_ms = delay_ms;
    m_device.fifo = &m_fifo;
    int8_t result = bmi160_init(&m_device);

    // Set the imu config (values good for general motion capture)
    m_device.accel_cfg.odr = BMI160_ACCEL_ODR_FLAG;
    m_device.accel_cfg.range = BMI160_ACCEL_RANGE_FLAG;
    m_device.accel_cfg.bw = BMI160_ACCEL_BW_NORMAL_AVG4;
    m_device.accel_cfg.power = BMI160_ACCEL_NORMAL_MODE;

    m_device.gyro_cfg.odr = BMI160_GYRO_ODR_FLAG;
    m_device.gyro_cfg.range = BMI160_GYRO_RANGE_FLAG;
    m_device.gyro_cfg.bw = BMI160_GYRO_BW_NORMAL_MODE;
    m_device.gyro_cfg.power = BMI160_GYRO_NORMAL_MODE;
    result = bmi160_set_sens_conf(&m_device);

    // Use FIFO to make sure no data gets missed or is duplicated
    m_device.fifo->data = FIFO_BUFFER;
    result = bmi160_set_fifo_config(
        BMI160_FIFO_ACCEL | BMI160_FIFO_HEADER | BMI160_FIFO_GYRO, BMI160_ENABLE, &m_device
    );

    if (result != BMI160_OK) {
        LOG_ERROR("Failed to initialize BMI160 with address 0x%02x", m_address);
        this->status = TrackerStatus::Error;
    }

    calibrate();
}

void TrackerBMI160::update() {
    int8_t result = read_fifo();

    // Probably means there is no data in fifo buffer
    if (result == BMI160_READ_WRITE_LENGHT_INVALID) {
        return;
    } else if (result != BMI160_OK) {
        LOG_ERROR("BMI160 tracker error %d", result);
        this->status = TrackerStatus::Error;
        return;
    }

    this->orientation = m_sensor_fusion.get_orientation();
    this->acceleration = m_sensor_fusion.get_acceleration();
    this->has_new_data = true;
}

void TrackerBMI160::calibrate() {
    LOG_INFO("Starting calibration");

    // Use Fast Offset Compensation for accelerometer
    struct bmi160_foc_conf foc_conf;
    foc_conf.acc_off_en = BMI160_ENABLE;
    foc_conf.foc_acc_x = BMI160_FOC_ACCEL_0G;
    foc_conf.foc_acc_y = BMI160_FOC_ACCEL_0G;
    foc_conf.foc_acc_z = BMI160_FOC_ACCEL_POSITIVE_G;

    struct bmi160_offsets offset;
    bmi160_start_foc(&foc_conf, &offset, &m_device);
    m_accel_offsets[0] = (float)offset.off_acc_x;
    m_accel_offsets[1] = (float)offset.off_acc_y;
    m_accel_offsets[2] = (float)offset.off_acc_z;

    // FOC doesn't work with gyro for some reason so do it by collecting samples
    const float num_samples = 100;
    int32_t gyro_sum_xyz[3] = {0, 0, 0};
    for (int i = 0; i < num_samples; i++) {
        struct bmi160_sensor_data raw_gyro;
        bmi160_get_sensor_data(BMI160_GYRO_SEL, nullptr, &raw_gyro, &m_device);
        gyro_sum_xyz[0] += raw_gyro.x;
        gyro_sum_xyz[1] += raw_gyro.y;
        gyro_sum_xyz[2] += raw_gyro.z;

        delay(20);
    }

    // Calculate average offsets
    for (size_t i = 0; i < 3; i++) {
        m_gyro_offsets[i] = (float)gyro_sum_xyz[i] / num_samples;
    }

    LOG_INFO("Finished calibration: ");
    LOG_INFO("GYRO: [%f, %f, %f]", m_gyro_offsets[0], m_gyro_offsets[1], m_gyro_offsets[2]);
    LOG_INFO("ACCEL: [%f, %f, %f]", m_accel_offsets[0], m_accel_offsets[1], m_accel_offsets[2]);
}

uint8_t TrackerBMI160::read_fifo() {
    m_fifo.length = BMI160_FIFO_BUFFER_SIZE;
    // Number of bytes read written into m_fifo.length
    uint8_t result = bmi160_get_fifo_data(&m_device);

    int16_t sensor_data[3];

    // The library bmi160.h does provide functions to read FIDO data but it comes seperately so we
    // read it by ourselves instead
    for (size_t i = 0; i < m_fifo.length;) {
        // Ignore iterrupt flags
        uint8_t header = m_fifo.data[i] & BMI160_FIFO_TAG_INTR_MASK;
        i++;

        if (header == BMI160_FIFO_HEAD_SKIP_FRAME || header == BMI160_FIFO_HEAD_INPUT_CONFIG) {
            i++;
        } else if (header == BMI160_FIFO_HEAD_SENSOR_TIME) {
            i += BMI160_SENSOR_TIME_LENGTH;
        } else {
            // Data comes in the order magnometer, gyro, then accel
            // Check if header has the sensor data bits set
            if ((header & BMI160_FIFO_HEAD_M) == BMI160_FIFO_HEAD_M) {
                // Not using magnometer
                i += BMI160_FIFO_M_LENGTH;
            }

            if ((header & BMI160_FIFO_HEAD_G) == BMI160_FIFO_HEAD_G) {
                if (fifo_unpack_i16(&i, 3, sensor_data)) {
                    handle_raw_gyro(sensor_data);
                }
            }

            if ((header & BMI160_FIFO_HEAD_A) == BMI160_FIFO_HEAD_A) {
                if (fifo_unpack_i16(&i, 3, sensor_data)) {
                    handle_raw_accel(sensor_data);
                }
            }
        }
    }

    return result;
}

// Takes out int16_ts into out based on index from the fifo buffer
// Returns whether or not the size will be contained within the fifo buffer
bool TrackerBMI160::fifo_unpack_i16(size_t* index, size_t count, int16_t* out) {
    size_t start_index = *index;
    *index += count * sizeof(int16_t);
    if (*index >= m_fifo.length) {
        return false;
    }

    for (size_t i = 0; i < count; i++) {
        uint16_t lsb = m_fifo.data[start_index + i * sizeof(int16_t) + 0];
        uint16_t msb = m_fifo.data[start_index + i * sizeof(int16_t) + 1];
        out[i] = (int16_t)((msb << 8) | lsb);
    }

    return true;
}

void TrackerBMI160::handle_raw_accel(int16_t raw_accel[3]) {
    float accel_xyz[3];
    for (size_t i = 0; i < 3; i++) {
        accel_xyz[i] = ((float)raw_accel[i] - m_accel_offsets[i]) * BMI160_ACCEL_CONVERSION;
    }

    m_sensor_fusion.update_accel(accel_xyz);
}

void TrackerBMI160::handle_raw_gyro(int16_t raw_gyro[3]) {
    float gyro_xyz[3];
    for (size_t i = 0; i < 3; i++) {
        gyro_xyz[i] = ((float)raw_gyro[i] - m_gyro_offsets[i]) * BMI160_GYRO_CONVERSION;
    }

    m_sensor_fusion.update_gyro(gyro_xyz);
}
