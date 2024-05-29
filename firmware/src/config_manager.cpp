#include "config_manager.h"
#include "log.h"
#include <Arduino.h>

#include <LittleFS.h>
#include <cstring>

#define WIFI_DIR "/wifi"

void ConfigManager::setup() {
    if (!LittleFS.begin()) {
        LOG_ERROR("Could not begin LittleFS, ConfigManager will not save");
        return;
    }

    FSInfo fs_info;
    LittleFS.info(fs_info);
    LOG_INFO("Used %zu bytes in LittleFS", fs_info.usedBytes);

    if (!LittleFS.exists(WIFI_DIR)) {
        LittleFS.mkdir(WIFI_DIR);
    }
}

void ConfigManager::save_wifi_creds(const char* ssid, const char* password) {
    set_ssid_path(ssid);

    LOG_INFO("Saving wifi creds");
    File file = LittleFS.open(m_path_buf, "rw");

    // Only write up to the length of the password (plus null byte) or the maxmium possible
    size_t bytes = min((int)strlen(password) + 1, MAX_PASSWORD_LENGTH);
    file.write((uint8_t*)password, bytes);
    file.close();
}

std::array<char, MAX_PASSWORD_LENGTH> ConfigManager::get_wifi_password(const char* ssid) {
    set_ssid_path(ssid);

    std::array<char, MAX_PASSWORD_LENGTH> password;
    File file = LittleFS.open(m_path_buf, "r");
    file.read((uint8_t*)&password, password.size());
    file.close();
    return password;
}

bool ConfigManager::wifi_creds_exists(const char* ssid) {
    set_ssid_path(ssid);
    return LittleFS.exists(m_path_buf);
}

void ConfigManager::set_ssid_path(const char* ssid) {
    memset(m_path_buf, 0, sizeof(m_path_buf)); // Makes sure the null byte is set
    snprintf(m_path_buf, sizeof(m_path_buf), WIFI_DIR "/%s", ssid);
}
