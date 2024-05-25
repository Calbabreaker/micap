#include "tracker.h"
#include "globals.h"

void Tracker::send_data() {
    g_connection_manager.send_acceleration(m_accleration);
}
