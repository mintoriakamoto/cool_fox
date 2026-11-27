# Victory Game Launcher - Deployment & Testing Guide

## Current Status

✅ **Linux Complete**: Core profiling system fully functional
- Metrics collection: GPU (nvidia-smi), CPU (/proc/stat), VRAM, temperature
- IPC transport: Unix sockets with JSON-newline protocol
- Session persistence: ~/.local/share/launcher/profiles/
- Analysis engine: Bottleneck detection & recommendations
- CLI tool: `victory-profile` for session browsing/analysis
- Tests: 24/24 passing in release mode

⏳ **Windows Ready for Validation**: Code implemented, needs platform testing
- GetSystemTimes API for CPU metrics
- Named pipes for IPC (\\.\pipe\victory-game-[PID])
- nvidia-smi GPU metrics (CMP 170HX)
- Overlay file output to C:\temp\victory-overlay.txt

## Prerequisites

### Windows Machine
- **GPU**: NVIDIA CMP 170HX (40GB VRAM) or equivalent
- **CPU**: 9950X or similar
- **RAM**: 32GB DDR5+
- **Build**: NVIDIA driver installed, nvidia-smi available on PATH

### Tools
```bash
# Windows
- Rust toolchain (stable)
- cargo build/test
- nvidia-smi command-line
```

## Step 1: Build on Windows

```bash
cd cool_fox

# Full workspace build (includes victory-profile CLI)
cargo build --release

# Or just profiling components
cargo build -p victory-gpu --release
cargo build -p victory-core --release
cargo build -p victory-profile --release
```

Expected output:
```
   Compiling victory-gpu v0.1.0
   Compiling victory-core v0.1.0
   Compiling victory-profile v0.1.0
    Finished `release` profile [optimized] target(x64-pc-windows-msvc) in Xs
```

## Step 2: Validate CPU Metrics (GetSystemTimes API)

```bash
# Run Windows-specific CPU tests
cargo test -p victory-gpu windows_tests::test_getsystemtimes_api --release -- --nocapture

# Should output:
# ✓ GetSystemTimes API working
#   Idle: <value>
#   Kernel: <value>
#   User: <value>
```

This validates that Windows CPU measurement via GetSystemTimes works on your hardware.

## Step 3: Validate GPU Metrics

```bash
# First verify nvidia-smi is accessible
nvidia-smi --query-gpu=memory.total,memory.used,utilization.gpu,temperature.gpu --format=csv,noheader,nounits

# Should show:
# 40960, <used>, <util>, <temp>

# Run GPU metrics test
cargo test -p victory-gpu windows_tests::test_gpu_metrics_windows --release -- --nocapture

# Should output:
# ✓ Windows GPU metrics collected
#   GPU Load: XX.X%
#   VRAM: XXXX/40960 MB
#   Temp: XX.X°C
```

## Step 4: Validate Named Pipes IPC

```bash
# Test named pipe creation
cargo test -p victory-gpu windows_tests::test_named_pipe_creation --release -- --nocapture

# Should output:
# ✓ Named pipe path: \\.\pipe\victory-game-<PID>
```

For end-to-end IPC testing, you need an actual game/workload:

```bash
# Create test program that uses profiler
# Set environment variable
set VICTORY_PROFILE=1

# Run your game/benchmark
# Monitor named pipe traffic with ProcessMonitor or similar
# Verify data flows: game → named pipe → launcher
```

## Step 5: Test Session Management

```bash
# Run game-integration-test to verify full pipeline
cargo test -p victory-core test_game_process_profiling --release -- --nocapture

# Should show:
# ✅ Session file created
# ✅ Analysis generated
# ✅ Metrics valid
# ✅ Bottleneck detected
```

## Step 6: Use victory-profile CLI

```bash
# List all saved sessions
.\target\release\victory-profile.exe list

# Expected output:
# 📊 Profiling Sessions (N)
# Game | Timestamp | Frames | FPS Avg | GPU Load | Temp
# ...

# Analyze specific session
.\target\release\victory-profile.exe analyze 1

# Export to CSV for analysis
.\target\release\victory-profile.exe export <session-path> output.csv
```

## Step 7: Full Integration Test

Create a simple benchmark to test the complete pipeline:

