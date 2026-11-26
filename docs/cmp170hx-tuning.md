# Tuning for CMP 170HX + 32GB DDR5 + 9950X

This document tracks optimizations done and remaining for your specific system.

## System Profile
- **GPU:** NVIDIA CMP 170HX (40 GB VRAM, no display engine)
- **CPU:** AMD Ryzen 9950X (Zen 5, 16-core)
- **RAM:** 32 GB DDR5
- **Display:** Motherboard iGPU HDMI (PRIME render offload to CMP)
- **PCIe:** B650E (gen4 x16, typical ~16 GB/s per direction)
- **Storage:** NVMe for swap (must be real storage, not zram)

## Completed Tuning

### Launch Environment (Auto-tuned)
Both launchers now read your system specs and auto-compute:
- **Texture staging:** Reads PCIe link gen/width from `nvidia-smi`, computes texture upload per frame
  - Your B650E gen4 x16: ~32 GB/s link → ~64 MiB texture stage per frame
  - Clamped 16–128 MiB; set once at launch via `VICTORY_TEXTURE_STAGE_MB`
- **DirectStorage staging:** 128 MiB (your RAM is ≤34 GB)
- **FMA switch:** Auto-detects CMP, disables (emits separate mul+add instead of fmad)
- **Belobug ports:** All enabled except `VICTORY_PORT_CBV` (untested on NVIDIA)
- **Warnings:** Checks VRAM, swap size, card detection; reports to stderr at startup

**Files:**
- `victory-linux --print-env` (Linux)
- `Play-VIctory.ps1 --print-env` or `Play-VIctory.cmd` (Windows)

### CPU-Side Optimizations
- `VICTORY_UPLOAD_AHEAD=1`: Queue texture uploads ahead of GPU work
- `VICTORY_TEXTURE_NO_WAIT=1`: Never stall for texture readback (narrow PCIe)
- `VICTORY_LEVELS_ON_DEMAND=1`: Stream LOD textures as needed
- `VICTORY_CARD_BUFFERS_MIN_KB=64`: Small buffers to fit in GPU cache
- Shader cache key includes `HOST_NO_FMA` state (clear cache between FMA changes)

### Shader Tuning
- **FMA disable:** `VICTORY_NO_FMA=1` (default for CMP) → fmad(a,b,c) becomes mul(a,b) + add(result,c)
  - Both ops emitted without fast-math flags (DXIL strict mode)
  - Expected to prevent driver fusion on CMP's allegedly throttled FP32 FMA
  - **Status:** Implemented, unverified on hardware
- **No ray tracing:** `-RayTracing` console flag (CMP has no RT hardware)
- **Shader tests:** 210 tests pass on Linux; nothing ran on GPU

### Memory Profiling
- Title uses ~40–46 GB private memory on a 64 GB system
- Your 32 GB + swap strategy:
  - Set `--console series-s` (default in both launchers) to reduce memory footprint
  - Add 16+ GB of swap on a fast NVMe (not zram or tmpfs)
  - Launcher warns if swap < 16 GB when RAM ≤ 34 GB

### Renderer Ports (Belobug's AMD optimizations, enabled by default)
Measure performance with and without each by setting them to `=0`:
- `VICTORY_PORT_CONSTANTS=1`: CPU-side byte-patched const buffers (measured on AMD RDNA)
- `VICTORY_PORT_DESCRIPTORS=1`: Descriptor patching
- `VICTORY_PORT_GUARD=1`: Memory guard pages
- `VICTORY_PORT_JOURNAL=1`: Journal tracking
- `VICTORY_PORT_CBV=0` (off): GPU-side raw byte const buffer reads (untested on NVIDIA/CMP)

## Known Limitations

### Hardware
- **No ray tracing:** CMP 170HX has no RT cores; `-RayTracing` already off
- **Narrow PCIe:** Gen4 x16 is 16 GB/s bidirectional; textures must stay resident once uploaded
- **No display engine:** Rendering to scan out via iGPU (PRIME offload) adds latency and bus traffic
- **FP32 FMA throttling:** Community reports, unverified here (the `VICTORY_NO_FMA` switch was implemented to test this)

### GPU Dispatch Limits (D3D12 Hard Limit on NVIDIA)
- Indirect dispatches clamped to 65,535 groups per dimension (X, Y, Z)
- AMD cards can exceed this; workarounds (CPU-side splitting) exist but are complex
- **Impact:** Unknown; depends on how many title compute passes actually exceed this

### Movie Plane & Synchronous Bake Uploads
- Dropped during Belobug merge (his texture upload in submission prelude replaces them)
- **Status:** Lost feature; impact on video/bake quality unknown
- Would require significant refactor to restore

### Not Yet Run
- **On GPU:** No runs on the CMP card yet; all measurements are cross-checks and estimates
- **FMA switch:** Not measured against title on CMP; unverified whether it helps or hurts
- **Belobug ports:** Developed on AMD RDNA only; `VICTORY_PORT_CBV` is completely untested on NVIDIA/CMP
- **Windows launcher:** Not run on Windows yet; PowerShell auto-tuning is new

## Measuring Performance

Run these in order to isolate what helps:

1. **Baseline:** Run with all defaults (launchers enable most things)
   ```
   victory-linux  # or Play-VIctory.cmd on Windows
   ```
   Record: frame time, VRAM usage, GPU load, CPU load.

2. **FMA switch:** Test with and without
   ```
   VICTORY_NO_FMA=1 victory-linux  # test with separate mul+add
   VICTORY_NO_FMA=0 victory-linux  # test with fmad (fused if driver allows)
   ```
   Clear shader cache between runs: `rm -rf ~/.cache/victory/shaders` or `%LOCALAPPDATA%\VIctory\cache-pareto`

3. **Belobug ports:** Disable each one at a time (default all =1 except CBV=0)
   ```
   VICTORY_PORT_CONSTANTS=0 VICTORY_PORT_DESCRIPTORS=1 ... victory-linux
   ```

4. **Texture staging:** Increase to test sensitivity to upload bandwidth
   ```
   VICTORY_TEXTURE_STAGE_MB=256 victory-linux
   ```

5. **DirectStorage staging:** Test with 128 MiB and 256 MiB
   ```
   VICTORY_DSTORAGE_STAGING_MB=256 victory-linux
   ```

6. **Shader variants:** Disable by running with `VICTORY_PORT_CBV=1` (enable untested GPU-side raw reads)
   ```
   VICTORY_PORT_CBV=1 victory-linux  # test at your own risk
   ```

## CI and Builds

- `.github/workflows/ci.yml` is configured: `cross-check` (Linux type-checks for Windows), `build-exe` (Windows exe), `deb` (Linux package)
- **Status:** Workflow file exists but CI has never run (GitHub Actions may need enabling in repo settings)
- Windows .exe built in CI is signed into the .deb; nothing signed here yet

## Next Steps (If Performance Remains Poor)

1. **Enable GitHub Actions** in your repo settings so CI can run and catch regressions
2. **Profile the first real run** with `perf` or Windows Performance Toolkit to find actual bottlenecks
3. **Test FMA switch on hardware** to confirm whether the community report (FMA throttling) is real for your card
4. **Consider native Linux port** if Wine overhead (PRIME offload) becomes a bottleneck
   - See `docs/linux-port.md` for the scope; it's a 115k-line job
5. **Share first-run data** (frame times, VRAM/VRAM usage patterns, shader cache size, what actually uses most bandwidth) for further optimization
