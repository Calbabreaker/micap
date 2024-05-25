#pragma once

#include <cstdint>

class Imu {
public:
    Imu(uint32_t id) : m_id(id) {}

private:
    uint32_t m_id;
};
