#pragma once

#include <cstdint>

class Vector3 {
public:
    Vector3() = default;
    Vector3(float x, float y, float z) : x(x), y(y), z(z){};

    inline uint8_t* as_bytes() {
        return (uint8_t*)this;
    }

    float x = 0;
    float y = 0;
    float z = 0;
};
