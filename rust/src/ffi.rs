//! Raw declarations for the C ABI in `rust/cpp/ffi.cpp`.

use std::ffi::c_char;

pub const VEGAM_OK: i32 = 0;
pub const VEGAM_NOT_IMPLEMENTED: i32 = 1;
pub const VEGAM_INVALID_ARGUMENT: i32 = 2;

#[repr(C)]
pub struct vegam_trajectory {
    _private: [u8; 0],
}

unsafe extern "C" {
    #[allow(clippy::too_many_arguments)]
    pub fn vegam_parameterize(
        rows: *const *const f64,
        row_lens: *const usize,
        n_rows: usize,
        max_velocity: *const f64,
        n_velocity: usize,
        max_acceleration: *const f64,
        n_acceleration: usize,
        max_jerk: *const f64,
        n_jerk: usize,
        out: *mut *mut vegam_trajectory,
        err: *mut *mut c_char,
    ) -> i32;

    pub fn vegam_trajectory_free(traj: *mut vegam_trajectory);
    pub fn vegam_trajectory_times_len(traj: *const vegam_trajectory) -> usize;
    pub fn vegam_trajectory_times(traj: *const vegam_trajectory) -> *const f64;
    pub fn vegam_trajectory_num_positions(traj: *const vegam_trajectory) -> usize;
    pub fn vegam_trajectory_position_len(traj: *const vegam_trajectory, i: usize) -> usize;
    pub fn vegam_trajectory_position(traj: *const vegam_trajectory, i: usize) -> *const f64;

    pub fn vegam_string_free(s: *mut c_char);
}
