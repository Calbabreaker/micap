#include <Wire.h>

#include "log.h"
#include "tracker_manager.h"
#include "trackers/tracker.h"
#include "trackers/tracker_bmi160.h"

bool i2c_device_connected(uint8_t address) {
    Wire.beginTransmission(address);
    return Wire.endTransmission() == 0;
}

Tracker* make_tracker(TrackerKind kind, uint8_t id, uint8_t address) {
    switch (kind) {
    case TrackerKind::BMI160:
        return new TrackerBMI160(id, address);
    }
}

void TrackerManager::register_tracker(TrackerKind kind, uint8_t address, bool required) {
    uint8_t id = m_next_tracker_id;
    if (id >= m_trackers.size()) {
        LOG_ERROR("Number of trackers exceeded MAX_TRACKER_COUNT, please increase in config.h");
        return;
    }

    LOG_INFO("Tracker %d found with address 0x%02x", id, address);
    Tracker* tracker = make_tracker(kind, id, address);

    if (i2c_device_connected(address)) {
        if (required) {
            LOG_ERROR("Required tracker %d with address 0x%02x was not found", id, address);
            tracker->status = TrackerStatus::Error;
        } else {
            LOG_WARN("Optional tracker %d with address 0x%02x was not found", id, address);
            tracker->status = TrackerStatus::Off;
        }
    }

    if (tracker->status == TrackerStatus::Ok) {
        tracker->setup();
        m_ok_tracker_count += 1;
    }

    m_trackers[id] = tracker;
    m_next_tracker_id++;
}

void TrackerManager::setup() {
    Wire.begin();

    m_ok_tracker_count = 0;
    register_tracker(TrackerKind::BMI160, 0x68, true);
    register_tracker(TrackerKind::BMI160, 0x69, false);
}

void TrackerManager::update() {
    // Do polling every 3000 ms
    if (millis() > m_last_status_poll_time + 3000) {
        poll_tracker_status();
    }

    m_ok_tracker_count = 0;
    for (Tracker* tracker : m_trackers) {
        if (tracker->status == TrackerStatus::Ok) {
            tracker->update();
            m_ok_tracker_count += 1;
        }
    }
}

void TrackerManager::poll_tracker_status() {
    LOG_INFO("Polling i2c bus for new trackers");
    for (Tracker* tracker : m_trackers) {
        // If the tracker isn't ok, try to see if it is connected and setup again
        if (tracker->status != TrackerStatus::Ok && i2c_device_connected(tracker->get_address())) {
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
