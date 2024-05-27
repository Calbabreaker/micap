#pragma once

#include <cstdint>

class Vector3 {
public:
    Vector3() = default;
    Vector3(float x, float y, float z) : x(x), y(y), z(z){};

    inline uint8_t* as_bytes() { return (uint8_t*)this; }

    float x = 0.0f;
    float y = 0.0f;
    float z = 0.0f;
};

class Quaternion {
public:
    Quaternion() = default;
    Quaternion(float x, float y, float z, float w) : x(x), y(y), z(z), w(w){};

    inline uint8_t* as_bytes() { return (uint8_t*)this; }

    float x = 0.0f;
    float y = 0.0f;
    float z = 0.0f;
    float w = 1.0f;
};
