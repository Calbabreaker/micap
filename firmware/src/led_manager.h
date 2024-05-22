#pragma once

#include <cstdint>

class LedManager {
public:
    LedManager(uint8_t pin) : m_pin(pin) {}

    void setup();
    void on();
    void off();
    void blink(uint64_t on_time);

private:
    uint8_t m_pin;
};
