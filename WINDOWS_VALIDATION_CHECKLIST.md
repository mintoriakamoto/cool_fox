# Windows Validation Checklist

## CMP 170HX Hardware Specifications
- **GPU:** NVIDIA CMP 170HX (40GB VRAM)
- **CPU:** AMD Ryzen 9950X
- **RAM:** 32GB DDR5
- **Platform:** Windows 11/Server 2022

## Phase 1: Metrics Collection Validation

### GPU Metrics (via nvidia-smi)
```bash
# Verify on CMP 170HX
nvidia-smi -L
nvidia-smi -q -d MEMORY
nvidia-smi -q -g 0 --format=csv.noheader
```

**Expected on CMP 170HX:**
- [ ] `nvidia-smi` queries run without errors
- [ ] VRAM total shows ~40000 MB
- [ ] GPU load percentage updates in real-time
- [ ] Temperature readings available (or n/a for datacenter GPU)
- [ ] Power usage reported accurately

### CPU Metrics (Windows GetSystemTimes API)
**Current code location:** `crates/victory-gpu/src/metrics.rs:query_cpu_load_windows()`

```rust
// MUST VERIFY:
unsafe {
    GetSystemTimes(
        &mut idle_time,
        &mut kernel_time, 
        &mut user_time,
    )
}
```

**Test:**
- [ ] CPU load percentage updates every frame
- [ ] Matches Task Manager's CPU usage
- [ ] Handles multi-threaded workload correctly

### VRAM Metrics
**Current code:** Reads from nvidia-smi parse

**Test with actual game:**
- [ ] VRAM used increases as game loads assets
- [ ] VRAM doesn't spike unexpectedly
- [ ] VRAM tracking survives long gameplay sessions

## Phase 2: IPC Communication Validation

### Named Pipes (Windows)
**Code location:** `crates/victory-gpu/src/profiler.rs::NamedPipeWriter`

```
\\.\pipe\victory-game-[PID]
```

**Test checklist:**
- [ ] Launcher creates named pipe successfully
- [ ] Game connects to pipe without blocking
- [ ] JSON frames serialize and transmit without corruption
- [ ] Launcher receives 60+ frames/sec at 60 FPS gameplay
- [ ] Pipe survives game crash (graceful close)
- [ ] Multiple games can run simultaneously (different PIDs)

### Frame Serialization
**Expected frame JSON:**
```json
{
  "timestamp_ms": 1000,
  "fps": 60.0,
  "gpu_load_percent": 85.0,
  "vram_used_mb": 30000,
  "vram_total_mb": 40960,
  "cpu_load_percent": 45.0,
  "gpu_temp_celsius": 72.0,
  "frame_time_ms": 16.667,
  "alerts": []
}
```

**Validate:**
- [ ] JSON parses without errors on launcher side
- [ ] All fields present and valid ranges
- [ ] No NaN or Infinity values
- [ ] Timestamps monotonically increase

## Phase 3: Gaming Session Validation

### Test with Real Game Workload

**Recommended test games (DirectX 12):**
1. **Unreal Engine 5 Project:**
   - Create simple UE5 scene with GPU-intensive rendering
   - Run at 1440p ultra settings
   
2. **Standalone DX12 Test:**
   - Use Microsoft's D3D12 sample from GitHub
   - Render at various complexities

3. **Professional Benchmark:**
   - GFXBench DirectX 12 (if available for Windows)
   - FrameView (from NVIDIA)

### During Gameplay Session
- [ ] Profiler initializes without errors
- [ ] Named pipe connection established
- [ ] Frames collected continuously for 5+ minutes
- [ ] No frame drops or stuttering visible
- [ ] Metrics in reasonable ranges (see below)
- [ ] Session saved to `~/.local/share/launcher/profiles/`

### Expected Metrics Ranges (CMP 170HX 40GB)

| Metric | Min | Typical | Max | Notes |
|--------|-----|---------|-----|-------|
| **FPS** | 1 | 60-120 | 240+ | Depends on game/settings |
| **GPU Load %** | 0 | 50-95 | 100 | 0 = idle, 100 = bottleneck |
| **VRAM Used MB** | 100 | 10000-30000 | 40000 | Datacenter GPU, allocate what needed |
| **CPU Load %** | 0 | 20-50 | 100 | Multi-core 9950X |
| **GPU Temp °C** | 20 | n/a | n/a | CMP 170 often disabled (datacenter) |
| **Frame Time ms** | 4-8 | 8-16 | 30+ | 1000/FPS |

## Phase 4: Bottleneck Detection Validation

### Test Cases

**GPU-Bound Scenario:**
- [ ] High GPU% (>90%), Low CPU% (<50%)
- [ ] Detection: "GPU-bound" ✓

**CPU-Bound Scenario:**
- [ ] High CPU% (>90%), Low GPU% (<80%)
- [ ] Detection: "CPU-bound" ✓

**VRAM-Critical Scenario:**
- [ ] Load game requiring >38GB VRAM
- [ ] Expected: "VRAM-critical" alert ✓

**Thermal-Limited (if temps available):**
- [ ] GPU temp > 85°C (if readings available)
- [ ] Expected: "Thermal-limited" alert ✓

**Balanced:**
- [ ] Moderate loads on all resources
- [ ] Detection: "Balanced" ✓

## Phase 5: Dashboard/Analysis Validation

### Web Server Test
```bash
./target/release/launcher-web
# Browse to http://localhost:8080
```

