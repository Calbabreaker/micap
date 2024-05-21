#include <ESP8266WiFi.h>
#include <WiFiUdp.h>

#include "consts.h"
#include "log.h"
#include "wifi_manager.h"

bool wifi_connected = false;

WiFiUDP udp;

void WiFiManager::setup() {
    WiFi.hostname("Mycap tracker");
    WiFi.mode(WIFI_STA);
}

void on_connect() {
    LOG("Connected to WiFi\n");
    wifi_connected = true;

    udp.begin(UDP_PORT);
    LOG("Listening at IP %s, UDP port %d\n", WiFi.localIP().toString().c_str(), UDP_PORT);
}

void WiFiManager::monitor() {
    if (WiFi.isConnected()) {
        if (!wifi_connected) {
            on_connect();
        }
        return;
    }

    if (wifi_connected) {
        LOG("Lost WiFi connection");
        wifi_connected = false;
    }
}
