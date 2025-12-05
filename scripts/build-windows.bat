@echo off
REM PortKiller Windows Build Script
REM Simple wrapper for the PowerShell build script

echo.
echo ========================================
echo  PortKiller Windows Build
echo ========================================
echo.

REM Check if PowerShell is available
where pwsh >nul 2>nul
if %ERRORLEVEL% EQU 0 (
    echo Using PowerShell Core...
    pwsh -ExecutionPolicy Bypass -File "%~dp0build-windows.ps1" %*
) else (
    where powershell >nul 2>nul
    if %ERRORLEVEL% EQU 0 (
        echo Using Windows PowerShell...
        powershell -ExecutionPolicy Bypass -File "%~dp0build-windows.ps1" %*
    ) else (
        echo ERROR: PowerShell not found!
        echo Please install PowerShell to build this project.
        exit /b 1
    )
)

exit /b %ERRORLEVEL%
