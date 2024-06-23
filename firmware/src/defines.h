#pragma once

#include <IPAddress.h>

#define UDP_PORT 5828

#define TARGET_LOOP_DELTA_MS 8 // about 120hz
// // #define TARGET_LOOP_DELTA_MS 16 // about 60hz
// #define TARGET_LOOP_DELTA_MS 64

// Uncomment to define hardcoded ip
// #define SERVER_IP IPAddress(10, 136, 41, 71)

#define MAX_TRACKER_COUNT 2

// Number of unique wifi ssid and passwords that can be stored
// Each wifi entry takes 96 bytes
#define MAX_WIFI_ENTRIES 4

// #define CUSTOM_LED_PIN D8
