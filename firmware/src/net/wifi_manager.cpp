#include <ESP8266WiFi.h>
#include <WiFiUdp.h>
#include <cstring>

#include "config_manager.h"
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
        m_test_networks.clear();
        m_connected = false;

        // First try the auto reconnect
        m_attempt_timer.reset();
    }

    if (m_test_networks_populated) {
        if (m_attempt_timer.elapsed(WIFI_CONNECT_TIMEOUT_MS)) {
            LOG_WARN("Failed to connect to network, trying next");
            Serial.print("WifiConnectTimeout\n");
            try_connect_next_network();
        }
    } else {
        try_populate_test_networks();
    }

    return false;
}

void WifiManager::use_credentials(const char* ssid, const char* password) {
    if (strlen(ssid) > MAX_SSID_LENGTH || strlen(password) > MAX_PASSWORD_LENGTH) {
        return;
    }

    if (m_connected && WiFi.SSID().equals(ssid)) {
        return;
    }

    LOG_INFO("Trying to connect to network %s", ssid);
    Serial.print("WifiConnecting\n");
    WiFi.begin(ssid, password);
    m_attempt_timer.reset();
    m_has_manually_set_creds = true;
}

// Connects to the next test network in the list
void WifiManager::try_connect_next_network() {
    g_internal_led.blink(20);
    m_has_manually_set_creds = false;
    if (m_test_networks.empty()) {
        start_scan();
        return;
    }

    // Highest is at the end so go from backwards
    const bss_info* info = m_test_networks.back();
    const char* ssid = (const char*)info->ssid;
    LOG_INFO("Trying to connect to network %.32s", (const char*)info->ssid);

    const char* password = g_config_manager.wifi_password_get(ssid);
    WiFi.begin(ssid, password);
    m_test_networks.pop_back();
    m_attempt_timer.reset();
}

void WifiManager::try_populate_test_networks() {
    int8_t num_networks = WiFi.scanComplete();
    if (num_networks <= 0) {
        return;
    }

    m_test_networks.clear();

    for (int8_t i = 0; i < num_networks; i++) {
        // Get network information
        const bss_info* info = WiFi.getScanInfoByIndex(i);

        // Check if the test networks to see if it already exists (potential duplicate SSIDs)
        if (!check_test_network_exists(info)) {
            if (g_config_manager.wifi_entry_exists((const char*)info->ssid)) {
                m_test_networks.push_back(info);
            }
        }
    }

    // Sort by signal strength (highest will be at the end)
    for (size_t i = 0; i < m_test_networks.size(); i++) {
        for (size_t j = i + 1; j < m_test_networks.size(); j++) {
            // Swap them when they are greater
            const bss_info* info_a = m_test_networks[i];
            const bss_info* info_b = m_test_networks[j];
            if (info_a->rssi > info_b->rssi) {
                m_test_networks[i] = info_b;
                m_test_networks[j] = info_a;
            }
        }
    }

    if (m_test_networks.empty()) {
        LOG_WARN("No WiFi networks found that was saved in flash memory");
        // This makes it so it will rescan after WIFI_CONNECT_TIMEOUT_MS
        m_attempt_timer.reset();
    } else {
        try_connect_next_network();
    }

    m_test_networks_populated = true;
}

bool WifiManager::check_test_network_exists(const bss_info* info) {
    for (uint8_t i = 0; i < m_test_networks.size(); i++) {
        const char* test_ssid = (const char*)m_test_networks[i]->ssid;
        const char* ssid = (const char*)info->ssid;
        if (strncmp(ssid, test_ssid, MAX_SSID_LENGTH) == 0) {
            // Set the one with the higher RSSI
            if (info->rssi > m_test_networks[i]->rssi) {
                m_test_networks[i] = info;
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
    m_test_networks.clear();
}

void WifiManager::on_connect() {
    LOG_INFO(
        "Connected to WiFi %s with ip %s", WiFi.SSID().c_str(), WiFi.localIP().toString().c_str()
    );
    m_connected = true;
    Serial.print("WifiConnectOk\n");

    if (m_has_manually_set_creds) {
        struct station_config config;
        wifi_station_get_config(&config);
        g_config_manager.wifi_entry_save((const char*)config.ssid, (const char*)config.password);
    }
}
