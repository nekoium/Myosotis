@echo off
setlocal
if exist "%~dp0dist\Myosotis\myosotis.exe" (
    start "" "%~dp0dist\Myosotis\myosotis.exe" --capture
) else (
    echo Build the portable app first. See CONTRIBUTING.md for source setup.
    pause
)
