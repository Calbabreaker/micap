#pragma once

#include "log.h"
#include <cstdint>

class BatteryManager {
public:
    BatteryManager(uint8_t pin) : m_pin(pin) {}

    // Returns battery level as percentage between 0 and 1, 0 means invalid
    float get_battery_level();

private:
    uint8_t m_pin;
    Timer m_check_level_timer;
};
