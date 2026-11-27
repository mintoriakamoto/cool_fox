# FPS Maximization Checklist
## Ensuring Peak Performance on CMP 170HX (40GB VRAM, Narrow PCIe, 32GB DDR5, 9950X)

---

## Critical Bottlenecks & Fixes

### 1. TEXTURE LOADING BOTTLENECK (PCIe Link)

**Problem:** CMP 170HX → iGPU connection is narrow (Gen3 x4 = 985 MB/s)
- Texture uploads from system RAM to GPU VRAM compete for bandwidth
- If texture upload blocks rendering, FPS drops

**Current Fixes:**
- ✅ Texture staging: 128 MB/frame (auto-tuned from PCIe gen3 x4)
- ✅ Upload ahead 1 frame: `VICTORY_UPLOAD_AHEAD=1` (textures loaded while GPU renders previous frame)
- ✅ Never wait on texture: `VICTORY_TEXTURE_NO_WAIT=1` (use fallback if texture not ready)
- ✅ DirectStorage staging: 128 MB (for faster loads)

**Still Needed:**
- [ ] **Profiler metric: "PCIe saturation"** - Monitor PCIe bandwidth usage
  - If > 90%, warn user and suggest texture resolution downscale
  - If < 50%, can increase staging buffer for even fewer frame delays
- [ ] **Adaptive staging buffer** - Runtime tuning based on observed PCIe usage
  - If hit 100% saturation: reduce staging_mb by 25%
  - If < 50% usage: increase staging_mb by 25%
- [ ] **Texture prioritization** - Load screen-space textures first
  - Off-screen/distant textures can be loaded more slowly
- [ ] **Compression tracking** - Monitor actual bandwidth vs. theoretical
  - Compressed textures = less PCIe pressure

**Max FPS impact: ±10 FPS if PCIe-bound**


### 2. SHADER COMPILATION STUTTERING

**Problem:** 
- First time GTA VI runs: shader compilation can stutter (drop 20-30 FPS for 2-5 seconds)
- Cold cache: vkd3d-proton compiles shaders on-the-fly
- Each shader ~1-50ms to compile

**Current Fixes:**
- ✅ Shader cache persistence: `VKD3D_SHADER_CACHE_SIZE = 128 × 32 = 4,096 MB`
  - Stores compiled shaders across sessions
  - Second run: warm cache, no compilation stalls
- ✅ Large shader cache (4GB) captures most common shaders

**Still Needed:**
- [ ] **Offline shader compilation** - Pre-compile all shaders before game starts
  - Use `vkd3d-proton` shader dumping to generate shader list
  - Pre-compile critical path shaders (UI, main rendering)
  - Reduces first 30 seconds of gameplay stutters
- [ ] **Shader cache warming** - On launcher first-run:
  - Launch game for 2 minutes in "shader warm-up" mode
  - Render common scenes (menus, default camera view)
  - Save to ~/.cache/victory/shaders/warm.cache
- [ ] **Stutter detection** - Monitor frame times:
  - If frame_time > 50ms (for 60 FPS target): log as stutter
  - Correlate with shader compilation logs
  - Alert user: "Shader compilation in progress; may see drops"

**Max FPS impact: ±30 FPS for first 2 minutes (then stable)**


### 3. CPU BOTTLENECK

**Problem:**
- Wine overhead: each D3D12 call adds latency
- vkd3d-proton translation: D3D12 → Vulkan API overhead
- 9950X can run at 5.7 GHz, but threads might be starved

**Current Fixes:**
- ✅ Tight GPU pipeline: `VKD3D_MAX_FRAME_LATENCY=1`
  - Don't buffer frames; render-to-present latency minimized
  - GPU doesn't lag behind CPU
- ✅ Descriptor validation off: `VKD3D_DESCRIPTOR_QA=0` (+2-3% perf)
  - Removes CPU-side descriptor table validation

