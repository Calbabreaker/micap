#include "consts.h"
#include "log.h"
#include "wifi_manager.h"

#include <BMI160Gen.h>
#include <ESP8266WiFi.h>

float gyro_range;
float accel_range;

// We will be using 3D Vectors here
void convert_from_raw(int raw[], float out[], float range) {
    for (size_t i = 0; i < 3; i++) {
        // (LSB/°/s or LSB/m/s^2)
        float sensitivity = 0x8000 / range;
        out[i] = (float)raw[i] / sensitivity;
    }
}

void setup() {
    Serial.begin(9600);

    WiFiManager::setup();

    LOG("Initializing IMU device...\n");
    Wire.begin();
    BMI160.begin(BMI160GenClass::I2C_MODE, Wire, 0x68);
    LOG("BMI160.begin finished");

    LOG("DEVICE ID: %x\n", BMI160.getDeviceID());
    gyro_range = (float)BMI160.getGyroRange();
    accel_range = (float)BMI160.getAccelerometerRange() * 9.8;
}

void loop() {

    WiFiManager::monitor();
    // int packetSize = udp.parsePacket();
    // if (packetSize) {
    //     char buffer[255];
    //     int len = udp.read(buffer, 255);
    //     if (len > 0) {
    //         buffer[len] = 0;
    //     }
    //     Serial.printf("UDP packet contents: %s\n", buffer);
    // }
    //
    int raw_data[3];
    float imu_data[6];

    BMI160.readGyro(raw_data[0], raw_data[1], raw_data[2]);
    convert_from_raw(raw_data, imu_data, gyro_range);

    BMI160.readAccelerometer(raw_data[0], raw_data[1], raw_data[2]);
    convert_from_raw(raw_data, &imu_data[3], accel_range);

    // send the packet
    udp.beginPacket("10.136.41.71", UDP_PORT);
    // udp.beginPacket("192.168.20.23", UDP_PORT);
    udp.write((uint8_t*)imu_data, sizeof(imu_data));
    udp.endPacket();

    delay(100);
}
