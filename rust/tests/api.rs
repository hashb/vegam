use vegam::{ErrorKind, Limits, Trajectory, parameterize};

#[test]
fn limits_roundtrip() {
    let limits = Limits::new(vec![1.0], vec![2.0], vec![3.0]);
    assert_eq!(limits.max_velocity, [1.0]);
    assert_eq!(limits.max_acceleration, [2.0]);
    assert_eq!(limits.max_jerk, [3.0]);
}

#[test]
fn empty_trajectory() {
    let traj = Trajectory::default();
    assert!(traj.times.is_empty());
    assert_eq!(traj.duration(), 0.0);
}

#[test]
fn trajectory_duration() {
    let traj = Trajectory {
        times: vec![0.5, 1.0, 2.5],
        positions: vec![vec![0.0]; 3],
    };
    assert_eq!(traj.duration(), 2.0);
}

#[test]
fn parameterize_not_implemented() {
    let limits = Limits::new(vec![1.0], vec![1.0], vec![1.0]);
    let err = parameterize(&[[0.0], [1.0]], &limits).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotImplemented);
    assert!(err.message().contains("not implemented"), "{err}");
}

#[test]
fn parameterize_accepts_empty_input() {
    // Empty slices must cross the FFI boundary without issue.
    let waypoints: [Vec<f64>; 0] = [];
    let err = parameterize(&waypoints, &Limits::default()).unwrap_err();
    assert_eq!(err.kind(), ErrorKind::NotImplemented);
}
