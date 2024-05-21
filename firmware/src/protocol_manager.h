#pragma once

#include "WiFiUdp.h"

const uint8_t PACKET_HEARTBEAT = 0x00;
const uint8_t PACKET_HANDSHAKE = 0x01;
const uint8_t PACKET_IMU = 0x03;
const uint8_t PACKET_ACK = 0xff;

class ProtocolManager {
public:
private:
    WiFiUDP m_udp;
};
