# Fix PostgreSQL Linking Error on Windows (GNU Toolchain)

## Problem
The linker cannot find PostgreSQL libraries (`-llibpq` and `-lpq`) when compiling with the GNU toolchain.

## Solution

### Step 1: Install PostgreSQL Libraries via MSYS2

Open an **MSYS2 UCRT64** terminal (not regular PowerShell) and run:

```bash
pacman -S mingw-w64-ucrt-x86_64-postgresql
```

This installs PostgreSQL client libraries for the UCRT64 environment.

### Step 2: Set Environment Variables

In PowerShell (or Command Prompt), set the environment variables:

```powershell
# For UCRT64 environment (matches your current setup)
setx PQ_LIB_DIR "C:\msys64\ucrt64\lib"
setx PQ_INCLUDE_DIR "C:\msys64\ucrt64\include"
```

**Note:** If your MSYS2 is installed in a different location, adjust the path accordingly.

### Step 3: Add DLL to PATH (for runtime)

```powershell
setx PATH "%PATH%;C:\msys64\ucrt64\bin"
```

### Step 4: Restart Your Terminal

Close and reopen your terminal/IDE for the environment variables to take effect.

### Step 5: Verify Installation

```powershell
# Check if libpq.dll exists
where libpq.dll
# Should show: C:\msys64\ucrt64\bin\libpq.dll

# Verify Rust toolchain
rustc --version
```

### Step 6: Clean and Rebuild

```powershell
cd D:\Work\rust\T3Chat\server
cargo clean
cargo build
```

## Alternative: Use MSVC Toolchain Instead

If you prefer to avoid MinGW setup, you can switch to the MSVC toolchain:

1. Install PostgreSQL from [postgresql.org](https://www.postgresql.org/download/windows/)
2. Set environment variables:
   ```powershell
   setx PQ_LIB_DIR "C:\Program Files\PostgreSQL\18\lib"
   setx PQ_INCLUDE_DIR "C:\Program Files\PostgreSQL\18\include"
   ```
3. The MSVC toolchain is the default, so no need to change Rust toolchain

## Troubleshooting

- **If `pacman` command not found**: Make sure you're running it in an MSYS2 UCRT64 terminal, not regular PowerShell
- **If libraries still not found**: Check that `PQ_LIB_DIR` points to a directory containing `libpq.a` (for static linking) or `libpq.dll.a` (for dynamic linking)
- **If DLL not found at runtime**: Ensure the PostgreSQL `bin` directory is in your `PATH`
