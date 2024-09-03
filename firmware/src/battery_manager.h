#pragma once

#include "log.h"
#include <cstdint>

class BatteryManager {
public:
    BatteryManager(uint8_t pin) : m_pin(pin) {}

    void update();

private:
    uint8_t m_pin;
    Timer m_check_level_timer;
};
