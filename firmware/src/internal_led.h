#pragma once

#include <Arduino.h>
#include <cstdint>

#if defined(CUSTOM_LED_PIN)
    #define INTERNAL_LED_PIN CUSTOM_LED_PIN
#elif defined(LED_BUILTIN)
    #define INTERNAL_LED_PIN LED_BUILTIN
#else
    #error "No LED pin detected, please CUSTOM_LED_PIN to a status LED or 0 to not use"
#endif

inline void internal_led_blink(uint32_t on_time) {
    digitalWrite(INTERNAL_LED_PIN, LOW);
    delay(on_time);
    digitalWrite(INTERNAL_LED_PIN, HIGH);
}
