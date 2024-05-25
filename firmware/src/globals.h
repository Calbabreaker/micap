#pragma once

#include "led_manager.h"
#include "net/connection_manager.h"
#include "trackers/tracker_manager.h"

extern ConnectionManager g_connection_manager;
extern LedManager g_internal_led;
extern TrackerManager g_tracker_manager;

// Only one IMU supported for now
#define IMU_TYPE_BMI160 0
