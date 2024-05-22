#include "connection_manager.h"
#include "consts.h"
#include "log.h"

#include <ESP8266WiFi.h>
#include <cstring>

void ConnectionManager::setup() {
    m_wifi.setup();
}

void ConnectionManager::update() {
    bool just_reconnected = m_wifi.monitor();

    if (just_reconnected) {
        m_udp.begin(UDP_PORT);
    }

    if (!m_connected) {
        // Send handshake every 1000 ms
        uint64_t now = millis();
        if (m_last_sent_handshake > now + 1000) {
            m_last_sent_handshake = now;
            send_handshake();
        }
    }

    receive_packets();
}

void ConnectionManager::receive_packets() {
    int packet_size = m_udp.parsePacket();
    if (packet_size != 0) {
        return;
    }

    int len = m_udp.read(m_buffer, sizeof(m_buffer));

    LOG("Received %d from %s", len, m_udp.remoteIP().toString().c_str());

    switch (m_buffer[0]) {
    case PACKET_ACK:
        check_ack_packet();
        break;
    }
}

void ConnectionManager::check_ack_packet() {
    switch (m_buffer[1]) {
    case PACKET_HANDSHAKE:
        // Have header stuff to ensure random data is not being sent
        if (!strcmp((const char*)m_buffer + 2, "MYCAP")) {
            break;
        }

        if (!m_connected) {
            LOG("Successfully handshaked with", m_udp.remoteIP().toString().c_str());
            m_connected = true;
            m_server_ip = m_udp.remoteIP();
        } else {
            // Ignore later handshake packets
            LOG("Received handshake ack while already connected");
        }
        break;
    }
}

void ConnectionManager::send_handshake() {
    uint8_t* mac = WiFi.macAddress(m_buffer);

    LOG("Sending handshake packet...");

    begin_packet();
    m_udp.write(PACKET_HANDSHAKE);
    m_udp.write("MYCAP", 5); // have header stuff to ensure random data is not being sent
    m_udp.write(mac, 6);     // mac adresss as unique id
    end_packet();
}

void ConnectionManager::send_acceleration(Vector3 acceleration) {
    // udp.beginPacket("192.168.20.23", UDP_PORT);
    begin_packet();
    m_udp.write(PACKET_ACCELERATION);
    m_udp.write(acceleration.as_bytes(), sizeof(float) * 3);
    end_packet();
}

void ConnectionManager::begin_packet() {
    m_udp.beginPacket(m_server_ip, UDP_PORT);
}

void ConnectionManager::end_packet() {
    m_udp.endPacket();
}
