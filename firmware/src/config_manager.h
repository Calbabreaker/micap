#pragma once

#include <array>
#include <cstdint>

struct WifiEntry {
    uint32_t last_server_ip;
    char password[65]; // Can only be a maximum of 64 characters (plus null byte)
};

class ConfigManager {
public:
    void setup();

    void save_wifi_entry(const char* ssid, uint32_t last_sever_ip, const char* password);
    WifiEntry get_wifi_entry(const char* ssid);

private:
    char m_path_buf[64];
};
