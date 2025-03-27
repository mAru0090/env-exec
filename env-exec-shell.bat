@echo off
env-exec "%USERPROFILE%/env-exec-config.toml" powershell -NoExit -Command "Set-Location -Path '%USERPROFILE%'"
