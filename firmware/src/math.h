#pragma once

#include "log.h"

class Vector3 {
public:
    Vector3() = default;
    Vector3(float x, float y, float z) : x(x), y(y), z(z){};

    uint8_t* as_bytes() { return (uint8_t*)this; }

    Vector3 operator/(float scalar) const { return Vector3(x / scalar, y / scalar, z / scalar); }
    Vector3 operator*(float scalar) const { return Vector3(x * scalar, y * scalar, z * scalar); }
    Vector3 operator-(const Vector3 vector) const {
        return Vector3(x - vector.x, y - vector.y, z - vector.z);
    }

public:
    float x = 0.0f;
    float y = 0.0f;
    float z = 0.0f;
};

class Quaternion {
public:
    Quaternion() = default;
    Quaternion(float x, float y, float z, float w) : x(x), y(y), z(z), w(w){};

    uint8_t* as_bytes() { return (uint8_t*)this; }

public:
    float x = 0.0f;
    float y = 0.0f;
    float z = 0.0f;
    float w = 1.0f;
};
