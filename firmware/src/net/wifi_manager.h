#pragma once

#include <ESP8266WiFi.h>
#include <vector>

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
    bool check_test_network_exists(const bss_info* info);

private:
    bool m_connected = false;
    bool m_has_manually_set_creds = false;
    bool m_test_networks_populated = false;
    uint64_t m_last_attempt_time = 0;
    std::vector<const bss_info*> m_test_networks;
};
