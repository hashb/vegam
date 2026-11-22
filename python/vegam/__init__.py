"""Jerk-limited time-optimal path parameterization."""

from importlib.metadata import version

from ._core import Limits, Trajectory, parameterize

__version__ = version("pyvegam")

__all__ = ["Limits", "Trajectory", "__version__", "parameterize"]

# hoge
