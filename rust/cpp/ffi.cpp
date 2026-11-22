// C ABI over the vegam C++ API, used by the Rust crate (rust/src/ffi.rs).
//
// No C++ exception crosses this boundary: every entry point that can throw
// catches everything and reports it as a status code plus a message.

#include <cstddef>
#include <cstring>
#include <exception>
#include <new>
#include <stdexcept>
#include <vector>

#include "vegam/vegam.hpp"

namespace {

enum Status : int {
    VEGAM_OK = 0,
    VEGAM_NOT_IMPLEMENTED = 1,
    VEGAM_INVALID_ARGUMENT = 2,
    VEGAM_ERROR = 3,
};

std::vector<double> to_vector(const double* data, std::size_t len) {
    return len ? std::vector<double>(data, data + len) : std::vector<double>();
}

// Returns a heap copy of `msg` for the caller to release with
// vegam_string_free, or nullptr if allocation fails.
char* copy_message(const char* msg) noexcept {
    const std::size_t len = std::strlen(msg);
    char* out = new (std::nothrow) char[len + 1];
    if (out) std::memcpy(out, msg, len + 1);
    return out;
}

}  // namespace

extern "C" {

struct vegam_trajectory {
    vegam::Trajectory inner;
};

// Runs vegam::parameterize. `rows[i]` points to `row_lens[i]` doubles: the
// joint values of waypoint i. On VEGAM_OK, *out owns a trajectory (free it
// with vegam_trajectory_free). Otherwise *err may hold a message (free it
// with vegam_string_free).
int vegam_parameterize(const double* const* rows, const std::size_t* row_lens,
                       std::size_t n_rows, const double* max_velocity,
                       std::size_t n_velocity, const double* max_acceleration,
                       std::size_t n_acceleration, const double* max_jerk,
                       std::size_t n_jerk, vegam_trajectory** out,
                       char** err) noexcept {
    *out = nullptr;
    *err = nullptr;
    try {
        std::vector<std::vector<double>> waypoints;
        waypoints.reserve(n_rows);
        for (std::size_t i = 0; i < n_rows; ++i) {
            waypoints.push_back(to_vector(rows[i], row_lens[i]));
        }
        const vegam::Limits limits{to_vector(max_velocity, n_velocity),
                                   to_vector(max_acceleration, n_acceleration),
                                   to_vector(max_jerk, n_jerk)};
        *out = new vegam_trajectory{vegam::parameterize(waypoints, limits)};
        return VEGAM_OK;
    } catch (const vegam::NotImplementedError& e) {
        *err = copy_message(e.what());
        return VEGAM_NOT_IMPLEMENTED;
    } catch (const std::invalid_argument& e) {
        *err = copy_message(e.what());
        return VEGAM_INVALID_ARGUMENT;
    } catch (const std::exception& e) {
        *err = copy_message(e.what());
        return VEGAM_ERROR;
    } catch (...) {
        *err = copy_message("unknown C++ exception");
        return VEGAM_ERROR;
    }
}

void vegam_trajectory_free(vegam_trajectory* traj) noexcept { delete traj; }

std::size_t vegam_trajectory_times_len(const vegam_trajectory* traj) noexcept {
    return traj->inner.times.size();
}

const double* vegam_trajectory_times(const vegam_trajectory* traj) noexcept {
    return traj->inner.times.data();
}

std::size_t vegam_trajectory_num_positions(const vegam_trajectory* traj) noexcept {
    return traj->inner.positions.size();
}

std::size_t vegam_trajectory_position_len(const vegam_trajectory* traj,
                                          std::size_t i) noexcept {
    return traj->inner.positions[i].size();
}

const double* vegam_trajectory_position(const vegam_trajectory* traj,
                                        std::size_t i) noexcept {
    return traj->inner.positions[i].data();
}

void vegam_string_free(char* s) noexcept { delete[] s; }

}  // extern "C"
