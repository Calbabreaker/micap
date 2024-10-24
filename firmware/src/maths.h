#pragma once

#include "log.h"

class Vector3 {
public:
    Vector3() = default;
    Vector3(float x, float y, float z) : x(x), y(y), z(z) {};

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

#define LOG_VECTOR(v) LOG_TRACE("[%f, %f, %f]", v.x, v.y, v.z)

class Quaternion {
public:
    Quaternion() = default;
    Quaternion(float x, float y, float z, float w) : x(x), y(y), z(z), w(w) {};

public:
    float x = 0.0f;
    float y = 0.0f;
    float z = 0.0f;
    float w = 1.0f;
};