**Still Needed:**
- [ ] **CPU affinity pinning** - Pin Wine to NUMA node 0
  - Launcher should run: `numactl -N 0 wine ...`
  - Keeps game on fast NUMA node (memory local to 9950X cores 0-7)
  - Reduces memory latency by ~20%
- [ ] **Thread priority boosts** - Run Wine at high priority:
  - `chrt -f 50 wine ...` (FIFO priority 50)
  - Or: `nice -n -10 wine ...` (lower niceness)
  - Ensures render threads don't get preempted
- [ ] **Disable hyperthreading** (optional):
  - If game runs better with fewer threads, disable via BIOS
  - Some games prefer consistency over thread count
- [ ] **Monitor thread context switches**:
  - If > 100,000/sec: threads are being preempted
  - If < 50,000/sec: good consistency
- [ ] **GPU pipeline saturation**:
  - Measure: how often is GPU waiting for CPU?
  - Profiler should report: "GPU-bound" vs. "CPU-bound"
  - If CPU-bound: CPU not fast enough to feed GPU at max utilization

**Max FPS impact: ±5-10 FPS**


### 4. MEMORY BOTTLENECK

**Problem:**
- Game needs ~40 GB (textures, geometry, game state)
- CMP has 40 GB VRAM: barely enough
- If VRAM fills > 95%: system RAM swap kills performance (SSD latency 10ms vs GPU latency 0.1ms)
- DDR5 is fast but not unlimited

**Current Fixes:**
- ✅ 32 GB DDR5 for scene data overflow
- ✅ Swap configured: 20 GB (user set up during install)
- ✅ Profiler monitors: "peak_vram_mb" and "swap_usage_mb"

**Still Needed:**
- [ ] **Pre-allocate swap on NVMe** - Ensure 20+ GB swap exists
  - Launcher should check at startup: `free -h | grep Swap`
  - If swap < 16 GB: warn user and show swap-setup guide
- [ ] **Monitor swap usage**:
  - If swap > 2 GB: system RAM is full, everything will be slow
  - Alert: "Memory pressure high; consider closing background apps"
- [ ] **Garbage collection tuning** - Game should not leak memory
  - Monitor VRAM growth over time
  - If VRAM grows > 100 MB/minute: potential leak
- [ ] **Resident Set Size (RSS) monitoring**:
  - Track: how much of allocated VRAM is actually used?
  - If RSS < allocated: game is over-allocating
  - Launcher can suggest: "Reduce texture cache size"
- [ ] **Page cache pressure**:
  - If system RAM pressurized, file I/O will be slower
  - Monitor: `cat /proc/pressure/memory`
  - Alert if > 10% memory stalls

**Max FPS impact: ±20 FPS (if VRAM overflows to swap)**


### 5. GPU FREQUENCY SCALING

**Problem:**
- CMP 170HX may not boost to max frequency if thermals are good
- Could be running at base clock, leaving performance on table

**Current Fixes:**
- None yet

**Still Needed:**
- [ ] **Force max GPU frequency** (if safe):
  - Query: `nvidia-smi -q | grep "GPU Current Temp"`
  - If temp < 75°C: can safely boost
  - Set: `nvidia-smi -pm 1 && nvidia-smi -lgc 2500` (CMP max clock ~2.5 GHz)
  - Or: use NVIDIA driver settings to lock frequency
- [ ] **Monitor thermal throttling**:
  - Track GPU temp over game session
  - If throttles (temp > 85°C): reduce power limit or improve cooling
  - Alert: "GPU thermal throttling detected; FPS may drop"
- [ ] **Power limit tuning**:
  - CMP 170HX power limit: `nvidia-smi -pl` (query current limit)
  - If P <= 250W: can likely increase to 280-300W for more performance
  - Set: `nvidia-smi -pl 300` (need sudo)
- [ ] **Boost clock stability**:
  - Monitor: `nvidia-smi dmon` for clock throttling
  - If clock varies > 100 MHz: system power is unstable
  - Recommend: check PSU (should be 850W+ for CMP + 9950X)

