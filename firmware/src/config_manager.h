#pragma once

#include <vector>

struct WifiEntry {
    const char* ssid;
    const char* passphrase;
};

struct Config {
    std::vector<WifiEntry> wifi_entries;
};

class ConfigManager {
public:
    void save();

private:
    Config m_config;
};
