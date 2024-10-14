#include "battery_manager.h"
#include "defines.h"
#include "globals.h"
#include "internal_led.h"
#include "log.h"
#include "net/connection_manager.h"
#include "serial_manager.h"
#include "trackers/tracker_manager.h"

SerialManager serial_manager;
ConnectionManager connection_manager;
ConfigManager g_config_manager;
TrackerManager g_tracker_manager;
BatteryManager battery_manager(BATTERY_MONITOR_PIN);

uint64_t last_loop_time = 0;
uint32_t iterations = 0;
uint64_t delta_sum = 0;

void setup() {
    pinMode(INTERNAL_LED_PIN, OUTPUT);
    digitalWrite(INTERNAL_LED_PIN, HIGH);
    Serial.begin(14400);
    connection_manager.setup();
    g_config_manager.setup();
    g_tracker_manager.setup();
}

void loop() {
    serial_manager.parse_incomming_command(connection_manager.get_wifi());
    connection_manager.update();

    if (connection_manager.is_connected()) {
        float level = battery_manager.get_battery_level();
        if (level != 0) {
            connection_manager.send_battery_level(level);
        }

        bool has_new_data = g_tracker_manager.update();
        if (has_new_data) {
            connection_manager.send_tracker_data();
        }
    }

    uint64_t delta = millis() - last_loop_time;
#if ENABLE_FPS_LOG == 1
    delta_sum += delta;
    iterations += 1;
    if (delta_sum > 2000) {
        LOG_INFO("Loop on average %ffps", 1000.f / ((float)delta_sum / (float)iterations));
        iterations = 0;
        delta_sum = 0;
    }
#endif

#ifdef TARGET_LOOP_DELTA_MS
    if (TARGET_LOOP_DELTA_MS > delta) {
        delay(TARGET_LOOP_DELTA_MS - delta);
    }
#endif
    last_loop_time = millis();
}
