#pragma once

#include "defines.h"
#include <Arduino.h>

// LOG_* is only for debugging purposes and will be removed in production builds
#define ENABLE_LOG 1
#define ENABLE_FPS_LOG 0

#if ENABLE_LOG == 1
    #define LOG_INFO(msg, ...) Serial.printf("[info] " msg "\n", ##__VA_ARGS__)
    #define LOG_ERROR(msg, ...) Serial.printf("[error] " msg "\n", ##__VA_ARGS__)
    #define LOG_WARN(msg, ...) Serial.printf("[warn] " msg "\n", ##__VA_ARGS__)
    #define LOG_TRACE(msg, ...) Serial.printf("[trace] " msg "\n", ##__VA_ARGS__)
#else
    #define LOG_INFO(msg, ...)
    #define LOG_ERROR(msg, ...)
    #define LOG_WARN(msg, ...)
    #define LOG_TRACE(msg, ...)
#endif

class Timer {
public:
    inline bool elapsed(uint64_t activation_interval) {
        uint64_t now = millis();
        return now > m_last_elapsed_time + activation_interval;
    }

    inline void reset() {
        m_last_elapsed_time = millis(); //
    }

private:
    uint64_t m_last_elapsed_time = 0;
};
