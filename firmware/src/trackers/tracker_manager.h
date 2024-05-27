#pragma once

#include <array>

#include "config.h"
#include "trackers/tracker.h"

class TrackerManager {
public:
    ~TrackerManager();
    void register_tracker(TrackerKind kind, uint8_t address, bool required);
    void setup();
    void update();

    inline uint8_t get_ok_tracker_count() const { return m_ok_tracker_count; }
    inline const std::array<Tracker*, MAX_TRACKER_COUNT>& get_trackers() const {
        return m_trackers;
    }

private:
    void poll_tracker_status();

private:
    std::array<Tracker*, MAX_TRACKER_COUNT> m_trackers;
    uint8_t m_ok_tracker_count;
    uint8_t m_next_tracker_id = 0;
    uint64_t m_last_status_poll_time = 0;
};
