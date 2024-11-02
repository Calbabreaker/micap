#include "tracker.h"

void Tracker::set_new_data(const Vector3 vector, const Quaternion quat) {
    Quaternion rotation = Quaternion(0.0, 0.0, 0.7071067811865476, -0.7071067811865475);
    Quaternion quat2 = quat * rotation;
    if (this->acceleration.nearly_equals(vector) && this->orientation.nearly_equals(quat2)) {
        return;
    }

    this->acceleration = vector;
    this->orientation = quat2;
    this->has_new_data = true;
}
