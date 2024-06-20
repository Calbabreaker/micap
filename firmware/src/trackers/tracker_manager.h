#pragma once

#include <array>

#include "defines.h"
#include "trackers/tracker.h"

class TrackerManager {
public:
    ~TrackerManager();
    void register_tracker(TrackerKind kind, uint8_t address, bool required);
    void setup();
    void update();
    void poll_tracker_status();

    inline const std::array<Tracker*, MAX_TRACKER_COUNT>& get_trackers() const {
        return m_trackers;
    }

private:
    std::array<Tracker*, MAX_TRACKER_COUNT> m_trackers;
    uint8_t m_tracker_count = 0;
    uint64_t m_last_status_poll_time = 0;
};
