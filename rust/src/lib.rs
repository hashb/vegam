//! Jerk-limited time-optimal path parameterization (TOPP).
//!
//! Rust bindings to the vegam C++ library. The C++ sources are bundled and
//! compiled by the build script, so the only requirement is a C++17 compiler.
//!
//! ```no_run
//! use vegam::{Limits, parameterize};
//!
//! let waypoints = [[0.0, 0.0], [1.0, 0.5], [2.0, 0.0]];
//! let limits = Limits::new(vec![1.0; 2], vec![2.0; 2], vec![10.0; 2]);
//! let trajectory = parameterize(&waypoints, &limits)?;
//! println!("duration: {} s", trajectory.duration());
//! # Ok::<(), vegam::Error>(())
//! ```

#![warn(missing_docs)]

use std::ffi::{CStr, c_char};
use std::fmt;
use std::ptr;

mod ffi;

/// Per-joint kinematic limits. Each vector has one entry per joint.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Limits {
    /// Maximum absolute velocity of each joint.
    pub max_velocity: Vec<f64>,
    /// Maximum absolute acceleration of each joint.
    pub max_acceleration: Vec<f64>,
    /// Maximum absolute jerk of each joint.
    pub max_jerk: Vec<f64>,
}

impl Limits {
    /// Creates limits from per-joint velocity, acceleration and jerk bounds.
    pub fn new(max_velocity: Vec<f64>, max_acceleration: Vec<f64>, max_jerk: Vec<f64>) -> Self {
        Self {
            max_velocity,
            max_acceleration,
            max_jerk,
        }
    }
}

/// A time-parameterized trajectory sampled at discrete times.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Trajectory {
    /// Sample times in seconds, increasing.
    pub times: Vec<f64>,
    /// Joint positions per sample.
    pub positions: Vec<Vec<f64>>,
}

impl Trajectory {
    /// Total duration in seconds; 0 for an empty trajectory.
    pub fn duration(&self) -> f64 {
        match (self.times.first(), self.times.last()) {
            (Some(first), Some(last)) => last - first,
            _ => 0.0,
        }
    }
}

/// The category of an [`Error`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[non_exhaustive]
pub enum ErrorKind {
    /// The requested functionality is not implemented yet.
    NotImplemented,
    /// An argument was rejected, for example mismatched joint counts.
    InvalidArgument,
    /// Any other failure inside the C++ library.
    Other,
}

/// An error reported by the vegam C++ library.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    kind: ErrorKind,
    message: String,
}

impl Error {
    /// The category of this error.
    pub fn kind(&self) -> ErrorKind {
        self.kind
    }

    /// The message reported by the C++ library.
    pub fn message(&self) -> &str {
        &self.message
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl std::error::Error for Error {}

/// Computes a jerk-limited time-optimal parameterization of the path through
/// `waypoints` (one row per waypoint, one column per joint).
pub fn parameterize<W: AsRef<[f64]>>(
    waypoints: &[W],
    limits: &Limits,
) -> Result<Trajectory, Error> {
    let rows: Vec<*const f64> = waypoints.iter().map(|w| w.as_ref().as_ptr()).collect();
    let row_lens: Vec<usize> = waypoints.iter().map(|w| w.as_ref().len()).collect();
    let mut out = ptr::null_mut();
    let mut err = ptr::null_mut();

    // SAFETY: every pointer/length pair describes a live slice borrowed for
    // the duration of the call, and the out-pointers are valid for writes.
    let status = unsafe {
        ffi::vegam_parameterize(
            rows.as_ptr(),
            row_lens.as_ptr(),
            rows.len(),
            limits.max_velocity.as_ptr(),
            limits.max_velocity.len(),
            limits.max_acceleration.as_ptr(),
            limits.max_acceleration.len(),
            limits.max_jerk.as_ptr(),
            limits.max_jerk.len(),
            &mut out,
            &mut err,
        )
    };

    if status == ffi::VEGAM_OK {
        // SAFETY: on success `out` is a valid trajectory that we now own.
        let trajectory = unsafe { read_trajectory(out) };
        unsafe { ffi::vegam_trajectory_free(out) };
        return Ok(trajectory);
    }

    let kind = match status {
        ffi::VEGAM_NOT_IMPLEMENTED => ErrorKind::NotImplemented,
        ffi::VEGAM_INVALID_ARGUMENT => ErrorKind::InvalidArgument,
        _ => ErrorKind::Other,
    };
    // SAFETY: `err` is null or a NUL-terminated string that we now own.
    let message = unsafe { take_message(err) };
    Err(Error { kind, message })
}

/// Copies a C++ trajectory into Rust-owned memory.
///
/// # Safety
/// `traj` must point to a live `vegam_trajectory`.
unsafe fn read_trajectory(traj: *const ffi::vegam_trajectory) -> Trajectory {
    unsafe {
        let times = copy_slice(
            ffi::vegam_trajectory_times(traj),
            ffi::vegam_trajectory_times_len(traj),
        );
        let positions = (0..ffi::vegam_trajectory_num_positions(traj))
            .map(|i| {
                copy_slice(
                    ffi::vegam_trajectory_position(traj, i),
                    ffi::vegam_trajectory_position_len(traj, i),
                )
            })
            .collect();
        Trajectory { times, positions }
    }
}

/// # Safety
/// If `len > 0`, `data` must point to `len` initialized doubles.
unsafe fn copy_slice(data: *const f64, len: usize) -> Vec<f64> {
    if len == 0 {
        // `std::vector::data()` may be null when empty, which `from_raw_parts` forbids.
        Vec::new()
    } else {
        unsafe { std::slice::from_raw_parts(data, len) }.to_vec()
    }
}

/// # Safety
/// `err` must be null or a string allocated by the C++ shim; it is freed here.
unsafe fn take_message(err: *mut c_char) -> String {
    if err.is_null() {
        return "vegam: unknown error (no message)".to_owned();
    }
    let message = unsafe { CStr::from_ptr(err) }
        .to_string_lossy()
        .into_owned();
    unsafe { ffi::vegam_string_free(err) };
    message
}
