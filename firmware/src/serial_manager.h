#pragma once

#include "net/wifi_manager.h"

class SerialManager {
public:
    void parse_incomming_command(WifiManager& wifi);

private:
    char m_buffer[128];
};
