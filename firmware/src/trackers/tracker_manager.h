#pragma once

#include <vector>

#include "trackers/tracker.h"

class TrackerManager {
public:
    ~TrackerManager();
    void register_tracker(TrackerKind kind, uint8_t address, bool required);
    void setup();
    void update();

private:
    std::vector<Tracker*> m_trackers;
};
