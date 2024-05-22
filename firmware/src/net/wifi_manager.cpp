#include <ESP8266WiFi.h>
#include <WiFiUdp.h>

#include "log.h"
#include "wifi_manager.h"

void WiFiManager::setup() {
    WiFi.hostname("Mycap tracker");
    WiFi.mode(WIFI_STA);
    WiFi.persistent(true);
    LOG("Loaded creds with SSID: %s\n", WiFi.SSID().c_str());
    WiFi.begin();
}

bool WiFiManager::monitor() {
    if (WiFi.isConnected()) {
        if (!m_connected) {
            LOG("Connected to WiFi\n");
            m_connected = true;
            return true;
        }
    } else {
        if (m_connected) {
            LOG("Lost WiFi connection\n");
            m_connected = false;
        }
    }

    return false;
}
