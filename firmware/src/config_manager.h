#pragma once

#include <array>
#include <cstdint>

#define MAX_PASSWORD_LENGTH 64

class ConfigManager {
public:
    void setup();

    void save_wifi_creds(const char* ssid, const char* password);
    std::array<char, MAX_PASSWORD_LENGTH> get_wifi_password(const char* ssid);
    bool wifi_creds_exists(const char* ssid);

private:
    void set_ssid_path(const char* ssid);

private:
    char m_path_buf[64];
};
