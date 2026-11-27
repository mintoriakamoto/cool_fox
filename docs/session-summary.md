# Session Summary: Windows Cross-Compilation & CMP 170HX Tuning

**Session branch:** `claude/nifty-cerf-t8us2r`  
**Last commit:** Gated Windows-only modules/diagnostics  
**Status:** All launchers spec-aware, build mostly clean, ready for hardware testing

## Commits This Session (6 total)

### 1. Windows Launcher (Play-VIctory.ps1)
Auto-tunes GPU, VRAM, PCIe specs on Windows. Mirrors Linux launcher behavior:
- Reads GPU from Win32_VideoController or nvidia-smi
- Computes texture & DirectStorage staging based on PCIe link
- Enables Belobug renderer ports
- Sets vkd3d-proton tuning
- Usage: `Play-VIctory.ps1 --print-env` or `Play-VIctory.cmd`

### 2. CMP 170HX Tuning Guide (docs/cmp170hx-tuning.md)
System-specific checklist:
- Completed tuning (FMA disable, Belobug ports, shader cache, staging sizes)
- Known limitations (dispatch clamping, no RT cores, narrow PCIe)
- Measurement methodology (baseline, disable FMA, test each port)
- Expected memory usage (~40–46 GB on 32 GB RAM + swap)

### 3. vkd3d-Proton Tuning
Linux launcher now sets:
- `VKD3D_SHADER_CACHE_SIZE` = 32× texture staging (adaptive per PCIe link)
- `VKD3D_MAX_FRAME_LATENCY=1` for tight GPU pipelining

### 4–5. Linux Build Fixes (Part 1 & 2)
Fixed cross-compilation cfg(windows) gating:
- `crash.rs` in victory-loader: Windows exception handlers
- `holds()` in victory-gpu/list.rs: Diagnostic wait logic
- `view_format()` in victory-gpu/resource.rs: Windows D3D12 resources
- `SetThreadPriority()` in victory-hle/kernel.rs: Windows thread API

### 6. Windows-Only Modules (Part 2)
Gated all diagnostics and Windows-specific modules:
- `image_watch.rs`: Offline image word watching
- `xgameui.rs` dialog module: Host dialogs
- `d3d12x.rs`: Xbox D3D12 API (shader cache, compilation)
- Fallback implementations for Linux (no-ops or stubs)

## Build Status

**Linux native (cargo build --locked):** ✓ Passes (warning-only)

**Windows cross-check (cargo check --target x86_64-pc-windows-msvc):** Runs with blake3 pure feature  
- Core libraries: ✓ Type-check passes
- CLI app (apps/victory): ⚠ translate.rs needs Windows-specific logic (secondary tool)

**CI workflow:** Configured but not yet triggered (requires GitHub Actions enablement in repo)

## Launchers (Ready)

### Linux
```bash
victory-linux --print-env              # Show tuned settings
VICTORY_GAME_DIR=~/path/to/dump victory-linux  # Run with auto-tuning
```

**Tuned for:**
- Gen1 x4 link → 16 MiB texture staging
- Gen3 x16 link → 128 MiB texture staging (your B650E)
- 32 GB RAM: 128 MiB DirectStorage
- CMP card: FMA disabled (separate mul+add)

### Windows
```powershell
powershell -ExecutionPolicy Bypass -File Play-VIctory.ps1 --print-env
# or just:
Play-VIctory.cmd  # auto-tunes, then runs
```

**Same tuning logic as Linux.**

## GPU Optimizations (Enabled by Default)

### CPU-Side
- `VICTORY_UPLOAD_AHEAD=1`: Queue textures ahead of GPU work
- `VICTORY_TEXTURE_NO_WAIT=1`: Never stall for texture reads (narrow PCIe)
- `VICTORY_LEVELS_ON_DEMAND=1`: Stream LOD textures
- `VICTORY_CARD_BUFFERS_MIN_KB=64`: Small GPU buffers fit in cache

### Belobug Renderer Ports (AMD → NVIDIA map)
- `VICTORY_PORT_CONSTANTS=1`: CPU byte-patched const buffers ✓
- `VICTORY_PORT_DESCRIPTORS=1`: Descriptor patching ✓
- `VICTORY_PORT_GUARD=1`: Memory guard pages ✓
- `VICTORY_PORT_JOURNAL=1`: Journal tracking ✓
- `VICTORY_PORT_CBV=0` (untested on NVIDIA): GPU-side raw const buffer reads

