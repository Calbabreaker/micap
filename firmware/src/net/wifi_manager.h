#pragma once

#include "defines.h"
#include <ESP8266WiFi.h>
#include <array>
#include <cstdint>

#define WIFI_CONNECT_TIMEOUT_MS 12000

class WifiManager {
public:
    void setup();
    // Returns if true if just (re)connected to wifi
    bool monitor();

    void use_credentials(const char* ssid, const char* password);

    bool is_connected() { return m_connected; }

private:
    void try_connect_next_network();
    void try_populate_test_networks();
    void start_scan();
    void on_connect();
    bool check_existing_test_network(const bss_info* info);

private:
    bool m_connected = false;
    bool m_has_manually_set_creds = false;
    bool m_test_networks_populated = false;
    uint8_t m_test_network_count = 0;
    uint64_t m_last_attempt_time = 0;
    std::array<const bss_info*, MAX_WIFI_ENTRIES> m_test_network_infos;
};
