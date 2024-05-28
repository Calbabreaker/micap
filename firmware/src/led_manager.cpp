#include "led_manager.h"
#include "defines.h"

#include <Arduino.h>

void LedManager::setup() {
#ifdef LED_ENABLED
    pinMode(m_pin, OUTPUT);
#endif
}

void LedManager::on() {
#ifdef LED_ENABLED
    digitalWrite(m_pin, LOW);
#endif
}

void LedManager::off() {
#ifdef LED_ENABLED
    digitalWrite(m_pin, HIGH);
#endif
}

void LedManager::blink(uint64_t on_time) {
    on();
    delay(on_time);
    off();
}
