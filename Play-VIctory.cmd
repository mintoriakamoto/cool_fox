@echo off
REM Windows launcher: calls the PowerShell auto-tuner, which reads GPU/RAM/PCIe specs
REM and auto-sets texture and DirectStorage staging, or runs help options.
REM
REM Usage:
REM   Play-VIctory.cmd                       Run with auto-tuned settings for your GPU.
REM   powershell -ExecutionPolicy Bypass -File Play-VIctory.ps1 --print-env
REM                                          Show tuned values without running.

setlocal EnableExtensions DisableDelayedExpansion
cd /d "%~dp0" || exit /b 1
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0Play-VIctory.ps1" %*
exit /b %errorlevel%
