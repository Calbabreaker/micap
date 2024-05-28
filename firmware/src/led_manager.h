#pragma once

#include <cstdint>
#include <pins_arduino.h>

#ifdef LED_BUILTIN
#define INTERNAL_LED_PIN LED_BUILTIN
#define LED_ENABLED
#else
#define INTERNAL_LED_PIN 0
#endif

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
