#pragma once

#include <stdexcept>
#include <vector>

namespace vegam {

/// Thrown by API entry points that are not implemented yet.
class NotImplementedError : public std::logic_error {
public:
    using std::logic_error::logic_error;
};

/// Per-joint kinematic limits. Each vector has one entry per joint.
struct Limits {
    std::vector<double> max_velocity;
    std::vector<double> max_acceleration;
    std::vector<double> max_jerk;
};

/// A time-parameterized trajectory sampled at discrete times.
struct Trajectory {
    std::vector<double> times;                   ///< Sample times [s], increasing.
    std::vector<std::vector<double>> positions;  ///< Joint positions per sample.

    /// Total duration [s]; 0 for an empty trajectory.
    double duration() const;
};

/// Compute a jerk-limited time-optimal parameterization of the path through
/// `waypoints` (one row per waypoint, one column per joint).
Trajectory parameterize(const std::vector<std::vector<double>>& waypoints,
                        const Limits& limits);

}  // namespace vegam
