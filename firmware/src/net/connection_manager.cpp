#include "connection_manager.h"
#include "defines.h"
#include "globals.h"
#include "log.h"
#include "trackers/tracker.h"

#include <ESP8266WiFi.h>
#include <cstdint>

void ConnectionManager::setup() {
    m_wifi.setup();
}

void ConnectionManager::update() {
    bool just_connected = m_wifi.monitor();

    if (just_connected) {
        m_udp.begin(UDP_PORT);
    }

    if (!m_wifi.is_connected()) {
        m_connected = false;
        return;
    }

    if (!m_connected) {
        // Send handshake every interval
        if (m_important_send_timer.elapsed(CONNECTION_RESEND_INTERVAL_MS)) {
            g_internal_led.blink(25);
            send_handshake();
            m_important_send_timer.reset();
        }
    } else {
        // Try and send tracker statuses every interval
        if (m_important_send_timer.elapsed(CONNECTION_RESEND_INTERVAL_MS)) {
            update_tracker_statuses();
            m_important_send_timer.reset();
        }

        // If we haven't got a packet from the server for some time, we can assume we
        // got disconnected
        if (m_packet_received_timer.elapsed(CONNECTION_TIMEOUT_MS)) {
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
    m_packet_received_timer.reset();

    switch (m_buffer[0]) {
    case PACKET_HANDSHAKE: {
        if (!m_connected) {
            // MCSVR indicates micap server response
            if (strcmp((const char*)m_buffer + 1, "MCSVR") != 0) {
                break;
            }

            LOG_INFO("Successfully handshaked with %s", m_udp.remoteIP().toString().c_str());
            Serial.print("Connected\n");
            m_connected = true;
            m_server_ip = m_udp.remoteIP();
            m_next_packet_number = 1; // Use 1 since handshake would use packet number 0

            // Set the tracker statuses to off so they can be resent
            std::fill(
                m_tracker_statuses_on_server.begin(), m_tracker_statuses_on_server.end(),
                TrackerStatus::Off
            );
        } else {
            // Ignore later handshake packets
            LOG_WARN("Received handshake while already connected");
        }
        break;
    }
    case PACKET_TRACKER_STATUS: {
        uint8_t id = m_buffer[1];
        if (id < m_tracker_statuses_on_server.size()) {
            m_tracker_statuses_on_server[id] = (TrackerStatus)m_buffer[2];
        }
        break;
    }
    case PACKET_PING_PONG:
        // Pong back ping
        send_pong(m_buffer[1]);
        break;
    default:
        LOG_WARN("Received invalid packet id %d", m_buffer[0]);
        break;
    }
}

void ConnectionManager::update_tracker_statuses() {
    for (Tracker* tracker : g_tracker_manager.get_trackers()) {
        if (tracker->status != m_tracker_statuses_on_server[tracker->get_index()]) {
            send_tracker_status(tracker);
        }
    }
}

void ConnectionManager::send_handshake() {
#ifdef SERVER_IP
    // Hardcoded server ip
    LOG_TRACE("Sending handshake to hardcoded ip %s", SERVER_IP.toString().c_str());
    m_server_ip = SERVER_IP;
    begin_packet(PACKET_HANDSHAKE);
#else
    // Start using multicast to find the server by sending handshake packets
    // After handshake use unicast
    LOG_TRACE("Sending handshake packet to multicast ip %s", MULTICAST_IP.toString().c_str());
    m_udp.beginPacketMulticast(MULTICAST_IP, UDP_PORT, WiFi.localIP());
    m_udp.write(PACKET_HANDSHAKE);
#endif

    write_value<uint32_t>(0); // Write packet number as 0
    write_str("MCDEV");       // mark as micap handshake

    // Send mac adresss for unique id
    uint8_t* mac = WiFi.macAddress(m_buffer);
    m_udp.write(mac, 6);
    end_packet();
}

void ConnectionManager::send_pong(uint8_t id) {
    g_internal_led.blink(20);
    begin_packet(PACKET_PING_PONG);
    write_value<uint32_t>(0);
    m_udp.write(id);
    end_packet();
}
void ConnectionManager::send_battery_level(float level) {
    begin_packet(PACKET_BATTERY_LEVEL);
    write_packet_number();
    write_value<float>(level);
    end_packet();
}

// Packs orientation and acceleration data for each tracker in a single packet
void ConnectionManager::send_tracker_data() {
    begin_packet(PACKET_TRACKER_DATA);
    write_packet_number();

    for (Tracker* tracker : g_tracker_manager.get_trackers()) {
        if (tracker->has_new_data) {
            m_udp.write(tracker->get_index());
            m_udp.write(tracker->orientation.as_bytes(), sizeof(Quaternion));
            m_udp.write(tracker->acceleration.as_bytes(), sizeof(Vector3));
            tracker->has_new_data = false;
        }
    }

    // 0xff where the tracker index would usually go signifies the end of the
    // packet
    m_udp.write(0xff);
    end_packet();
}

void ConnectionManager::send_tracker_status(Tracker* tracker) {
    begin_packet(PACKET_TRACKER_STATUS);
    write_packet_number();
    m_udp.write(tracker->get_index());
    m_udp.write((uint8_t)tracker->status);
    end_packet();
}

bool ConnectionManager::has_acked_tracker(Tracker* tracker) {
    return m_tracker_statuses_on_server[tracker->get_index()] == tracker->status;
}

void ConnectionManager::write_str(const char* str) {
    m_udp.write(str, strlen(str));
}

void ConnectionManager::begin_packet(uint8_t packet_type) {
    m_udp.beginPacket(m_server_ip, UDP_PORT);
    m_udp.write(packet_type);
}

void ConnectionManager::write_packet_number() {
    m_udp.write((uint8_t*)&m_next_packet_number, sizeof(uint32_t));
    m_next_packet_number += 1;
}

void ConnectionManager::end_packet() {
    m_udp.endPacket();
    // If about to overflow, restart the connection to reset the packet number
    if (m_next_packet_number == UINT32_MAX) {
        m_connected = false;
    }
}
