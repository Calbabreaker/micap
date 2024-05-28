#include "config_manager.h"
#include <EEPROM.h>
#include <cstring>

void ConfigManager::save() {
    // EEPROM.put();
}

void ConfigManager::set_wifi_entry(const char* ssid, const char* password) {
    for (WifiEntry& entry : m_config.wifi_entries) {
        // If the entry already exists, set the passphrase instead
        if (strcmp(entry.ssid, ssid) == 0) {
            strncpy(entry.password, password, sizeof(entry.password));
        }
    }
}
