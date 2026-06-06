@echo off
setlocal
cd /d "%~dp0"

if exist "collectionloom.exe" (
  start "" "%~dp0collectionloom.exe"
  exit /b 0
)

echo collectionloom.exe not found in %~dp0
exit /b 1