- [ ] Server starts without errors
- [ ] Sessions list shows game(s) from disk
- [ ] Can click session to open dashboard
- [ ] Metrics charts render correctly
- [ ] Real session data displayed (not sample data)

### Analysis Report Test
```bash
./target/release/launcher-ui analyze [session-name]
```

- [ ] Report generates without errors
- [ ] Bottleneck analysis matches actual gameplay
- [ ] Recommendations are relevant

### Session Export Test
```bash
./target/release/launcher-ui export [session-name] output.csv
```

- [ ] CSV file created
- [ ] All frames exported
- [ ] Data imports into Excel/Python correctly

## Phase 6: D3D12 Overlay Validation (Windows only)

### Present Hook Interception
- [ ] Hook successfully patches IDXGISwapChain::Present
- [ ] Game still runs smoothly
- [ ] No crashes or access violations

### Metrics Pipe Reading In-Game
- [ ] Overlay code reads from named pipe
- [ ] Gets latest ProfileFrame each present call
- [ ] No blocking reads (uses non-blocking I/O)

### Text Rendering
- [ ] "FPS: 120" visible in-game corner
- [ ] "GPU: 85% 72°C" updates in real-time
- [ ] "CPU: 45%" accurately reflects load
- [ ] Text readable at 1440p+ resolutions

### Performance Impact
- [ ] Frame time increase < 1ms
- [ ] No frame drops caused by overlay
- [ ] No GPU stalls observed

## Phase 7: Stress Testing

### Extended Session (1+ hour)
- [ ] No memory leaks in launcher
- [ ] No memory leaks in game
- [ ] Named pipe stays connected
- [ ] Session file size stays reasonable
- [ ] Dashboard still responsive

### Many Frames (10,000+)
- [ ] Ring buffer handles capacity
- [ ] Old frames properly evicted
- [ ] Session analysis still fast

### Multiple Games Simultaneously
- [ ] Run 2+ games with profiler active
- [ ] Each creates own pipe (different PID)
- [ ] Launcher sees all sessions
- [ ] No interference between sessions

## Phase 8: Error Handling

### GPU Disconnection
- [ ] nvidia-smi fails gracefully
- [ ] Game still runs (just no GPU metrics)
- [ ] Launcher shows "GPU metrics unavailable"

### Named Pipe Failure
- [ ] Game detects broken pipe
- [ ] Reconnects automatically
- [ ] No data loss on reconnection

### Game Crash
- [ ] Launcher detects unexpected close
- [ ] Session saved with final metrics
- [ ] No dangling processes

### Insufficient VRAM
- [ ] Profiler handles VRAM exhaustion
- [ ] No OOM crashes
- [ ] Alert generated for VRAM-critical

## Phase 9: Compatibility Testing

### Game Engines
- [ ] Unreal Engine 5 (D3D12)
- [ ] Unity 2022+ (D3D12)
- [ ] Custom D3D12 app

### Display Configurations
- [ ] Single monitor at 1440p
- [ ] Dual monitors
- [ ] 4K display (if available)

### DirectX Versions
- [ ] DirectX 12 Feature Level 12_0
- [ ] DirectX 12 Feature Level 12_1

## Validation Report Template

```markdown
# CMP 170HX Validation Report
Date: [DATE]
Tester: [NAME]

## Metrics Collection
- GPU metrics: [PASS/FAIL]
- CPU metrics: [PASS/FAIL]
- VRAM tracking: [PASS/FAIL]

## IPC Communication
- Named pipes: [PASS/FAIL]
- Frame transmission: [PASS/FAIL]
- Session persistence: [PASS/FAIL]

## Bottleneck Detection
- GPU-bound: [PASS/FAIL]
- CPU-bound: [PASS/FAIL]
- VRAM-critical: [PASS/FAIL]

## Dashboard
- Web server: [PASS/FAIL]
- Real data display: [PASS/FAIL]
- Analysis accuracy: [PASS/FAIL]

## D3D12 Overlay
- Present hook: [PASS/FAIL]
- Text rendering: [PASS/FAIL]
- Performance: [PASS/FAIL]

## Stress Testing
- 1+ hour session: [PASS/FAIL]
- 10,000+ frames: [PASS/FAIL]
- Multiple games: [PASS/FAIL]

## Overall Status
[ ] ✅ PRODUCTION READY
[ ] ⚠️  NEEDS FIXES
[ ] ❌ NOT READY

## Notes
[Any issues, workarounds, or observations]
```

## Success Criteria

**Minimum (MVP):**
- ✅ Metrics collected without errors
- ✅ Named pipes transmit frames
- ✅ Session saved and can be analyzed
- ✅ Bottleneck detection works

**Full (Production-Ready):**
- ✅ All above, PLUS:
- ✅ D3D12 overlay renders in-game
- ✅ Performance overhead < 1ms
- ✅ Works with popular game engines
- ✅ Dashboard displays real data accurately
- ✅ No crashes or memory leaks under extended use

## Known Limitations (Pre-Windows Testing)

1. **GPU Temperature:** CMP 170HX typically has no temp sensor in datacenter mode
   - Workaround: Show "N/A" or use estimated values
   
2. **Power Metrics:** Not collected by nvidia-smi in all configurations
   - Workaround: Show only available metrics

3. **Game Compatibility:** Some games with custom Present implementations may not work
   - Workaround: Document known incompatibilities

4. **Anti-Cheat:** Some games block DLL injection (EAC, BattlEye)
   - Workaround: Test with games that allow profiling tools
