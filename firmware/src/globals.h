#pragma once

const double EARTH_GRAVITY = 9.8;

#include "config_manager.h"
#include "led_manager.h"
#include "net/connection_manager.h"
#include "trackers/tracker_manager.h"

// Global variables
extern ConnectionManager g_connection_manager;
extern LedManager g_internal_led;
extern TrackerManager g_tracker_manager;
extern ConfigManager g_config_manager;
