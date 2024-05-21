#include <Arduino.h>

#include "ESP8266WiFi.h"
#include "serial_commands.h"

char* next_arg(char* start, size_t* length_left) {
    size_t length = strlen(start);
    if (length >= *length_left) {
        return 0;
    }

    *length_left -= length;
    return start + length;
}

// The commands are in the format of command name seperated by null byte for each argument
void SerialCommands::parse_command() {
    if (!Serial.available()) {
        return;
    }

    size_t bytes_read = Serial.readBytesUntil('\n', m_buffer, sizeof(m_buffer));
    if (bytes_read == 0 || bytes_read >= sizeof(m_buffer)) {
        return;
    }

    // Set the end null byte
    m_buffer[bytes_read] = '\0';

    // m_buffer will be the command name and m_arg_ptr will be the arg
    char* arg_ptr = next_arg(m_buffer, &bytes_read);
    if (!arg_ptr) {
        return;
    }

    if (strcmp(m_buffer, "WIFI") == 0) {
        char* password_ptr = next_arg(arg_ptr, &bytes_read);
        if (!password_ptr) {
            return;
        }

        WiFi.begin(arg_ptr, password_ptr);
    }
}
