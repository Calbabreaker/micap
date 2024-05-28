#pragma once

#include <array>
#include <cstdint>

#include "defines.h"

struct WifiEntry {
    char ssid[32];
    char password[64];
};

struct Config {
    std::array<WifiEntry, MAX_WIFI_ENTRIES> wifi_entries;
    uint8_t current_wifi_entry;
};

class ConfigManager {
public:
    // Set the password for the SSID or adds a new entry if it doesn't exist
    void set_wifi_entry(const char* ssid, const char* passphrase);
    WifiEntry& get_wifi_entry();

private:
    Config m_config;
};
