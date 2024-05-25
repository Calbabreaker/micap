#pragma once

#include "led_manager.h"
#include "net/connection_manager.h"

extern ConnectionManager connection_manager;
extern LedManager internal_led;

// Only one IMU supported for now
#define IMU_TYPE_BMI160 0
