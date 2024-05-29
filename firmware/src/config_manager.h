#pragma once

#include "defines.h"
#include <array>

#define MAX_PASSWORD_LENGTH 64
#define MAX_SSID_LENGTH 32

struct WifiEntry {
    char ssid[MAX_PASSWORD_LENGTH];
    char password[MAX_PASSWORD_LENGTH];
};

struct WifiEntries {
    uint8_t next_index_free;
    std::array<WifiEntry, MAX_WIFI_ENTRIES> array;
};

class ConfigManager {
public:
    void setup();
    void reset();

    void wifi_entry_save(const char* ssid, const char* password);
    const char* wifi_password_get(const char* ssid);
    bool wifi_entry_exists(const char* ssid);

private:
    int find_wifi_entry_index(const char* ssid);

private:
    WifiEntries m_wifi_entries;
};
