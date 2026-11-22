#include <nanobind/nanobind.h>
#include <nanobind/stl/vector.h>

#include <new>
#include <utility>

#include "vegam/vegam.hpp"

namespace nb = nanobind;
using namespace nb::literals;

NB_MODULE(_core, m) {
    m.doc() = "vegam native extension";

    nb::register_exception_translator([](const std::exception_ptr& p, void*) {
        try {
            std::rethrow_exception(p);
        } catch (const vegam::NotImplementedError& e) {
            PyErr_SetString(PyExc_NotImplementedError, e.what());
        }
    });

    nb::class_<vegam::Limits>(m, "Limits", "Per-joint kinematic limits.")
        .def(nb::init<>())
        .def(
            "__init__",
            [](vegam::Limits* self, std::vector<double> max_velocity,
               std::vector<double> max_acceleration, std::vector<double> max_jerk) {
                new (self) vegam::Limits{std::move(max_velocity),
                                         std::move(max_acceleration),
                                         std::move(max_jerk)};
            },
            "max_velocity"_a, "max_acceleration"_a, "max_jerk"_a)
        .def_rw("max_velocity", &vegam::Limits::max_velocity)
        .def_rw("max_acceleration", &vegam::Limits::max_acceleration)
        .def_rw("max_jerk", &vegam::Limits::max_jerk);

    nb::class_<vegam::Trajectory>(m, "Trajectory",
                                  "A time-parameterized trajectory.")
        .def(nb::init<>())
        .def_ro("times", &vegam::Trajectory::times)
        .def_ro("positions", &vegam::Trajectory::positions)
        .def_prop_ro("duration", &vegam::Trajectory::duration);

    m.def("parameterize", &vegam::parameterize, "waypoints"_a, "limits"_a,
          "Compute a jerk-limited time-optimal parameterization of the path "
          "through `waypoints` (one row per waypoint, one column per joint).");
}