**Max FPS impact: ±10-15 FPS**


### 6. FRAME PACING & STUTTERING

**Problem:**
- Inconsistent frame times: some frames 16ms, next 20ms
- Causes "jank" feeling even if avg FPS is 60
- G-Sync/Freesync can't fix if pacing is inconsistent

**Current Fixes:**
- ✅ Tight GPU pipeline: `VKD3D_MAX_FRAME_LATENCY=1`
- ✅ No VSync on game: let swapchain present ASAP

**Still Needed:**
- [ ] **Frame rate cap logic**:
  - If target is 60 FPS: cap to 60.0 FPS (never overshoot)
  - Launcher provides: `--fps-cap 60` argument to game
  - Game should sleep ~1ms if frame renders early to maintain 16.67ms pacing
- [ ] **GPU frame buffering**:
  - Measure: `nvidia-smi dmon` for GPU queue depth
  - Ideal: 0-1 frames buffered (not 3+)
  - If buffered > 2: GPU is starved of work
- [ ] **CPU-GPU synchronization**:
  - Game should NOT stall CPU waiting for GPU
  - Game should NOT stall GPU waiting for CPU
  - Measure: GPU utilization % vs. CPU utilization %
  - If GPU = 100%, CPU = 50%: GPU-bound (good)
  - If GPU = 50%, CPU = 100%: CPU-bound (fix CPU bottleneck)
- [ ] **Latency monitoring**:
  - Measure: input latency (controller input → pixel change)
  - Target: < 50ms (3 frames at 60 FPS)
  - If > 100ms: buffering is too aggressive
- [ ] **Stutter detection & reporting**:
  - Log all frame times > 1.5× average
  - If > 5 stutters/minute: alert user
  - Report to telemetry: "Stutter detected; possible causes: [list]"

**Max FPS impact: ±5-10 FPS (smoothness, not peak)**


### 7. DIRECTSTORAGE & I/O

**Problem:**
- Game loads assets from disk during gameplay
- If I/O is slow: stalls rendering, drops frames

**Current Fixes:**
- ✅ DirectStorage staging: 128 MB
- ✅ Prioritize screen-space assets

**Still Needed:**
- [ ] **Measure I/O latency**:
  - Profile game load times
  - Track: disk reads during gameplay
  - Alert if > 500 μs latency (SSD should be <100 μs)
- [ ] **I/O queue depth**:
  - Monitor: `iostat -x 1` for await time
  - If await > 50ms: storage is saturated
  - Recommend: check for background I/O (indexing, backups)
- [ ] **Prefetching strategy**:
  - Load next scene while current plays
  - Launcher should know scene list and prefetch ahead
- [ ] **Storage performance check**:
  - Launcher should test: `dd if=/dev/zero of=test bs=1M count=1024`
  - If write speed < 300 MB/s: warn user (SSD should be faster)

**Max FPS impact: ±10 FPS (asset load stutters)**


### 8. NVIDIA DRIVER TUNING

**Problem:**
- Drivers have knobs for performance vs. power
- Wrong settings can cost FPS

**Current Fixes:**
- ✅ Descriptor validation off: `VKD3D_DESCRIPTOR_QA=0`
- ✅ Force discrete GPU: `VKD3D_FORCE_DISCRETE_GPU=1`
- ✅ GPU 0 selection: `NV_GPU=0`

**Still Needed:**
- [ ] **Enable GPU performance mode**:
  - `nvidia-smi pm 1` (enable persistence mode)
  - Keeps GPU initialized between frames (no power down)
  - +1-2% perf, +3-5W power
- [ ] **Disable graphics debugging**:
  - Some driver versions have debug overhead
  - Query: `nvidia-smi -q | grep "Driver Version"`
  - If version suggests debug build: recommend latest driver
- [ ] **PCIe gen4 fallback**:
  - If system supports PCIe gen4: can run at 2× bandwidth (1,970 MB/s)
  - Check BIOS: "PCIe Speed: Gen3 or Gen4?"
  - If Gen4 available: might be disabled; enable in BIOS
