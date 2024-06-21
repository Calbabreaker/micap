#include "defines.h"
#include "globals.h"
#include "log.h"
#include "serial_manager.h"

#include <i2c_clearbus.h>

SerialManager serial_manager;
ConnectionManager g_connection_manager;
ConfigManager g_config_manager;
LedManager g_internal_led(INTERNAL_LED_PIN);
TrackerManager g_tracker_manager;

uint64_t last_loop_time = 0;
uint32_t iterations = 0;
uint64_t delta_sum = 0;

void setup() {
    Serial.begin(9600);
    g_config_manager.setup();
    g_internal_led.setup();
    g_internal_led.off();
    g_tracker_manager.setup();
    g_connection_manager.setup();

    // Make sure i2c bus doesn't get stuck
    I2C_ClearBus();
}

void loop() {
    serial_manager.parse_incomming_command();
    g_connection_manager.update();
    g_tracker_manager.update();

    if (g_connection_manager.is_connected()) {
        g_connection_manager.send_tracker_data();
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