```bash
# 1. Set environment
set VICTORY_PROFILE=1
set VICTORY_OVERLAY=1

# 2. Run a game/workload for 30-60 seconds
# (Any GPU-intensive application)

# 3. Verify outputs
#    - Check C:\temp\victory-overlay.txt for metrics
#    - Check %LOCALAPPDATA%\launcher\profiles\ for sessions
#    - Run victory-profile analyze to see results

# 4. Expected output format in overlay file:
# 60.0 fps │ GPU 85% 72°C │ CPU 50% │ VRAM 29/40MB│ OK
# 59.5 fps │ GPU 87% 73°C │ CPU 51% │ VRAM 30/40MB│ OK
```

## Validation Checklist

- [ ] **CPU Metrics**: GetSystemTimes returns valid load percentages
- [ ] **GPU Metrics**: nvidia-smi queries work, CMP 170HX shows 40GB VRAM
- [ ] **Named Pipes**: Data flows from game to launcher
- [ ] **Session Files**: ~/.local/share/launcher/profiles/ populated
- [ ] **Analysis**: Bottleneck detection identifies GPU/CPU/VRAM constraints
- [ ] **Overlay Output**: C:\temp\victory-overlay.txt shows real-time metrics
- [ ] **CLI Tool**: victory-profile lists, analyzes, exports sessions
- [ ] **Performance**: <1ms per-frame overhead

## Troubleshooting

### GetSystemTimes fails
- Requires Windows 7+
- API declaration is in `crates/victory-gpu/src/metrics.rs`
- Check your Rust windows-sys dependency version

### nvidia-smi not found
- Install NVIDIA driver
- Add to PATH: typically `C:\Program Files\NVIDIA Corporation\NVIDIA GPU Computing Toolkit\CUDA\bin`
- Run `where nvidia-smi` to verify

### Named pipes error
- Verify process ID in pipe name format: `\\.\pipe\victory-game-<PID>`
- Check Windows permissions for creating named pipes
- Use ProcessMonitor to debug

### No overlay file created
- Verify VICTORY_OVERLAY=1 environment variable
- Check C:\temp\ exists and is writable
- Look for %TEMP%\victory-overlay.txt alternative

## Performance Expectations

On CMP 170HX (9950X, 32GB DDR5):

| Metric | Expected |
|--------|----------|
| Profiler overhead | <1ms per frame |
| JSON serialization | <100μs per frame |
| IPC latency | <500μs |
| Session analysis | <100ms (600 frames) |
| Memory (10K frames) | ~50MB |

## Next Steps After Validation

1. **Overlay Rendering**: Implement D3D12 hook for in-game HUD
   - Location: `crates/victory-gpu/src/overlay.rs`
   - Render metrics directly to game frame buffer

2. **Tauri UI Dashboard**: Build post-game report viewer
   - Location: Planned for `apps/launcher-ui-web`
   - Display historical sessions, trends, bottleneck analysis

3. **Game Integration**: Package for distribution
   - Launcher executable
   - Configuration file for game detection
   - Automatic profiling on game launch

## Files Reference

| File | Purpose |
|------|---------|
| `crates/victory-gpu/src/profiler.rs` | Frame collection & ring buffer |
| `crates/victory-gpu/src/metrics.rs` | System metrics (GPU/CPU/VRAM) |
| `crates/victory-core/src/profiler_client.rs` | IPC client for launcher |
| `crates/victory-core/src/session_persistence.rs` | Save/load sessions |
| `crates/victory-core/src/session_analysis.rs` | Bottleneck detection |
| `apps/victory-profile/src/main.rs` | Working CLI tool |
| `crates/victory-gpu/src/windows_validation.rs` | Windows-specific tests |

## Commands Cheat Sheet

```bash
# Build
cargo build --release

# Test Linux
cargo test --release

# Test Windows (run on Windows machine)
cargo test -p victory-gpu windows_tests --release -- --nocapture

# Benchmark
cargo run -p profile-demo --release

# CLI
./target/release/victory-profile list
./target/release/victory-profile analyze 1
./target/release/victory-profile export <path> output.csv
```

## Support

- **Linux builds**: ✅ Working in this environment
- **Windows validation**: 🖥️ Run the tests above on your Windows machine
- **Issues**: Document platform-specific findings in `WINDOWS_VALIDATION.md`

---

**Status**: Ready for Windows platform validation and integration testing.
