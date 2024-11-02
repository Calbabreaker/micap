#pragma once

#include "log.h"
#include <cstdlib>

constexpr float EARTH_GRAVITY = 9.8f;
constexpr float EPSILON = 0.001f;

class Vector3 {
public:
    Vector3() = default;
    Vector3(float x, float y, float z) : x(x), y(y), z(z) {};

    Vector3 operator/(float scalar) const { return Vector3(x / scalar, y / scalar, z / scalar); }
    Vector3 operator*(float scalar) const { return Vector3(x * scalar, y * scalar, z * scalar); }
    Vector3 operator-(const Vector3 v) const { return Vector3(x - v.x, y - v.y, z - v.z); }

    bool nearly_equals(const Vector3 v) {
        return abs(x - v.x) < EPSILON && abs(y - v.y) < EPSILON && abs(z - v.z) < EPSILON;
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

    Quaternion operator*(const Quaternion q) const {
        return Quaternion(
            w * q.x + x * q.w + y * q.z - z * q.y, //
            w * q.y + y * q.w + z * q.x - x * q.z, //
            w * q.z + z * q.w + x * q.y - y * q.x, //
            w * q.w - x * q.x - y * q.y - z * q.z  //
        );
    }

    bool nearly_equals(const Quaternion q) {
        return abs(x - q.x) < EPSILON && abs(y - q.y) < EPSILON && abs(z - q.z) < EPSILON &&
               abs(w - q.w) < EPSILON;
    }

public:
    float x = 0.0f;
    float y = 0.0f;
    float z = 0.0f;
    float w = 1.0f;
};

class Timer {
public:
    // Returns true if the inner time has ellapsed elapsed_time
    inline bool elapsed(uint64_t elapsed_time) {
        uint64_t now = millis();
        return now > m_last_elapsed_time + elapsed_time;
    }

    inline void reset() {
        m_last_elapsed_time = millis(); //
    }

private:
    uint64_t m_last_elapsed_time = 0;
};
