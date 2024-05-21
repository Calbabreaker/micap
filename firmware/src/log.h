#pragma once

#define ENABLE_LOG 1

#if ENABLE_LOG == 1
#define LOG(...) Serial.printf(__VA_ARGS__)
// for libraries
#define DEBUG
#else
#define LOG(...)
#endif
