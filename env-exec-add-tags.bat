@echo off
chcp 65001
echo タグを設定します...
eec-tag --tag-name powershell00 --config-file "%USERPROFILE%\env-exec-config.toml" --program "powershell"  -- "-NoExit" "-Command" "Set-ExecutionPolicy RemoteSigned -Scope Process;" "Set-Location -Path $env:USERPROFILE"
eec-tag --tag-name powershell01 --config-file "%USERPROFILE%\env-exec-config.toml" --program "powershell"  -- "-NoExit" "-Command" "Set-ExecutionPolicy RemoteSigned -Scope Process; Set-Location -Path 'D:\win\program\'"
eec-tag --tag-name cmd00 --config-file "%USERPROFILE%/env-exec-config.toml" --program cmd -- "/K cd /d %USERPROFILE%"
eec-tag --tag-name cmd01 --config-file "%USERPROFILE%/env-exec-config.toml" --program cmd -- "/K cd /d D:\win\program\"


echo タグの設定が終了しました
:: キー入力を待機
pause
