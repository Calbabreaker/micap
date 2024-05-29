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
    int8_t scan_result = WiFi.scanComplete();
    if (scan_result <= 0) {
        return;
    }

    for (int8_t i = 0; i < scan_result; i++) {
        // Get network information
        const bss_info* info = WiFi.getScanInfoByIndex(i);
        if (g_config_manager.wifi_entry_exists((const char*)info->ssid)) {
            if (m_next_test_network_index >= MAX_WIFI_ENTRIES) {
                LOG_WARN("Test network count %d exceeded MAX_WIFI_ENTRIES", i);
                break;
            }

            m_test_network_infos[m_next_test_network_index] = info;
            m_next_test_network_index += 1;
        }
    }

    // Sort by signal strength (highest will be at the end)
    for (int i = 0; i < m_next_test_network_index; i++) {
        for (int j = i + 1; j < m_next_test_network_index; j++) {
            if (m_test_network_infos[i]->rssi > m_test_network_infos[j]->rssi) {
                const bss_info* tmp = m_test_network_infos[i];
                m_test_network_infos[i] = m_test_network_infos[j];
                m_test_network_infos[j] = tmp;
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

void WifiManager::start_scan() {
    LOG_INFO("Scanning for networks...");
    WiFi.scanDelete();
    WiFi.scanNetworks(true, true); // Scan in async mode
    m_test_networks_populated = false;
    m_next_test_network_index = 0;
}

void WifiManager::on_connect() {
    LOG_INFO("Connected to WiFi with ip %s", WiFi.localIP().toString().c_str());
    m_connected = true;

    if (m_has_manually_set_creds) {
        struct station_config config;
        wifi_station_get_config(&config);
        g_config_manager.wifi_entry_save((const char*)config.ssid, (const char*)config.password);
    }
}
