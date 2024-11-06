#include "tracker.h"

void Tracker::set_new_data(const Vector3 vector, const Quaternion quat) {
    if (this->acceleration.nearly_equals(vector) && this->orientation.nearly_equals(quat)) {
        return;
    }

    this->acceleration = vector;
    this->orientation = quat;
    this->has_new_data = true;
}
