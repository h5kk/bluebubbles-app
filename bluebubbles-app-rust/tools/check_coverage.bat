@echo off
REM BlueBubbles Rust Rewrite - Coverage Tracker
REM Run from the tools/ directory or pass the script path.

python "%~dp0coverage_tracker.py" %*
