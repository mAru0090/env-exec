@echo off
setlocal
chcp 65001
set "eec_deleter=eec-deleter.exe"
REM set "test_arg1=--env-exec-deleter-hide"

tasklist /FI "IMAGENAME eq %eec_deleter%" /NH | find /I "%eec_deleter%" >nul
if %ERRORLEVEL% equ 0 (
    echo [%eec_deleter%] は既に実行中です。
) else (
    echo [%eec_deleter%] を起動します…
    REM start "%eec_deleter%" "%eec_deleter%"	
    if "%1" == "--env-exec-deleter-hide" (
       REM powershell -WindowStyle Normal -Command "Start-Process -FilePath '%eec_deleter%' -WindowStyle Normal"
       start "%eec_deleter%" "%eec_deleter%"
    ) else (
      	powershell -WindowStyle Normal -Command "Start-Process -FilePath '%eec_deleter%' -WindowStyle Hidden"	
    )
)

REM eec run  --config-file "%USERPROFILE%/env-exec-config.toml" --program cmd -- "/K cd /d %USERPROFILE%"
eec run --tag cmd00
