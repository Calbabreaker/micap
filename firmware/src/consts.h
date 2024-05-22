#pragma once

#include <cstdint>
#include <pins_arduino.h>

const uint16_t UDP_PORT = 5828;

#ifdef LED_BUILTIN
#define INTERNAL_LED_PIN LED_BUILTIN
#define LED_ENABLED
#else
#define LED_PIN 0
#endif
