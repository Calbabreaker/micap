#pragma once

#define STRINGIFY(x) #x
#define STRINGIFY_V(x) STRINGIFY(x)

#include "led_manager.h"
#include "net/connection_manager.h"
#include "trackers/tracker_manager.h"

// Global variables
extern ConnectionManager g_connection_manager;
extern LedManager g_internal_led;
extern TrackerManager g_tracker_manager;
