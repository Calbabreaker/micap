#pragma once

#include <WiFiUdp.h>
#include <vector>

#include "IPAddress.h"
#include "net/wifi_manager.h"
#include "trackers/tracker.h"

// A ping packet has two purposes, to ensure the device won't be timed out on
// the server, and for caculating latency between server and device
constexpr uint8_t PACKET_PING_PONG = 0x00;

// Packet for establishing a connection between the server and the device
// Will always be the first packet sent before any other packet
constexpr uint8_t PACKET_HANDSHAKE = 0x01;
constexpr uint8_t PACKET_TRACKER_STATUS = 0x02;
constexpr uint8_t PACKET_TRACKER_DATA = 0x03;
constexpr uint8_t PACKET_BATTERY_LEVEL = 0x04;

const IPAddress MULTICAST_IP = IPAddress(239, 255, 0, 123);

class ConnectionManager {
public:
    void setup();
    void update();

    void send_tracker_data();
    void send_tracker_status(const Tracker* tracker);
    void send_handshake();
    void send_pong(uint8_t id);
    void send_battery_level(float level);

    bool has_acked_tracker(Tracker* tracker);

    WifiManager& get_wifi() { return m_wifi; }
    bool is_connected() { return m_connected; }

private:
    void begin_packet(uint8_t packet_type);
    void write_packet_number();
    void write_str(const char* str);
    void end_packet();

    template <typename T>
    void write_value(T value) {
        m_udp.write((uint8_t*)&value, sizeof(T));
    }

    void receive_packets();
    void update_tracker_statuses();

private:
    WiFiUDP m_udp;
    bool m_connected = false;
    IPAddress m_server_ip = INADDR_NONE;
    WifiManager m_wifi;
    uint8_t m_buffer[64];

    uint32_t m_next_packet_number = 0;
    Timer m_packet_received_timer;
    Timer m_important_send_timer;
};
