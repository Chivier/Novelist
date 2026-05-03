@echo off
rem Novelist CLI shim for Windows.
rem Copy or alias to a directory on PATH (e.g. via the in-app installer);
rem forwards all args to the bundled novelist.exe and detaches.

setlocal
set "SHIM_DIR=%~dp0"

rem Conventional install layouts:
rem   resources\bundled-cli\novelist.cmd  -> ..\..\novelist.exe
rem   resources\bundled-cli\novelist.cmd  -> ..\novelist.exe
for %%P in (
    "%SHIM_DIR%..\..\novelist.exe"
    "%SHIM_DIR%..\novelist.exe"
    "%SHIM_DIR%novelist.exe"
) do (
    if exist %%~fP (
        start "" "%%~fP" %*
        endlocal
        exit /b 0
    )
)

echo novelist: cannot locate novelist.exe near %SHIM_DIR% 1>&2
echo          re-run "Install 'novelist' command in PATH" from inside the app. 1>&2
endlocal
exit /b 1
