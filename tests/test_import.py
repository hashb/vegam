import pytest

import vegam


def test_import():
    assert vegam.__version__
    assert vegam._core.__doc__


def test_limits_roundtrip():
    limits = vegam.Limits(max_velocity=[1.0], max_acceleration=[2.0], max_jerk=[3.0])
    assert limits.max_velocity == [1.0]
    assert limits.max_acceleration == [2.0]
    assert limits.max_jerk == [3.0]


def test_empty_trajectory():
    traj = vegam.Trajectory()
    assert traj.times == []
    assert traj.duration == 0.0


def test_parameterize_not_implemented():
    limits = vegam.Limits([1.0], [1.0], [1.0])
    with pytest.raises(NotImplementedError):
        vegam.parameterize([[0.0], [1.0]], limits)
