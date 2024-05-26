#pragma once

#include <IPAddress.h>
#include <pins_arduino.h>

#define UDP_PORT 5828
// #define TARGET_LOOP_DELTA_MS 16 // about 60hz
#define TARGET_LOOP_DELTA_MS 100

// Uncomment to define hardcoded ip
// #define SERVER_IP IPAddress(192, 168, 0, 0)

#ifdef LED_BUILTIN
#define INTERNAL_LED_PIN LED_BUILTIN
#define LED_ENABLED
#else
#define INTERNAL_LED_PIN 0
#endif
