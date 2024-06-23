#include "connection_manager.h"
#include "defines.h"
#include "globals.h"
#include "log.h"
#include "trackers/tracker.h"

#include <ESP8266WiFi.h>

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
        // Send handshake every 2000 ms
        if (millis() > m_last_sent_handshake_time + 2000) {
            g_internal_led.blink(25);
            send_handshake();
        }
    } else {
        // Try and send tracker statuses every 2000 ms
        if (millis() > m_last_tracker_status_sent_time + 2000) {
            update_tracker_statuses();
        }

        // If we haven't got a packet from the server for 5000ms, we can assume we got disconnected
        if (millis() > m_last_received_time + 5000) {
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
        // MYCAP-SERVER indicates mycap server response
        if (strcmp((const char*)m_buffer + 1, "MYCAP-SERVER") != 0) {
            break;
        }

        if (!m_connected) {
            LOG_INFO("Successfully handshaked with %s", m_udp.remoteIP().toString().c_str());
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
    case PACKET_TRACKER_STATUS: {
        uint8_t id = m_buffer[1];
        if (id < m_tracker_statuses_on_server.size()) {
            m_tracker_statuses_on_server[id] = (TrackerStatus)m_buffer[2];
        }
        break;
    }
    case PACKET_HEARTBEAT:
        // Ping back heartbeat
        send_hearbeat();
        break;
    default:
        LOG_WARN("Received invalid packet id %d", m_buffer[0]);
        break;
    }
}

void ConnectionManager::update_tracker_statuses() {
    for (Tracker* tracker : g_tracker_manager.get_trackers()) {
        if (tracker->status != m_tracker_statuses_on_server[tracker->get_index()]) {
            send_tracker_status(tracker->get_index(), tracker->status);
        }
    }

    m_last_tracker_status_sent_time = millis();
}

void ConnectionManager::send_handshake() {

#ifdef SERVER_IP
    // Hardcoded server ip
    LOG_TRACE("Sending handshake hardcoded ip %s", SERVER_IP.toString().c_str());
    m_udp.beginPacket(SERVER_IP, UDP_PORT);
#else
    // Start using multicast to find the server by sending handshake packets
    // After handshake use unicast
    LOG_TRACE("Sending handshake packet to multicast ip %s", MULTICAST_IP.toString().c_str());
    m_udp.beginPacketMulticast(MULTICAST_IP, UDP_PORT, WiFi.localIP());
#endif
    m_udp.write(PACKET_HANDSHAKE);

    write_str("MYCAP-DEVICE"); // mark as mycap handshake

    uint8_t* mac = WiFi.macAddress(m_buffer);
    m_udp.write(mac, 6); // mac adresss as unique id
    end_packet();

    m_last_sent_handshake_time = millis();
}

void ConnectionManager::send_hearbeat() {
    begin_packet(PACKET_HEARTBEAT);
    end_packet();
}

// Packs orientation and acceleration data for each tracker in a single packet
void ConnectionManager::send_tracker_data() {
    begin_packet(PACKET_TRACKER_DATA);

    for (Tracker* tracker : g_tracker_manager.get_trackers()) {
        if (tracker->status == TrackerStatus::Ok) {
            tracker->update();
        }

        bool acked = m_tracker_statuses_on_server[tracker->get_index()] == tracker->status;
        if (tracker->has_new_data && acked) {
            m_udp.write(tracker->get_index());
            m_udp.write(tracker->acceleration.as_bytes(), sizeof(Vector3));
            m_udp.write(tracker->orientation.as_bytes(), sizeof(Quaternion));
            tracker->has_new_data = false;
        }
    }

    // 0xff where the tracker index would usually go signifies the end of the packet
    m_udp.write(0xff);
    end_packet();
}

void ConnectionManager::send_tracker_status(uint8_t tracker_id, TrackerStatus tracker_state) {
    begin_packet(PACKET_TRACKER_STATUS);
    m_udp.write(tracker_id);
    m_udp.write((uint8_t)tracker_state);
    end_packet();
}

void ConnectionManager::write_str(const char* str) {
    m_udp.write(str, strlen(str));
}

void ConnectionManager::begin_packet(uint8_t packet_type) {
    m_udp.beginPacket(m_server_ip, UDP_PORT);
    m_udp.write(packet_type);
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
