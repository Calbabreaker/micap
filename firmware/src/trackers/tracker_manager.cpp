#include <Wire.h>
#include <i2c_clearbus.h>

#include "defines.h"
#include "log.h"
#include "tracker_manager.h"
#include "trackers/tracker.h"
#include "trackers/tracker_bmi160.h"

bool test_i2c_connection(uint8_t address) {
    Wire.beginTransmission(address);
    return Wire.endTransmission() == 0;
}

Tracker* make_tracker(TrackerKind kind, uint8_t index, uint8_t address) {
    switch (kind) {
    case TrackerKind::BMI160:
        // Have to heap allocate to use polymorphism
        return new TrackerBMI160(index, address);
    default:
        LOG_ERROR("Unknown tracker kind %d", (uint8_t)kind);
        return nullptr;
    }
}

void TrackerManager::register_tracker(
    uint8_t index, TrackerKind kind, uint8_t address, bool required
) {
    if (index >= MAX_TRACKER_COUNT) {
        LOG_ERROR("Index is greater than MAX_TRACKER_COUNT, please increase in defines.h");
        return;
    }

    Tracker* tracker = make_tracker(kind, index, address);
    m_trackers[index] = tracker;

    if (test_i2c_connection(address)) {
        LOG_INFO("Tracker %d with address 0x%02x found", index, address);
        tracker->setup();
    } else {
        if (required) {
            LOG_ERROR("Required tracker %d with address 0x%02x not found", index, address);
            tracker->status = TrackerStatus::Error;
        } else {
            LOG_WARN("Optional tracker %d with address 0x%02x not found", index, address);
            tracker->status = TrackerStatus::Off;
        }
    }
}

void TrackerManager::setup() {
    // Make sure i2c bus doesn't get stuck
    I2C_ClearBus();
    Wire.begin();

    register_tracker(0, TrackerKind::BMI160, 0x68, true);
    register_tracker(1, TrackerKind::BMI160, 0x69, false);
}

// Not currently used
void TrackerManager::poll_tracker_status() {
    // Do polling every 5000 ms
    if (millis() < m_last_status_poll_time + 5000) {
        return;
    }

    LOG_TRACE("Polling i2c bus for new trackers");
    for (Tracker* tracker : m_trackers) {
        // If the tracker isn't ok, try to see if it is connected and setup again
        if (tracker->status != TrackerStatus::Ok && test_i2c_connection(tracker->get_address())) {
            tracker->status = TrackerStatus::Ok;
            tracker->setup();
            if (tracker->status == TrackerStatus::Ok) {
                LOG_INFO("New tracker detected with address 0x%02x", tracker->get_address());
            }
        }
    }

    m_last_status_poll_time = millis();
}

TrackerManager::~TrackerManager() {
    for (Tracker* tracker : m_trackers) {
        delete tracker;
    }
}
