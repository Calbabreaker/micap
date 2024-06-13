#include "defines.h"
#include "globals.h"
#include "log.h"
#include "serial_manager.h"

#include <ESP8266WiFi.h>

SerialManager g_serial_manager;
ConnectionManager g_connection_manager;
ConfigManager g_config_manager;
LedManager g_internal_led(INTERNAL_LED_PIN);
TrackerManager g_tracker_manager;

uint64_t last_loop_time;

void setup() {
    Serial.begin(9600);
    g_config_manager.setup();
    g_internal_led.setup();
    g_tracker_manager.setup();
    g_connection_manager.setup();

    last_loop_time = millis();
}

void loop() {
    g_serial_manager.parse_incomming_command();
    g_connection_manager.update();

    if (g_connection_manager.is_connected()) {
        g_connection_manager.send_tracker_data();
    }

#ifdef TARGET_LOOP_DELTA_MS
    uint64_t delta = millis() - last_loop_time;
    int64_t sleep_time = TARGET_LOOP_DELTA_MS - (int64_t)delta;
    if (sleep_time > 0) {
        delay(sleep_time);
    } else {
        LOG_WARN(
            "Loop took %lu ms which is longer than target " STRINGIFY_V(TARGET_LOOP_DELTA_MS) " ms",
            delta
        );
    }

    last_loop_time = millis();
#endif
}
