#include "led_manager.h"

#include <Arduino.h>

void LedManager::setup() {
    pinMode(m_pin, OUTPUT);
}

void LedManager::on() {
    digitalWrite(m_pin, LOW);
}

void LedManager::off() {
    digitalWrite(m_pin, HIGH);
}

void LedManager::blink(uint64_t on_time) {
    on();
    delay(on_time);
    off();
}
