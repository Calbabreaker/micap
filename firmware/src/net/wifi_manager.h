#pragma once

class WiFiManager {
public:
    void setup();
    // Returns if true if just (re)connected to wifi
    bool monitor();

    inline bool is_connected() {
        return m_connected;
    }

private:
    void on_connect();

private:
    bool m_connected = false;
};
