@echo off
setlocal
cd /d "%~dp0"

echo ==================================================
echo  TLC LINK - Windows 10/11 Builder (Installer + Portable)
echo ==================================================
echo.

where node >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Node.js is not installed. Download LTS from https://nodejs.org/
  pause
  exit /b 1
)

where cargo >nul 2>nul
if errorlevel 1 (
  echo [ERROR] Rust/Cargo is not installed. Download from https://rustup.rs/
  pause
  exit /b 1
)

if not exist "src-tauri\binaries\sing-box\sing-box.exe" (
  echo [ERROR] Missing src-tauri\binaries\sing-box\sing-box.exe
  echo Please place official sing-box.exe in src-tauri\binaries\sing-box\sing-box.exe
  pause
  exit /b 1
)

if not exist "src-tauri\binaries\xray\xray.exe" (
  echo [ERROR] Missing src-tauri\binaries\xray\xray.exe
  echo Please place official xray.exe in src-tauri\binaries\xray\xray.exe
  pause
  exit /b 1
)

echo [1/2] Building Windows Installer (.exe / .msi)...
powershell -ExecutionPolicy Bypass -File "scripts\build-windows.ps1"
if errorlevel 1 (
  echo [ERROR] Installer build failed.
  pause
  exit /b 1
)

echo.
echo [2/2] Building Windows Portable Package (.exe + zip)...
powershell -ExecutionPolicy Bypass -File "scripts\build-portable-windows.ps1"
if errorlevel 1 (
  echo [ERROR] Portable build failed.
  pause
  exit /b 1
)

echo.
echo ==================================================
echo  BUILD COMPLETE!
echo  - Installer (.exe): src-tauri\target\release\bundle\nsis\
echo  - Installer (.msi): src-tauri\target\release\bundle\msi\
echo  - Portable (.exe):  dist\TLC-LINK-Portable\TLC-LINK.exe
echo  - Portable (.zip):  dist\TLC-LINK-Portable.zip
echo ==================================================
pause
