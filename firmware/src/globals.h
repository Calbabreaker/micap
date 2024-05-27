#pragma once

#include "led_manager.h"
#include "net/connection_manager.h"
#include "trackers/tracker_manager.h"

// Global variables
extern ConnectionManager g_connection_manager;
extern LedManager g_internal_led;
extern TrackerManager g_tracker_manager;

#include <pins_arduino.h>

#ifdef LED_BUILTIN
#define INTERNAL_LED_PIN LED_BUILTIN
#define LED_ENABLED
#else
#define INTERNAL_LED_PIN 0
#endif
