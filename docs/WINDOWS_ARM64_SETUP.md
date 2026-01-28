# Windows ARM64 Development Setup Guide

This guide is specifically for setting up a development environment on **Windows ARM64** devices, such as Microsoft Surface Pro X, Surface Pro 9 (5G), or laptops powered by Qualcomm Snapdragon processors.

## Why ARM64 Requires Special Setup

ARM64 Windows is a different CPU architecture from the traditional x64 (Intel/AMD). While Windows provides x64 emulation, compiling native code requires ARM64-specific toolchains. This project uses Rust with native dependencies that must be compiled specifically for ARM64, which requires additional setup beyond what x64 developers need.

---

## Prerequisites

Before starting, ensure you have:

- **Windows 11 ARM64** - Required for ARM64 development
- **Git** - For cloning the repository
- **Rust toolchain** (via [rustup](https://rustup.rs/)) - The Tauri backend is written in Rust
- **Node.js** (LTS version recommended) - Required for the frontend build process

### Why these are needed

| Prerequisite | Reason |
|--------------|--------|
| Rust | Tauri's backend is written in Rust; all native functionality compiles through the Rust toolchain |
| Node.js | The frontend uses web technologies (React/TypeScript) that require Node.js for building and bundling |

---

## Visual Studio Setup

The Rust toolchain on Windows uses **MSVC (Microsoft Visual C++)** as its default compiler backend. You must install the ARM64-specific build tools.

### Installation Steps

1. **Download Visual Studio Community 2022** (or latest) from [visualstudio.microsoft.com](https://visualstudio.microsoft.com/)

2. **Open Visual Studio Installer**

3. **Select the "Desktop development with C++" workload**

4. **In the Installation details panel** (right side), ensure these components are checked:
   - MSVC v143 - VS 2022 C++ ARM64/ARM64EC build tools (Latest)
   - C++ ATL for latest v143 build tools (ARM64/ARM64EC)
   - Windows 11 SDK (10.0.22621.0 or latest)

5. Click **Install** and wait for completion

### Why Visual Studio is Required

| Component | Purpose |
|-----------|---------|
| MSVC Build Tools for ARM64 | Compiles native Rust code and C/C++ dependencies for ARM64 architecture |
| Windows SDK | Provides headers and libraries for Windows API calls used by Tauri |
| C++ ATL | Required by some Windows-specific crates that interface with COM/Windows APIs |

The ARM64 build tools are **specifically required** because standard x64 tools cannot compile code for ARM64 architecture—they produce incompatible binaries.

### What Happens Without It

You'll encounter errors like:
```
error: linker `link.exe` not found
```
or
```
error: could not find native static library `windows`, perhaps an -L flag is missing?
```

---

## LLVM/Clang Installation

This project requires **LLVM/Clang** specifically because of the `ring` cryptography crate in our dependency chain.

### Installation Steps

1. **Download LLVM for Windows ARM64** from the [LLVM GitHub Releases](https://github.com/llvm/llvm-project/releases)
   - Look for a file named like `LLVM-19.1.0-woa64.exe`
   - The `woa64` suffix means "Windows on ARM64"

2. **Run the installer**

3. **Important:** Check the option **"Add LLVM to the system PATH for all users"** (or current user)

4. **Restart your terminal** after installation to pick up the new PATH

### Why Clang is Required

The dependency chain that requires Clang:

```
sea-orm (database ORM)
    └── sqlx (async SQL toolkit)
            └── rustls (TLS implementation)
                    └── ring (cryptography library)
```

The `ring` crate contains hand-written assembly code for cryptographic operations. On ARM64 Windows specifically:

- **MSVC cannot compile this assembly code** (it uses a different assembly syntax)
- **Clang is required** to compile the assembly portions of `ring`
- This is a known limitation specific to the ARM64 Windows platform

### What Happens Without It

The build fails with:
```
error occurred in cc-rs: failed to find tool "clang"
```

or similar errors during compilation of the `ring` crate.

---

## Verification Steps

After installing all prerequisites, verify your setup:

### 1. Verify Rust is targeting MSVC

```powershell
rustc --print cfg | findstr msvc
```

**Expected output:**
```
target_env="msvc"
```

### 2. Verify Clang is installed and in PATH

```powershell
clang --version
```

**Expected output:** (version numbers may vary)
```
clang version 19.1.0
Target: aarch64-pc-windows-msvc
Thread model: posix
```

### 3. Verify the project builds

```powershell
cd apps/web/src-tauri
cargo build
```

A successful build confirms all tools are properly configured.

---

## Troubleshooting

### Common Errors and Solutions

| Error | Cause | Solution |
|-------|-------|----------|
| `failed to find tool "clang"` | LLVM not installed or not in PATH | Install LLVM ARM64 and ensure "Add to PATH" was checked during installation |
| `linker 'link.exe' not found` | MSVC not installed | Install Visual Studio with C++ Desktop workload |
| `ring` build failures with ARM64 errors | Missing ARM64 MSVC build tools | Open VS Installer and add ARM64/ARM64EC build tools |
| `windows.lib` not found | Windows SDK not installed | Add Windows 11 SDK in VS Installer |

### Useful Diagnostic Commands

**Find where `cl.exe` (MSVC compiler) is installed:**
```powershell
Get-ChildItem "C:\Program Files\Microsoft Visual Studio" -Recurse -Filter "cl.exe" 2>$null | Select-Object FullName
```

Look for a path containing `arm64` to confirm ARM64 tools are installed.

**Check if LLVM/Clang is in PATH:**
```powershell
where.exe clang
```

**Check your Rust installation:**
```powershell
rustup show
```

This displays your default toolchain and installed targets.

**Rebuild with verbose output for debugging:**
```powershell
cargo build -vv
```

The `-vv` flag shows detailed compilation commands, useful for diagnosing tool path issues.

---

## Alternative: Using native-tls Instead of rustls

If you want to avoid installing LLVM/Clang, you can modify the project to use Windows' built-in TLS (Schannel) instead of `rustls`.

### How to Switch

In `apps/web/src-tauri/Cargo.toml`, change the `sea-orm` feature from:

```toml
sea-orm = { version = "...", features = ["runtime-tokio-rustls", ...] }
```

to:

```toml
sea-orm = { version = "...", features = ["runtime-tokio-native-tls", ...] }
```

### Trade-offs

| Aspect | rustls | native-tls |
|--------|--------|------------|
| Setup complexity | Requires LLVM/Clang on ARM64 | No extra tools needed |
| Build time | Slower (compiles crypto code) | Faster |
| TLS updates | Via dependency updates | Via Windows updates |
| Cross-platform consistency | Same TLS on all platforms | Uses OS-specific TLS |

**Note:** Switching to `native-tls` may require additional changes if other dependencies also use `rustls`. This alternative is mentioned for awareness but hasn't been tested with this specific project.

---

## Additional Resources

- [Tauri Prerequisites](https://tauri.app/v1/guides/getting-started/prerequisites) - Official Tauri setup guide
- [Rust on Windows](https://rust-lang.github.io/rustup/installation/windows.html) - Rust installation documentation
- [LLVM Releases](https://github.com/llvm/llvm-project/releases) - Download LLVM/Clang
- [ring crate documentation](https://github.com/briansmith/ring#building) - Details on ring's build requirements