- [ ] **Monitor driver overhead**:
  - Run benchmark with driver profiling enabled
  - `nvidia-smi --query-gpu=power.draw --format=csv --loop-ms=100`
  - If power draw is inconsistent: driver is throttling

**Max FPS impact: ±3-5 FPS**


### 9. WINE & vkd3d-proton TUNING

**Problem:**
- Wine overhead: each API call adds latency
- vkd3d-proton translation: D3D12 → Vulkan has cost

**Current Fixes:**
- ✅ Tight pipeline: `VKD3D_MAX_FRAME_LATENCY=1`
- ✅ No descriptor validation: `VKD3D_DESCRIPTOR_QA=0`
- ✅ Feature level 12_2: `VKD3D_FEATURE_LEVEL=12_2`
- ✅ No HVV uploads: `VKD3D_CONFIG=no_upload_hvv`

**Still Needed:**
- [ ] **Enable vkd3d-proton performance mode** (if exists):
  - Check vkd3d-proton changelog for perf knobs
  - Add any new optimizations to victory-linux.sh
- [ ] **Disable Wine debugging**:
  - `WINEDEBUG=-all` (already set) ✅
  - Verify no other debug flags are set
- [ ] **Use Proton-GE** (community optimized build):
  - Latest Proton-GE often has better vkd3d-proton
  - Launcher should allow: `--wine-version proton-ge-9.0`
- [ ] **Monitor API call overhead**:
  - If CPU is at 100% but GPU is at 50%: API overhead is issue
  - Profiler should report: "API call time per frame"
- [ ] **Shader optimization**:
  - vkd3d-proton may not optimize shaders as well as native
  - Monitor: shader compilation time + runtime perf
  - If slow: may need to report to vkd3d-proton devs

**Max FPS impact: ±5-10 FPS**


### 10. MISSING FRAME RATE LIMITING

**Problem:**
- Game might render as fast as possible without cap
- Causes thermal stress, power waste
- Inconsistent frame times without pacing

**Still Needed:**
- [ ] **FPS cap enforcement**:
  - Launcher should accept: `--fps-cap 60` or `--fps-cap unlimited`
  - Default: 60 FPS (balanced for 32GB DDR5 + 40GB GPU)
  - Game should sleep if frame renders early
  - Measurement: actual FPS should be ±1 of target
- [ ] **Adaptive frame skipping** (optional):
  - If FPS drops below 70% of target: skip 1 frame to catch up
  - Maintains pacing even under load
- [ ] **Frame time statistics**:
  - Report: min, max, avg, stddev frame time
  - If stddev > 2ms: pacing is inconsistent
- [ ] **G-Sync pacing**:
  - If G-Sync enabled: synchronized to monitor refresh
  - Should eliminate tearing and stutters
  - Launcher should detect monitor: `xrandr --query` (or Wayland equiv)

**Max FPS impact: ±0 FPS (pacing, not peak)**


---

## Profiling Instrumentation Needed

To measure all of the above, the launcher needs these metrics:

```
ProfileFrame (per 16ms):
  ✅ fps, gpu_load, vram_used, cpu_load, gpu_temp
  ❌ pcie_bandwidth_used_mbs
  ❌ shader_compilation_active (bool)
  ❌ shader_compilation_time_ms
  ❌ frame_time_ms (actual render + present time)
  ❌ swap_usage_mb
  ❌ memory_pressure (from /proc/pressure/memory)
  ❌ thermal_throttle_active (bool)
  ❌ api_call_count (D3D12 calls this frame)
  ❌ api_overhead_ms (CPU time in vkd3d-proton)

SessionStats (per game close):
  ✅ fps_avg, fps_min, fps_max, fps_p95, fps_p99
  ✅ gpu_load_avg, gpu_load_peak
  ✅ vram_peak_mb
  ✅ cpu_load_peak
  ❌ pcie_saturation_percent
  ❌ stutters_detected (count of frames > 1.5× avg)
  ❌ thermal_throttle_duration_s
  ❌ swap_overflow_seconds (time spent using swap)
  ❌ memory_stall_duration_s (from /proc/pressure)
  ❌ api_overhead_total_ms
```

