#pragma once

class SerialCommands {
public:
    void parse_incomming_command();

private:
    char m_buffer[128];
};
