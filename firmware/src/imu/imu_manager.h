#pragma once

#include "imu/imu.h"
#include <vector>

class ImuManager {
public:
    void make_imu();

private:
    std::vector<Imu> m_imus;
};
