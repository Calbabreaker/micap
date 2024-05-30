#include <ESP8266WiFi.h>
#include <WiFiUdp.h>

#include "core_esp8266_features.h"
#include "defines.h"
#include "globals.h"
#include "log.h"
#include "wifi_manager.h"

void WifiManager::setup() {
    WiFi.mode(WIFI_STA);
    WiFi.persistent(false);
    WiFi.setSleepMode(WIFI_MODEM_SLEEP);
    WiFi.setAutoConnect(false);
    WiFi.setAutoReconnect(true);
    start_scan();
}

bool WifiManager::monitor() {
    if (WiFi.isConnected()) {
        // Just connected
        if (!m_connected) {
            on_connect();
            return true;
        }

        return false;
    }

    // Just disconnected
    if (m_connected) {
        LOG_INFO("Disconnected from WiFi, reconnecting...");
        m_next_test_network_index = 0;
        m_connected = false;

        // First try the auto reconnect
        m_last_attempt_time = millis();
    }

    if (m_test_networks_populated) {
        if (millis() > m_last_attempt_time + WIFI_CONNECT_TIMEOUT_MS) {
            LOG_ERROR("Failed to connect to network, trying next");
            try_connect_next_network();
        }
    } else {
        try_populate_test_networks();
    }

    return false;
}

void WifiManager::use_credentials(const char* ssid, const char* password) {
    if (m_connected && WiFi.SSID() == ssid) {
        return;
    }

    LOG_INFO("Trying to connect to network %s", ssid);
    WiFi.begin(ssid, password);
    m_last_attempt_time = millis();
    m_has_manually_set_creds = true;
}

// Connects to the next test network in the list
void WifiManager::try_connect_next_network() {
    m_has_manually_set_creds = false;
    if (m_next_test_network_index == 0) {
        start_scan();
        return;
    }

    // Highest is at the end so go from backwards
    const bss_info* info = m_test_network_infos[m_next_test_network_index - 1];
    const char* ssid = (const char*)info->ssid;
    LOG_INFO("Trying to connect to network %.32s", (const char*)info->ssid);

    auto password = g_config_manager.wifi_password_get(ssid);
    WiFi.begin(ssid, password);
    m_next_test_network_index -= 1;
    m_last_attempt_time = millis();
}

void WifiManager::try_populate_test_networks() {
    int8_t num_networks = WiFi.scanComplete();
    if (num_networks <= 0) {
        return;
    }

    m_next_test_network_index = 0;

    for (int8_t i = 0; i < num_networks; i++) {
        // Get network information
        const bss_info* info = WiFi.getScanInfoByIndex(i);

        if (check_existing_test_network(info)) {
            continue;
        }

        if (g_config_manager.wifi_entry_exists((const char*)info->ssid)) {
            if (m_next_test_network_index >= MAX_WIFI_ENTRIES) {
                LOG_WARN(
                    "Test network count %d exceeded MAX_WIFI_ENTRIES", m_next_test_network_index
                );
                break;
            }

            m_test_network_infos[m_next_test_network_index] = info;
            m_next_test_network_index += 1;
        }
    }

    // Sort by signal strength (highest will be at the end)
    for (int i = 0; i < m_next_test_network_index; i++) {
        for (int j = i + 1; j < m_next_test_network_index; j++) {
            // Swap them when they are greater
            const bss_info* info_a = m_test_network_infos[i];
            const bss_info* info_b = m_test_network_infos[j];
            if (info_a->rssi > info_b->rssi) {
                m_test_network_infos[i] = info_b;
                m_test_network_infos[j] = info_a;
            }
        }
    }

    if (m_next_test_network_index == 0) {
        LOG_WARN("No WiFi networks found that was saved in flash memory");
        // This makes it so it will rescan after WIFI_CONNECT_TIMEOUT_MS
        m_last_attempt_time = millis();
    } else {
        try_connect_next_network();
    }

    m_test_networks_populated = true;
}

bool WifiManager::check_existing_test_network(const bss_info* info) {
    // Go through the test networks to see if it already exists (potential duplicate SSIDs)
    for (int8_t i = 0; i < m_next_test_network_index; i++) {
        const char* test_ssid = (const char*)m_test_network_infos[i]->ssid;
        const char* current_ssid = (const char*)info->ssid;
        if (strncmp(current_ssid, test_ssid, MAX_SSID_LENGTH) == 0) {
            // Set the one with the higher RSSI
            if (info->rssi > m_test_network_infos[i]->rssi) {
                m_test_network_infos[i] = info;
                return true;
            }
        }
    }

    return false;
}

void WifiManager::start_scan() {
    LOG_INFO("Scanning for networks...");
    WiFi.scanDelete();
    WiFi.scanNetworks(true, true); // Scan in async mode
    m_test_networks_populated = false;
    m_next_test_network_index = 0;
}

void WifiManager::on_connect() {
    LOG_INFO(
        "Connected to WiFi %s with ip %s", WiFi.SSID().c_str(), WiFi.localIP().toString().c_str()
    );
    m_connected = true;

    if (m_has_manually_set_creds) {
        struct station_config config;
        wifi_station_get_config(&config);
        g_config_manager.wifi_entry_save((const char*)config.ssid, (const char*)config.password);
    }
}
