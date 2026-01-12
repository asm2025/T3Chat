@echo off
setlocal

echo ========================================
echo T3Chat Rust Bridge Build Script
echo ========================================
echo.

REM Step 1: Delete .dart_tool folder
echo [1/5] Deleting .dart_tool folder...
if exist ".dart_tool" (
    rmdir /s /q ".dart_tool"
    if errorlevel 1 (
        echo ERROR: Failed to delete .dart_tool folder
        exit /b 1
    )
    echo   .dart_tool folder deleted successfully
) else (
    echo   .dart_tool folder not found, skipping
)
echo.

REM Step 2: Delete pubspec.lock
echo [2/5] Deleting pubspec.lock...
if exist "pubspec.lock" (
    del /q "pubspec.lock"
    if errorlevel 1 (
        echo ERROR: Failed to delete pubspec.lock
        exit /b 1
    )
    echo   pubspec.lock deleted successfully
) else (
    echo   pubspec.lock not found, skipping
)
echo.

REM Step 3: Build Rust crate
echo [3/5] Building Rust crate (release mode)...
cd native\t3chat_core
if errorlevel 1 (
    echo ERROR: Failed to change directory to native\t3chat_core
    exit /b 1
)

cargo build --release
if errorlevel 1 (
    echo ERROR: Cargo build failed
    cd ..\..
    exit /b 1
)
echo   Rust build completed successfully
cd ..\..
echo.

REM Step 4: Generate Flutter Rust Bridge bindings
echo [4/5] Generating Flutter Rust Bridge bindings...
flutter_rust_bridge_codegen generate --config-file native/t3chat_core/flutter_rust_bridge.yaml
if errorlevel 1 (
    echo ERROR: Flutter Rust Bridge code generation failed
    exit /b 1
)
echo   Bridge bindings generated successfully
echo.

REM Step 5: Get Flutter dependencies
echo [5/5] Running flutter pub get...
flutter pub get
if errorlevel 1 (
    echo ERROR: Flutter pub get failed
    exit /b 1
)
echo   Dependencies installed successfully
echo.

echo ========================================
echo Build completed successfully!
echo ========================================
exit /b 0
