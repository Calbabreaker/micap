#include "config_manager.h"
#include "log.h"
#include <cstdio>
#include <cstring>

#include <LittleFS.h>

#define WIFI_DIR "/wifi"

void ConfigManager::setup() {
    if (!LittleFS.begin()) {
        LOG_ERROR("Could not begin LittleFS, ConfigManager will not save");
        return;
    }

    FSInfo fs_info;
    LittleFS.info(fs_info);
    LOG_INFO("Used %zu so far", fs_info.usedBytes);
}

void ConfigManager::save_wifi_entry(
    const char* ssid, uint32_t last_server_ip, const char* password
) {
    snprintf(m_path_buf, sizeof(m_path_buf), WIFI_DIR "/%s", ssid);

    File file = LittleFS.open(m_path_buf, "w");
    file.write((uint8_t*)&last_server_ip, sizeof(uint32_t));

    if (password != nullptr && strlen(password) <= 64) {
        // Only write up to the length of the password
        file.write((uint8_t*)password, strlen(password) + 1);
    }

    file.close();
}

WifiEntry ConfigManager::get_wifi_entry(const char* ssid) {
    snprintf(m_path_buf, sizeof(m_path_buf), WIFI_DIR "/%s", ssid);

    WifiEntry entry;
    File file = LittleFS.open(m_path_buf, "r");
    file.read((uint8_t*)&entry, sizeof(WifiEntry));
    file.close();
    return entry;
}
