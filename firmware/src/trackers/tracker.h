#pragma once

#include "math.h"

enum class TrackerKind : uint8_t {
    BMI160, // only bmi160 supported for now
};

enum class TrackerStatus : uint8_t {
    Ok,
    Error, // Shown as error on the UI
    Off,   // Won't be shown on the UI
};

class Tracker {
public:
    Tracker(TrackerKind kind, uint8_t index, uint8_t address)
        : m_kind(kind), m_index(index), m_address(address) {}
    virtual ~Tracker(){};

    virtual void setup(){};
    virtual void update(){};

    inline uint8_t get_index() { return m_index; }
    inline uint8_t get_address() { return m_address; }

public:
    TrackerStatus status = TrackerStatus::Ok;
    // Values to be sent to server on each update loop
    Vector3 acceleration;
    Vector3 orientation;

protected:
    TrackerKind m_kind;
    uint8_t m_index;
    uint8_t m_address;
};
