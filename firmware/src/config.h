#pragma once

#include <IPAddress.h>

#define UDP_PORT 5828
// #define TARGET_LOOP_DELTA_MS 16 // about 60hz
#define TARGET_LOOP_DELTA_MS 100

// Uncomment to define hardcoded ip
#define SERVER_IP IPAddress(10, 136, 41, 71)

#define MAX_TRACKER_COUNT 2
