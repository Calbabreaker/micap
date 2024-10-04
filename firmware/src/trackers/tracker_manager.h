#pragma once

#include <array>

#include "defines.h"
#include "trackers/tracker.h"

class TrackerManager {
public:
    ~TrackerManager();
    void register_tracker(Tracker* tracker, bool required);
    void setup();
    void poll_tracker_status();
    // Returns true if any tracker has new data
    bool update();

    inline const std::array<Tracker*, MAX_TRACKER_COUNT>& get_trackers() const {
        return m_trackers;
    }

private:
    std::array<Tracker*, MAX_TRACKER_COUNT> m_trackers;
    uint64_t m_last_status_poll_time = 0;
};
