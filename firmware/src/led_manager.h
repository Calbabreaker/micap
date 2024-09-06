#pragma once

#include <cstdint>
#include <pins_arduino.h>

#if defined(CUSTOM_LED_PIN)
    #define INTERNAL_LED_PIN CUSTOM_LED_PIN
#elif defined(LED_BUILTIN)
    #define INTERNAL_LED_PIN LED_BUILTIN
#else
    #error "No LED pin detected, please CUSTOM_LED_PIN to a status LED or 0 to not use"
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
