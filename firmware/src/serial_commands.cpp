#include <Arduino.h>

#include "ESP8266WiFi.h"
#include "log.h"
#include "serial_commands.h"

// Go to the start of the next string using strlen (which relies on the null byte)
const char* next_arg(const char* start, size_t* length_left) {
    size_t length = strlen(start) + 1;
    if (length >= *length_left) {
        return 0;
    }

    *length_left -= length;
    return start + length;
}

// The commands are in the format of command name seperated by null byte for each argument
void SerialCommands::parse_incomming_command() {
    if (!Serial.available()) {
        return;
    }

    size_t bytes_read = Serial.readBytesUntil('\n', m_buffer, sizeof(m_buffer));
    if (bytes_read == 0 || bytes_read >= sizeof(m_buffer)) {
        return;
    }

    // Set the end null byte
    m_buffer[bytes_read] = '\0';
    LOG("Got command %s with %zu chars\n", m_buffer, bytes_read);

    const char* arg_ptr = next_arg(m_buffer, &bytes_read);
    if (!arg_ptr) {
        return;
    }

    if (strcmp(m_buffer, "WIFI") == 0) {
        const char* password_ptr = next_arg(arg_ptr, &bytes_read);
        // Set password to empty string if non provided
        if (!password_ptr) {
            password_ptr = "";
        }

        LOG("Connecting to %s with %s\n", arg_ptr, password_ptr);
        WiFi.begin(arg_ptr, password_ptr);
    }
}
