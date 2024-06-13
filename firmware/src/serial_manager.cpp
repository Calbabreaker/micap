#include <Arduino.h>

#include "globals.h"
#include "log.h"
#include "serial_manager.h"

// Go to the start of the next string using strlen (which relies on the null byte)
const char* next_arg(const char* start, size_t* length_left) {
    size_t length = strlen(start) + 1; // Includes null byte
    if (length >= *length_left) {
        return nullptr;
    }

    *length_left -= length;
    return start + length;
}

// The commands are in the format of command name seperated by null byte for each argument
void SerialManager::parse_incomming_command() {
    if (!Serial.available()) {
        return;
    }

    size_t bytes_read = Serial.readBytesUntil('\n', m_buffer, sizeof(m_buffer));
    if (bytes_read == 0 || bytes_read >= sizeof(m_buffer)) {
        return;
    }

    // Set the end null byte
    m_buffer[bytes_read] = '\0';
    LOG_TRACE("Got command %s with %zu chars", m_buffer, bytes_read);

    if (strcmp(m_buffer, "Wifi") == 0) {
        const char* ssid_ptr = next_arg(m_buffer, &bytes_read);
        const char* password_ptr = next_arg(ssid_ptr, &bytes_read);
        if (!ssid_ptr) {
            return;
        }
        if (!password_ptr) {
            password_ptr = "";
        }

        g_connection_manager.get_wifi().use_credentials(ssid_ptr, password_ptr);
    } else if (strcmp(m_buffer, "FactoryReset") == 0) {
        g_config_manager.reset();
    }
}
