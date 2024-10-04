#include "battery_manager.h"

#include "defines.h"
#include "globals.h"
#include <Arduino.h>

float mapfloat(float x, float in_min, float in_max, float out_min, float out_max) {
    return (x - in_min) * (out_max - out_min) / (in_max - in_min) + out_min;
}

float BatteryManager::get_battery_level() {
    if (m_check_level_timer.elapsed(BATTERY_MONITOR_INTERVAL_MS)) {
        float voltage = analogRead(m_pin) * 5.f / 1023.f;
        float level = mapfloat(voltage, 3.2f, 3.8f, 0.f, 1.f);
        level = constrain(level, 0.f, 1.f);
        LOG_TRACE(
            "Battery percentage: %f, voltage: %f, value %i", level, voltage, analogRead(m_pin)
        );
        m_check_level_timer.reset();
        return level;
    }

    return 0;
}
