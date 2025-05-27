@echo off
chcp 65001
echo タグを設定します...
eec-tag --tag-name powershell00 --config-file "%USERPROFILE%\env-exec-configs\env-exec-config.toml" --program "powershell"  -- "-NoExit" "-Command" "Set-ExecutionPolicy RemoteSigned -Scope Process; checkitems %USERPROFILE%\env-exec-configs\checkitems.csv"
eec-tag --tag-name powershell01 --config-file "%USERPROFILE%\env-exec-configs\env-exec-config.toml" --program "powershell"  -- "-NoExit" "-Command" "Set-ExecutionPolicy RemoteSigned -Scope Process; checkitems %USERPROFILE%\env-exec-configs\checkitems.csv; Set-Location -Path 'D:\win\program\'"
eec-tag --tag-name cmd00 --config-file "%USERPROFILE%\env-exec-configs\env-exec-config.toml"  --program cmd -- "/K checkitems %USERPROFILE%\env-exec-configs\checkitems.csv"
eec-tag --tag-name cmd01 --config-file "%USERPROFILE%\env-exec-configs\env-exec-config.toml " --program cmd -- "/K checkitems %USERPROFILE%\env-exec-configs\checkitems.csv & cd /d D:\win\program\"


echo タグの設定が終了しました
:: キー入力を待機
pause
