#include "connection_manager.h"
#include "consts.h"
#include "globals.h"
#include "log.h"

#include <ESP8266WiFi.h>

void ConnectionManager::setup() {
    m_wifi.setup();
}

void ConnectionManager::update() {
    bool just_reconnected = m_wifi.monitor();

    if (just_reconnected) {
        m_udp.begin(UDP_PORT);
    }

    if (!m_wifi.is_connected()) {
        m_connected = false;
        return;
    }

    uint64_t now = millis();
    if (!m_connected) {
        // Send handshake every 2000 ms
        if (now > m_last_sent_handshake_time + 2000) {
            internal_led.blink(25);
            m_last_sent_handshake_time = now;
            send_handshake();
        }
    } else {
        // If we haven't got a packet from the server for 5000ms, we can assume we got disconnected
        if (now > m_last_received_time + 5000) {
            LOG("Timed out and disconnected from server\n");
            m_connected = false;
        }
    }

    receive_packets();
}

void ConnectionManager::receive_packets() {
    int packet_size = m_udp.parsePacket();
    if (packet_size == 0) {
        return;
    }

    int len = m_udp.read(m_buffer, sizeof(m_buffer));
    LOG("Received %d bytes from %s\n", len, m_udp.remoteIP().toString().c_str());
    m_last_received_time = millis();

    switch (m_buffer[0]) {
    case PACKET_HANDSHAKE:
        // Check mycap header mark
        // MYCAP-SERVER indicates server response
        if (strcmp((const char*)m_buffer + 1, "MYCAP-SERVER") != 0) {
            break;
        }

        if (!m_connected) {
            LOG("Successfully handshaked with %s\n", m_udp.remoteIP().toString().c_str());
            m_connected = true;
            m_server_ip = m_udp.remoteIP();
        } else {
            // Ignore later handshake packets
            LOG("Received handshake ack while already connected\n");
        }
        break;
    case PACKET_HEARTBEAT:
        // Ping back heartbeat
        send_hearbeat();
        break;
    }
}

void ConnectionManager::send_handshake() {
    LOG("Sending handshake packet...\n");

    begin_packet();
    m_udp.write(PACKET_HANDSHAKE);
    write_str("MYCAP-DEVICE"); // mark as mycap handshake
    uint8_t* mac = WiFi.macAddress(m_buffer);
    m_udp.write(mac, 6); // mac adresss as unique id
    end_packet();
}

void ConnectionManager::send_hearbeat() {
    begin_packet();
    m_udp.write(PACKET_HEARTBEAT);
    end_packet();
}

void ConnectionManager::send_acceleration(Vector3 acceleration) {
    begin_packet();
    m_udp.write(PACKET_ACCELERATION);
    m_udp.write(acceleration.as_bytes(), sizeof(float) * 3);
    end_packet();
}

void ConnectionManager::write_str(const char* str) {
    m_udp.write(str, strlen(str));
}

void ConnectionManager::begin_packet() {
    m_udp.beginPacket(m_server_ip, UDP_PORT);
}

void ConnectionManager::end_packet() {
    m_udp.endPacket();
}
