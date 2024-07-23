#pragma once

#include <cstdint>

class BatteryManager {
public:
    BatteryManager(uint8_t pin) : m_pin(pin) {}

    void update();

private:
    uint8_t m_pin;
    uint64_t m_last_check_level_time = 0;
};
