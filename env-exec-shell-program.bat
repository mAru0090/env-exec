@echo off
setlocal
chcp 65001
set "eec_deleter=eec-deleter.exe"

tasklist /FI "IMAGENAME eq %eec_deleter%" /NH | find /I "%eec_deleter%" >nul
if %ERRORLEVEL% equ 0 (
    echo [%eec_deleter%] は既に実行中です。
) else (
    echo [%eec_deleter%] を起動します…
    start "" "%eec_deleter%"
)

eec run --config-file "%USERPROFILE%\env-exec-config.toml" --program powershell --  "-NoExit" "-Command" "Set-ExecutionPolicy RemoteSigned -Scope Process; Set-Location -Path 'D:\win\program\'"