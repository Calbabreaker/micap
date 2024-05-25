#include "config.h"
#include "led_manager.h"
#include "log.h"
#include "math.h"
#include "net/connection_manager.h"
#include "serial_commands.h"
#include "trackers/tracker_manager.h"

#include <ESP8266WiFi.h>

SerialCommands g_serial_commands;
ConnectionManager g_connection_manager;
LedManager g_internal_led(INTERNAL_LED_PIN);
TrackerManager g_tracker_manager;

float gyro_range;
float accel_range;

float from_raw(int raw, float range) {
    // (LSB/°/s or LSB/m/s^2)
    float sensitivity = 0x8000 / range;
    return (float)raw / sensitivity;
}

void setup() {
    Serial.begin(9600);
    g_internal_led.setup();
    g_tracker_manager.setup();
    g_connection_manager.setup();
}

void loop() {
    g_serial_commands.parse_incomming_command();
    g_connection_manager.update();

    if (g_connection_manager.is_connected()) {
        g_tracker_manager.update();
    }

    delay(100);
}
