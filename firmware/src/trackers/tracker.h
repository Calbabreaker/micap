#pragma once

#include "maths.h"

enum class TrackerKind : uint8_t {
    BMI160, // only bmi160 supported for now
};

enum class TrackerStatus : uint8_t {
    Ok,
    Error, // Shown as error on the UI
    Off,
};

class Tracker {
public:
    Tracker(TrackerKind kind, uint8_t index, uint8_t address)
        : m_kind(kind), m_index(index), m_address(address) {}
    virtual ~Tracker() {};

    virtual void setup() {};
    virtual void update() {};

    uint8_t get_index() const { return m_index; }
    uint8_t get_address() const { return m_address; }

    void set_new_data(const Vector3 vector, const Quaternion quat);

public:
    TrackerStatus status = TrackerStatus::Ok;
    TrackerStatus acked_status = TrackerStatus::Off; // The status that the server knows
    bool has_new_data = false;

    // Values to be sent to server on each update loop
    Vector3 acceleration;
    Quaternion orientation;

protected:
    TrackerKind m_kind;
    uint8_t m_index;
    uint8_t m_address;
};
