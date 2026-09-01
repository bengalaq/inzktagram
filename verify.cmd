@echo off
REM Verifica un receipt guardado en download_receipts\ (cmd.exe o PowerShell).
cd /d "%~dp0"
powershell -NoProfile -ExecutionPolicy Bypass -File "%~dp0verify.ps1" %*
exit /b %ERRORLEVEL%
