@echo off
echo env-exec build...

:: 最初のディレクトリを保存
set "ORIGINAL_DIR=%CD%"

:: 環境変数の設定
set "LIB=%LIB%;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\ucrt\x64"
set "LIB=%LIB%;C:\Program Files (x86)\Windows Kits\10\Lib\10.0.26100.0\um\x64"
set "LIB=%LIB%;D:\win\dev-tools\microsoft-build-tools\BuildTools\VC\Tools\MSVC\14.39.33519\lib\x64"
set "PATH=%PATH%;D:\win\dev-tools\rust\cargo-home\bin"
set "PATH=%PATH%;D:\win\dev-tools\microsoft-build-tools\BuildTools\VC\Tools\MSVC\14.39.33519\bin\Hostx64\x64"
set "RUSTUP_HOME=D:\win\dev-tools\rust\rustup-home"
set "CARGO_HOME=D:\win\dev-tools\rust\cargo-home"

:: カレントディレクトリを変更
cd /d D:\win\program\rs\main-project\env-exec\

:: Cargoビルド実行
cargo build --release

echo finished env-exec build.

:: 元のディレクトリに戻る
cd /d "%ORIGINAL_DIR%"

:: キー入力を待機
pause
