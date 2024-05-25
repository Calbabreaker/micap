#pragma once

#include <bmi160.h>

#include "tracker.h"

class TrackerBMI160 : public Tracker {
public:
    TrackerBMI160(uint8_t id, uint8_t address) : Tracker(TrackerKind::BMI160, id, address) {}

    void setup() override final;
    void update() override final;

private:
    bmi160_dev m_device;
};
