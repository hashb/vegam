#!/usr/bin/env python3
"""Bump the version, tag it and create a GitHub release.

Publishing the release triggers .github/workflows/wheels.yml, which builds the
sdist and wheels and uploads them to PyPI.

Usage:
    python scripts/release.py patch            # 0.1.2 -> 0.1.3
    python scripts/release.py minor            # 0.1.2 -> 0.2.0
    python scripts/release.py major            # 0.1.2 -> 1.0.0
    python scripts/release.py 0.2.0rc1         # explicit version
    python scripts/release.py patch --dry-run  # show what would happen

Requires git, and the GitHub CLI (gh) logged in to this repository.
"""

from __future__ import annotations

import argparse
import re
import shutil
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
PYPROJECT = ROOT / "pyproject.toml"

# X.Y.Z with an optional pre-release suffix (a1, b2, rc1).
VERSION_RE = re.compile(r"^(\d+)\.(\d+)\.(\d+)((?:a|b|rc)\d+)?$")
# The `version = "..."` line inside the [project] table.
PROJECT_VERSION_RE = re.compile(
    r'(^\[project\]\s*$.*?^version\s*=\s*")([^"]+)(")', re.MULTILINE | re.DOTALL
)


def run(*cmd: str, capture: bool = False) -> str:
    result = subprocess.run(
        cmd, cwd=ROOT, check=True, text=True, capture_output=capture
    )
    return result.stdout.strip() if capture else ""


def fail(msg: str) -> None:
    sys.exit(f"error: {msg}")


def read_version() -> str:
    match = PROJECT_VERSION_RE.search(PYPROJECT.read_text())
    if not match:
        fail("could not find version in [project] of pyproject.toml")
    return match.group(2)


def write_version(new: str) -> None:
    text = PYPROJECT.read_text()
    PYPROJECT.write_text(PROJECT_VERSION_RE.sub(rf"\g<1>{new}\g<3>", text, count=1))


def next_version(current: str, part: str) -> str:
    if VERSION_RE.match(part):
        return part
    match = VERSION_RE.match(current)
    if not match:
        fail(f"current version {current!r} is not X.Y.Z[a|b|rcN]; pass one explicitly")
    major, minor, patch = (int(x) for x in match.group(1, 2, 3))
    prerelease = match.group(4)
    if part == "major":
        return f"{major + 1}.0.0"
    if part == "minor":
        return f"{major}.{minor + 1}.0"
    if part == "patch":
        # 1.2.0rc1 -> 1.2.0 (finalise the pre-release) rather than 1.2.1.
        return f"{major}.{minor}.{patch if prerelease else patch + 1}"
    fail(f"expected major, minor, patch or a version like 1.2.3, got {part!r}")
    raise AssertionError  # unreachable


def check_repo(branch: str) -> None:
    if run("git", "status", "--porcelain", capture=True):
        fail("working tree has uncommitted changes")
    current = run("git", "rev-parse", "--abbrev-ref", "HEAD", capture=True)
    if current != branch:
        fail(f"on branch {current!r}, releases are made from {branch!r} (see --branch)")
    run("git", "fetch", "--quiet", "origin", branch)
    local = run("git", "rev-parse", "HEAD", capture=True)
    remote = run("git", "rev-parse", f"origin/{branch}", capture=True)
    if local != remote:
        fail(f"{branch} is not in sync with origin/{branch}; pull or push first")


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.splitlines()[0])
    parser.add_argument("bump", help="major, minor, patch, or an explicit version")
    parser.add_argument("--branch", default="main", help="branch to release from")
    parser.add_argument("--draft", action="store_true", help="create a draft release (nothing is published until you publish it on GitHub)")
    parser.add_argument("--dry-run", action="store_true", help="print the plan and change nothing")
    parser.add_argument("-y", "--yes", action="store_true", help="do not ask for confirmation")
    args = parser.parse_args()

    for tool in ("git", "gh"):
        if not shutil.which(tool):
            fail(f"{tool} is not installed")

    current = read_version()
    new = next_version(current, args.bump)
    tag = f"v{new}"
    prerelease = VERSION_RE.match(new).group(4) is not None

    if run("git", "tag", "--list", tag, capture=True):
        fail(f"tag {tag} already exists")

    print(f"version: {current} -> {new}")
    print(f"tag:     {tag} on {args.branch}")
    print(f"release: {'draft' if args.draft else 'published'}{' pre-release' if prerelease else ''}")

    if args.dry_run:
        return
    check_repo(args.branch)
    if not args.yes and input("proceed? [y/N] ").strip().lower() != "y":
        sys.exit("aborted")

    write_version(new)
    files = ["pyproject.toml"]
    if (ROOT / "uv.lock").exists() and shutil.which("uv"):
        run("uv", "lock", "--quiet")
        files.append("uv.lock")

    run("git", "add", *files)
    run("git", "commit", "--quiet", "-m", f"Release {tag}")
    run("git", "tag", "-a", tag, "-m", f"Release {tag}")
    run("git", "push", "--quiet", "--atomic", "origin", args.branch, tag)

    gh = ["gh", "release", "create", tag, "--title", tag, "--generate-notes", "--verify-tag"]
    if args.draft:
        gh.append("--draft")
    if prerelease:
        gh.append("--prerelease")
    run(*gh)

    print(f"released {tag}; the build and publish workflow is now running:")
    print("  gh run list --workflow wheels.yml --limit 1")


if __name__ == "__main__":
    try:
        main()
    except subprocess.CalledProcessError as e:
        fail(f"command failed: {' '.join(e.cmd)}")
