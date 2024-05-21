#pragma once

#include <cstdint>

class SerialCommands {
public:
    void parse_command();

private:
    char m_buffer[128];
};
