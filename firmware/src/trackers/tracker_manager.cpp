#include <Wire.h>

#include "log.h"
#include "tracker_manager.h"
#include "trackers/tracker_bmi160.h"

bool i2c_device_connected(uint8_t address) {
    Wire.beginTransmission(address);
    return Wire.endTransmission() == 0;
}

Tracker* make_tracker(TrackerKind kind, uint8_t id, uint8_t address, bool required) {
    switch (kind) {
    case TrackerKind::BMI160:
        return new TrackerBMI160(id, address);
    default:
        LOG_WARN("Tried to create unknown tracker type");
        return new Tracker(TrackerKind::Bad, id, address);
    }
}

void TrackerManager::register_tracker(TrackerKind kind, uint8_t address, bool required) {
    uint8_t id = m_trackers.size();

    if (i2c_device_connected(address)) {
        LOG_INFO("Tracker %d found with address 0x%02x", id, address);
        Tracker* tracker = make_tracker(kind, id, address, required);
        tracker->setup();
        m_trackers.push_back(tracker);
    } else {
        if (required) {
            LOG_ERROR("Required tracker %d with address 0x%02x was not found", id, address);
            m_trackers.push_back(new Tracker(TrackerKind::Bad, id, address));
        } else {
            LOG_WARN("Optional tracker %d with address 0x%02x was not found", id, address);
        }
    }
}

void TrackerManager::setup() {
    Wire.begin();

    register_tracker(TrackerKind::BMI160, 0x68, true);
    register_tracker(TrackerKind::BMI160, 0x69, false);
}

void TrackerManager::update() {
    for (Tracker* tracker : m_trackers) {
        if (tracker->is_working()) {
            tracker->update();
            tracker->send_data();
        }
    }
}

TrackerManager::~TrackerManager() {
    for (Tracker* tracker : m_trackers) {
        delete tracker;
    }
}