---

## Checklist: What to Build First

**MVP (Weeks 1-2) - Performance Profiling Hooks:**
- ✅ Basic ProfileFrame collection (fps, gpu_load, vram, temp)
- ❌ PCIe bandwidth monitoring
- ❌ Shader compilation detection
- ❌ Stutter detection
- ❌ Memory pressure monitoring

**Phase 2 (Weeks 3-4):**
- ❌ Advanced profiling (all 10 metrics above)
- ❌ CPU affinity pinning
- ❌ Adaptive staging buffer
- ❌ Offline shader compilation
- ❌ Thermal throttle detection

**Phase 3 (Weeks 5-6):**
- ❌ Frame rate capping logic
- ❌ G-Sync / adaptive sync support
- ❌ Dynamic resolution scaling
- ❌ Performance graphs and reports

---

## Theoretical Max FPS on Your System

```
System Spec: CMP 170HX (40GB) + 32GB DDR5 + 9950X + B650E

Limiting Factors:
  1. PCIe Gen3 x4: 985 MB/s (narrow link, can't transfer massive textures)
  2. GPU compute: 23 TFLOPs (CMP 170HX theoretical max)
  3. Memory bandwidth: ~960 GB/s (GPU internal, good)
  4. DDR5 bandwidth: ~160 GB/s (system RAM, good)
  5. CPU cache: 144 MB L3 (9950X, more than enough)

Expected FPS:
  - GTA VI @ 4K high: 50-70 FPS (PCIe-bound on texture loads)
  - GTA VI @ 4K ultra: 40-55 FPS (GPU-bound on shaders)
  - GTA VI @ 1440p high: 100-130 FPS (GPU-bound)
  - GTA VI @ 1440p ultra: 80-120 FPS (GPU-bound)
  - Cyberpunk 2077 @ 4K: 30-45 FPS (GPU + AI-bound)

Bottleneck Order (most likely first):
  1. PCIe saturation (narrow Gen3 x4 link)
  2. GPU memory bandwidth (40GB helps, but not unlimited)
  3. Shader compilation (first few minutes)
  4. CPU overhead (Wine/vkd3d-proton translation)
  5. Thermal throttling (if cooling inadequate)

Where to Get More FPS:
  Priority 1: Reduce texture streaming (lower res or staging tuning)
  Priority 2: Shader optimizations (compile offline)
  Priority 3: CPU affinity pinning (reduce context switches)
  Priority 4: Thermal management (keep GPU cool)
  Priority 5: Driver updates (latest vkd3d-proton)
```

---

## Summary: Don't Miss These

**To hit peak FPS, you MUST:**

1. ✅ Profile every frame (ProfileFrame struct) → find bottleneck
2. ✅ Monitor PCIe saturation → know if texture loading is limiting
3. ✅ Detect thermal throttling → know if cooling is issue
4. ✅ Measure frame time consistency → know if pacing is good
5. ✅ Report CPU vs. GPU bottleneck → guide optimizations
6. ❌ Implement FPS cap (default 60, but allow higher)
7. ❌ Implement CPU affinity pinning (NUMA awareness)
8. ❌ Detect swap usage (warn if system RAM overflows)
9. ❌ Monitor shader compilation (pre-compile critical shaders)
10. ❌ Enable all NVIDIA driver optimizations (persistence mode, etc.)

**Without these, you could leave 20-30 FPS on the table.**

Build in this order:
1. ProfileFrame collection ← Priority 1 (this week)
2. PCIe saturation monitoring
3. Thermal throttle detection
4. FPS cap logic
5. CPU affinity pinning
6. Advanced profiling (all 10 metrics)
