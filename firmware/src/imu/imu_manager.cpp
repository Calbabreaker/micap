#include "imu_manager.h"

void ImuManager::make_imu() {
    uint32_t id = m_imus.size();
    m_imus.push_back(Imu(id));
}
