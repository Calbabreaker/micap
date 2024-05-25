#include <ESP8266WiFi.h>
#include <WiFiUdp.h>

#include "log.h"
#include "wifi_manager.h"

void WiFiManager::setup() {
    WiFi.hostname("Mycap tracker");
    WiFi.mode(WIFI_STA);
    WiFi.persistent(true);
    LOG_INFO("Loaded WiFi creds with SSID: %s", WiFi.SSID().c_str());
    WiFi.begin();
}

bool WiFiManager::monitor() {
    if (WiFi.isConnected()) {
        if (!m_connected) {
            LOG_INFO("Connected to WiFi on %s", WiFi.localIP().toString().c_str());
            m_connected = true;
            return true;
        }
    } else {
        if (m_connected) {
            LOG_WARN("Lost WiFi connection");
            m_connected = false;
        }
    }

    return false;
}
