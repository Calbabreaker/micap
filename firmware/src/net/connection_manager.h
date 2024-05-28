#pragma once

#include <WiFiUdp.h>

#include "defines.h"
#include "net/wifi_manager.h"
#include "trackers/tracker.h"

const uint8_t PACKET_HEARTBEAT = 0x00;
const uint8_t PACKET_HANDSHAKE = 0x01;
const uint8_t PACKET_TRACKER_STATUS = 0x02;
const uint8_t PACKET_TRACKER_DATA = 0x03;

class ConnectionManager {
public:
    void setup();
    void update();

    void send_tracker_data();
    void send_tracker_status(uint8_t tracker_id, TrackerStatus tracker_state);
    void send_handshake();
    void send_hearbeat();

    inline bool is_connected() { return m_connected; }

private:
    void begin_packet();
    void write_str(const char* str);
    void end_packet();

    void receive_packets();
    void update_tracker_statuses();
    bool should_send_tracker_data(Tracker* tracker);

    void set_server_ip();

private:
    WiFiUDP m_udp;
    bool m_connected = false;
    IPAddress m_server_ip;
    WiFiManager m_wifi;
    uint8_t m_buffer[64];

    std::array<TrackerStatus, MAX_TRACKER_COUNT> m_tracker_statuses_on_server;

    uint64_t m_last_sent_handshake_time = 0;
    uint64_t m_last_received_time = 0;
    uint64_t m_last_tracker_status_sent_time = 0;
};
