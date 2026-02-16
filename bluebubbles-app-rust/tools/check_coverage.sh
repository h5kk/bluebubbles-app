#!/usr/bin/env bash
# BlueBubbles Rust Rewrite - Coverage Tracker
# Run from the tools/ directory or pass the script path.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
python3 "$SCRIPT_DIR/coverage_tracker.py" "$@"
