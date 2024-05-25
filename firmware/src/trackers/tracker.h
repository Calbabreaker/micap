#pragma once

#include "math.h"

enum class TrackerKind : uint8_t {
    BMI160, // only bmi160 supported for now
    Bad,    // error setting up tracker
};

class Tracker {
public:
    Tracker(TrackerKind kind, uint8_t id, uint8_t address)
        : m_kind(kind), m_id(id), m_address(address) {}
    virtual ~Tracker(){};

    virtual void setup(){};
    virtual void update(){};

    inline bool is_working() {
        return m_working;
    }

    void send_data();

protected:
    TrackerKind m_kind;
    uint8_t m_id;
    uint8_t m_address;
    Vector3 m_accleration;
    bool m_working = false;
};
