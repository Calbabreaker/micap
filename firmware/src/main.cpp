#include "log.h"
#include "math.h"
#include "net/connection_manager.h"
#include "net/wifi_manager.h"
#include "serial_commands.h"

#include <BMI160Gen.h>
#include <ESP8266WiFi.h>

SerialCommands serial_commands;
ConnectionManager connection_manager;

float gyro_range;
float accel_range;

float from_raw(int raw, float range) {
    // (LSB/°/s or LSB/m/s^2)
    float sensitivity = 0x8000 / range;
    return (float)raw / sensitivity;
}

void setup() {
    Serial.begin(9600);

    connection_manager.setup();

    LOG("Initializing IMU device...\n");
    Wire.begin();
    BMI160.begin(BMI160GenClass::I2C_MODE, Wire, 0x68);
    LOG("BMI160.begin finished");

    LOG("DEVICE ID: %x\n", BMI160.getDeviceID());
    gyro_range = (float)BMI160.getGyroRange();
    accel_range = (float)BMI160.getAccelerometerRange() * 9.8;
}

void loop() {
    serial_commands.parse_incomming_command();
    connection_manager.update();

    if (connection_manager.is_connected()) {
        int raw_data[3];
        BMI160.readAccelerometer(raw_data[0], raw_data[1], raw_data[2]);

        Vector3 accel(
            from_raw(raw_data[0], accel_range), from_raw(raw_data[1], accel_range),
            from_raw(raw_data[2], accel_range)
        );
        connection_manager.send_acceleration(accel);
    }

    delay(100);
}
