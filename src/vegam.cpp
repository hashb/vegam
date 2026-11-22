#include "vegam/vegam.hpp"

namespace vegam {

double Trajectory::duration() const {
    return times.empty() ? 0.0 : times.back() - times.front();
}

Trajectory parameterize(const std::vector<std::vector<double>>& /*waypoints*/,
                        const Limits& /*limits*/) {
    // TODO: implement jerk-limited TOPP.
    throw NotImplementedError("vegam::parameterize is not implemented yet");
}

}  // namespace vegam
