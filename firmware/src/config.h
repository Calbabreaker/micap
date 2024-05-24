#pragma once

#include <IPAddress.h>
#include <cstdint>
#include <pins_arduino.h>

#define UDP_PORT 5828

// Uncomment if using a hardcoded ip
// #define SERVER_IP IPAddress(192, 168, 0, 0)

#ifdef LED_BUILTIN
#define INTERNAL_LED_PIN LED_BUILTIN
#define LED_ENABLED
#else
#define INTERNAL_LED_PIN 0
#endif
