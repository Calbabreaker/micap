#pragma once

#include <WiFiUdp.h>

#include "IPAddress.h"
#include "defines.h"
#include "net/wifi_manager.h"
#include "trackers/tracker.h"

// A ping packet has two purposes, to ensure packets can still be send round trip,
// and for caculating latency between server and device
constexpr uint8_t PACKET_PING_PONG = 0x00;

// Packet for establishing a connection between the server and the device
// Will always be the first packet sent before any other packet
constexpr uint8_t PACKET_HANDSHAKE = 0x01;
constexpr uint8_t PACKET_TRACKER_STATUS = 0x02;
constexpr uint8_t PACKET_TRACKER_DATA = 0x03;

const IPAddress MULTICAST_IP = IPAddress(239, 255, 0, 123);

class ConnectionManager {
public:
    void setup();
    void update();

    void send_tracker_data();
    void send_tracker_status(uint8_t tracker_id, TrackerStatus tracker_state);
    void send_handshake();
    void send_pong(uint8_t id);

    WifiManager& get_wifi() { return m_wifi; }
    bool is_connected() { return m_connected; }

private:
    void begin_packet(uint8_t packet_type);
    void write_packet_number();
    void write_str(const char* str);
    void end_packet();

    void receive_packets();
    void update_tracker_statuses();

private:
    WiFiUDP m_udp;
    bool m_connected = false;
    IPAddress m_server_ip = INADDR_NONE;
    WifiManager m_wifi;
    uint8_t m_buffer[64];

    std::array<TrackerStatus, MAX_TRACKER_COUNT> m_tracker_statuses_on_server;

    uint32_t m_next_packet_number = 0;
    uint64_t m_last_sent_handshake_time = 0;
    uint64_t m_last_received_time = 0;
    uint64_t m_last_tracker_status_sent_time = 0;
};
