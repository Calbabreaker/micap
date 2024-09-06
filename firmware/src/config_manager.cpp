#include "config_manager.h"
#include "defines.h"
#include "log.h"
#include <Arduino.h>

#include <LittleFS.h>
#include <cstring>

#define WIFI_FILE "/wifi"

void ConfigManager::setup() {
    if (!LittleFS.begin()) {
        LOG_ERROR("Could not begin LittleFS, ConfigManager will not save");
        return;
    }

    FSInfo fs_info;
    LittleFS.info(fs_info);
    LOG_INFO("Used %zu bytes in LittleFS", fs_info.usedBytes);

    memset(&m_wifi_entries, 0, sizeof(WifiEntries));

    if (LittleFS.exists(WIFI_FILE)) {
        File file = LittleFS.open(WIFI_FILE, "r");
        file.read((uint8_t*)&m_wifi_entries, sizeof(WifiEntries));
        file.close();
    }
}

void ConfigManager::reset() {
    LittleFS.format();
}

void ConfigManager::wifi_entry_save(const char* ssid, const char* password) {
    LOG_INFO("Saving wifi creds");

    File file = LittleFS.open(WIFI_FILE, "w");

    // See if SSID already exists
    int i = find_wifi_entry_index(ssid);
    if (i < 0) {
        // Get a new index to set
        i = m_wifi_entries.next_index_free;
        strncpy(m_wifi_entries.array[i].ssid, ssid, MAX_SSID_LENGTH);

        m_wifi_entries.next_index_free += 1;
        // Wrap around once go over
        if (m_wifi_entries.next_index_free >= MAX_WIFI_ENTRIES) {
            m_wifi_entries.next_index_free = 0;
        }
    }

    strncpy(m_wifi_entries.array[i].password, password, MAX_PASSWORD_LENGTH);

    file.write((uint8_t*)&m_wifi_entries, sizeof(WifiEntries));
    file.close();
}

bool ConfigManager::wifi_entry_exists(const char* ssid) {
    return find_wifi_entry_index(ssid) >= 0;
}

const char* ConfigManager::wifi_password_get(const char* ssid) {
    int index = find_wifi_entry_index(ssid);
    if (index < 0) {
        return nullptr;
    } else {
        return m_wifi_entries.array[index].password;
    }
}

int ConfigManager::find_wifi_entry_index(const char* ssid) {
    for (int i = 0; i < MAX_WIFI_ENTRIES; i++) {
        const char* entry_ssid = m_wifi_entries.array[i].ssid;
        if (entry_ssid[0] != 0 && strncmp(entry_ssid, ssid, MAX_SSID_LENGTH) == 0) {
            return i;
        }
    }

    return -1;
}
