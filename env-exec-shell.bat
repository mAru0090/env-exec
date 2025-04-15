@echo off
env-exec "%USERPROFILE%\env-exec-config.toml" powershell -NoExit -Command "Set-ExecutionPolicy RemoteSigned -Scope Process; Set-Location -Path '%USERPROFILE%'"
