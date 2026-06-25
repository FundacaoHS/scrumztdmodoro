@echo off
cd /d "%~dp0sm-tauri"
cargo tauri dev %*
