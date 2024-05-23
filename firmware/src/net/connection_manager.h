#pragma once

#include <WiFiUdp.h>

#include "math.h"
#include "net/wifi_manager.h"

const uint8_t PACKET_HEARTBEAT = 0x00;
const uint8_t PACKET_HANDSHAKE = 0x01;
const uint8_t PACKET_ACCELERATION = 0x02;

class ConnectionManager {
public:
    void setup();
    void update();

    void receive_packets();
    void check_ack_packet();

    void send_acceleration(Vector3 acceleration);
    void send_handshake();

    inline bool is_connected() {
        return m_connected;
    }

private:
    void begin_packet();
    void write_str(const char* str);
    void end_packet();

private:
    WiFiUDP m_udp;
    bool m_connected = false;
    // Start with the broadcast ip and then set the actual server ip for future (re)connections
    IPAddress m_server_ip = IPAddress(255, 255, 255, 255);
    WiFiManager m_wifi;
    uint8_t m_buffer[64];

    uint64_t m_last_sent_handshake = 0;
};