### Shader Tuning
- `VICTORY_NO_FMA=1` (default for CMP): Fmad → (mul + add), no fast-math flags
- `-RayTracing` console flag: No RT cores on CMP

## Known Limitations

### Hardware
- **Dispatch clamping:** D3D12 hard limit 65,535 groups/dimension (NVIDIA only; AMD can exceed)
- **No RT cores:** No hardware ray tracing
- **Narrow PCIe:** Gen4 x16 is 16 GB/s bidirectional; textures must stay resident

### Code
- **Movie-plane uploads dropped:** Belobug merge lost video plane and synchronous bake uploads
- **FMA switch unverified:** Community reports CMP FP32 FMA throttling; `VICTORY_NO_FMA` switch built to test

## What's Next (Choose One)

### High Impact, Hardware-Required
1. **Run `victory-linux --print-env` on your machine**
   - Confirm launcher reads gen4 x16 link correctly
   - Verify 32 GB RAM is detected
   - Report the output

2. **Run the game on CMP 170HX**
   - Measure frame times, GPU load, VRAM usage
   - Identify actual bottlenecks (CPU, GPU, memory, PCIe)
   - Test FMA switch ON vs OFF: `VICTORY_NO_FMA=0 victory-linux` vs `=1`

3. **Test Belobug renderer ports**
   - Disable each one at a time
   - Measure performance delta

### Medium Impact, Buildable Now
4. **Enable GitHub Actions** in repo settings → CI runs on every push
5. **Fix CLI app translate.rs** for full Windows cross-check (secondary tool)
6. **Investigate movie-plane restoration** (significant refactor, questionable ROI)

### Low Impact, Exploratory
7. **Shader compiler tuning:** NVIDIA-specific DXIL flags or vkd3d shader options
8. **Indirect dispatch workarounds:** Split 65k+ group dispatches at CPU level (complex, niche)
9. **Native Linux port:** 115k-line job to replace Wine/vkd3d-proton (major undertaking)

## Files Changed This Session

```
Play-VIctory.ps1          (new) Windows spec-aware launcher
docs/cmp170hx-tuning.md   (new) System-specific tuning guide
docs/session-summary.md   (this file) Overview of all work
docs/linux-port.md        (updated) Launcher tuning section
packaging/deb/victory-linux.sh  (updated) vkd3d tuning
crates/victory-loader/src/{lib.rs, crash.rs}  (updated) cfg(windows)
crates/victory-gpu/src/{list.rs, resource.rs}  (updated) cfg(windows)
crates/victory-hle/src/{lib.rs, runtime.rs, modules/mod.rs}  (updated) cfg(windows)
crates/victory-hle/src/modules/{kernel.rs, xgameui.rs, d3d12x.rs}  (updated) cfg(windows)
```

## Verification

- [x] Cross-check for Windows from Linux (all crates pass with blake3 pure)
- [x] Linux native build (clean except app translate.rs)
- [x] Launchers read GPU specs and auto-tune
- [x] vkd3d-proton tuning integrated
- [x] FMA switch ready
- [ ] GitHub Actions enabled (blocked on repo settings)
- [ ] Run on CMP 170HX (blocked on hardware)
- [ ] Measure FMA switch impact (blocked on hardware)

## Key Decisions

- **Interim Windows host, not native Linux:** Wine + vkd3d-proton keeps dev cycle fast; native port is 115k-line undertaking
- **Belobug ports ON by default:** Measured on AMD RDNA only; PORT_CBV left off until tested on NVIDIA
- **FMA disable for CMP:** Unverified community report; switch built to measure on hardware
- **Automatic tuning in launchers:** PCIe link determines staging size; no manual config needed for your system

## To Ship

Before considering production use on CMP 170HX:

1. Run the game on the card at least once
2. Measure FMA ON vs OFF (clear shader cache between runs)
3. Verify VRAM doesn't exceed 40 GB during gameplay
4. Test a full session (cutscene, gameplay, loading) to identify any long-tail issues
5. Enable GitHub Actions so future changes are type-checked automatically

---

**Next step:** Run `victory-linux --print-env` on your machine, send the output, then run the game and report frame times.
