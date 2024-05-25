#include "connection_manager.h"
#include "config.h"
#include "globals.h"
#include "log.h"

#include <ESP8266WiFi.h>

void ConnectionManager::setup() {
    m_wifi.setup();
}

void ConnectionManager::update() {
    bool just_reconnected = m_wifi.monitor();

    if (just_reconnected) {
        set_server_ip();
        LOG_INFO("Broadcasting to %s", m_server_ip.toString().c_str());
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
            g_internal_led.blink(25);
            send_handshake();
        }
    } else {
        // If we haven't got a packet from the server for 5000ms, we can assume we got disconnected
        if (now > m_last_received_time + 5000) {
            LOG_WARN("Timed out and disconnected from server");
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
    LOG_TRACE("Received %d bytes from %s", len, m_udp.remoteIP().toString().c_str());
    m_last_received_time = millis();

    switch (m_buffer[0]) {
    case PACKET_HANDSHAKE:
        // Check mycap header mark
        // MYCAP-SERVER indicates server response
        if (strcmp((const char*)m_buffer + 1, "MYCAP-SERVER") != 0) {
            break;
        }

        if (!m_connected) {
            LOG_INFO("Successfully handshaked with %s", m_udp.remoteIP().toString().c_str());
            m_connected = true;
            m_server_ip = m_udp.remoteIP();
        } else {
            // Ignore later handshake packets
            LOG_WARN("Received handshake while already connected");
        }
        break;
    case PACKET_HEARTBEAT:
        // Ping back heartbeat
        send_hearbeat();
        break;
    }
}

void ConnectionManager::send_handshake() {
    LOG_TRACE("Sending handshake packet...");

    begin_packet();
    m_udp.write(PACKET_HANDSHAKE);
    write_str("MYCAP-DEVICE"); // mark as mycap handshake
    uint8_t* mac = WiFi.macAddress(m_buffer);
    m_udp.write(mac, 6); // mac adresss as unique id
    end_packet();
    m_last_sent_handshake_time = millis();
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

void ConnectionManager::set_server_ip() {
#ifdef SERVER_IP
    // Use the hardcoded ip
    m_server_ip = SERVER_IP;
#else
    // Or use the broadcast ip
    m_server_ip = ~((uint32_t)WiFi.subnetMask()) | (uint32_t)WiFi.localIP();
#endif
}
