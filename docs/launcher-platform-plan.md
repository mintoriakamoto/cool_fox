# Game Launcher Platform Plan
## Custom Launcher (Steam-like, Not Steam)

Your own game launcher for Linux + NVIDIA. Scope: complete platform, not just the game.

---

## Core Areas to Build

### 1. **Launcher UI / Frontend**
**Purpose:** User interface for launching games, managing settings, monitoring performance

**Options:**
- **Web-based (Tauri + Leptos/React)** ← Bun fits here for dev
  - Cross-platform (Linux, Windows if needed later)
  - Hot reload during dev
  - Use Bun for bundling/tooling
- **Native GTK3/4** (Linux-native, lightweight)
- **Qt** (heavier but feature-rich)

**Features:**
- [ ] One-click game launch
- [ ] Graphics settings (resolution, texture quality, shader detail)
- [ ] Performance settings (FPS cap, VRAM target, CPU threads)
- [ ] Game library display (recently played, all games, filters)
- [ ] Real-time stats overlay (FPS, GPU load, VRAM, CPU %, temps)
- [ ] Controller detection and remapping
- [ ] Keybinds customization

**Technology:** Tauri + React (or Leptos) + Bun for bundling

---

### 2. **Package & Dependency Management**
**Purpose:** Ensure wine, vkd3d-proton, drivers, libraries are correct versions

**What needs managing:**
- [ ] Wine version selection (staging, proton-ge, proton-next)
- [ ] vkd3d-proton version auto-update
- [ ] NVIDIA driver version detection (warn if < 535)
- [ ] Vulkan layers (vkd3d, NV optimus, debugging layers)
- [ ] System libraries (libvulkan1, libxcb, etc.)

**Implementation:**
```bash
# launcher-cli pkg install wine-ge-proton-9.0
# launcher-cli pkg update vkd3d-proton
# launcher-cli pkg check nvidia-driver
```

---

### 3. **Game Library & Asset Management**
**Purpose:** Organize games, dumps, patches, mods

**Directory Structure:**
```
~/.local/share/launcher/
├── games/
│   ├── gta6/
│   │   ├── game_scarlett_master.exe
│   │   ├── metadata.json (title, version, playtime)
│   │   ├── mods/
│   │   └── saves/
│   └── [other-games]/
├── cache/
│   ├── victory/shaders/
│   └── [game-specific]/
├── prefixes/
│   ├── gta6.prefix/
│   └── [per-game wine prefixes]/
└── config/
    ├── controller-mappings.yaml
    ├── keybinds.yaml
    └── launcher-config.toml
```

**Features:**
- [ ] Auto-detect game directories
- [ ] Game metadata (title, cover art, playtime, last played)
- [ ] Per-game settings (separate Wine prefix, shader cache)
- [ ] Save backup to cloud (optional)
- [ ] Mod manager (enable/disable, load order)

---

### 4. **Performance Profiling & Telemetry**
**Purpose:** Understand what's working, what's not, what to optimize next

**Data to collect:**
- [ ] Launch time (wine overhead, shader compilation)
- [ ] First-frame time (shader cache cold start)
- [ ] FPS over time (min/max/avg, stutters)
- [ ] GPU utilization (GPU load, memory usage, temperature)
- [ ] CPU utilization (per-core, threads spawned)
- [ ] Memory usage (system RAM, swap usage, VRAM)
- [ ] PCIe link saturation (texture upload throughput)

**Output:**
- [ ] Real-time overlay during gameplay
- [ ] Post-game report (CSV, JSON)
- [ ] Historical tracking (compare runs)
- [ ] Automated detection of bottlenecks (CPU-bound? GPU-bound? Memory-bound?)

**Tool:** Use `nvidia-smi` polling + `perf` + custom profiler in victory-gpu

---

### 5. **Auto-Update & Patching**
**Purpose:** Keep game and dependencies current

**Versioning:**
- [ ] Game .exe auto-download when new version available
- [ ] Shader cache invalidation on game update
- [ ] Launcher binary self-update
- [ ] Wine/vkd3d-proton auto-update (or notify user)

**Transport:**
- [ ] Delta updates (only changed files)
- [ ] Resumable downloads
- [ ] Checksum verification (SHA-256)

---

### 6. **Crash Reporting & Debugging**
**Purpose:** Catch failures and understand why

**On crash:**
- [ ] Capture Wine logs, dmesg, dxvk/vkd3d-proton logs
- [ ] Save last N frames of GPU state
- [ ] Gather system info (GPU, driver version, CPU, RAM, kernel)
- [ ] Upload to telemetry service (opt-in)
- [ ] Show user: "What went wrong + what to try"

**Debugging tools:**
- [ ] `--log-level=debug` to dump shader compilation
- [ ] `--record-traces` for GPU timeline captures
- [ ] Wine console window toggle
- [ ] vkd3d-proton debug layer toggle

---

### 7. **Controller & Input Management**
**Purpose:** Support Xbox controllers, remapping, accessibility

**Features:**
- [ ] Auto-detect Xbox 360/One/Series controllers
- [ ] On-screen controller layout (what buttons are bound)
- [ ] Remapping UI (rebind any button)
- [ ] Deadzone calibration
- [ ] Vibration on/off (rumble over Wine)
- [ ] Touch input support (for future handheld)

**Tech:** Use `evdev` or SDL2 gamepad API

---

### 8. **Social / Community**
**Purpose:** Track achievements, compare stats, share clips

**Optional (Phase 2):**
- [ ] Local achievements (unlock system)
- [ ] Leaderboards (speedrun times, high scores)
- [ ] Screenshot capture (Shift+F12)
- [ ] Video recording (last 5 min on demand)
- [ ] Share to YouTube/Twitch
- [ ] In-game chat (future multiplayer)

---

### 9. **System Requirements Check**
**Purpose:** Warn before launching if system is under-spec

**At startup:**
- [ ] Detect GPU (nvidia-smi)
- [ ] Detect driver version
- [ ] Detect VRAM (40 GB recommended, warn if < 16 GB)
- [ ] Detect system RAM (32 GB recommended, warn if < 16 GB)
- [ ] Detect swap (warn if < 16 GB on <= 32 GB RAM)
- [ ] Detect PCIe link (gen4 x16 ideal, warn if gen3 x8 or worse)
- [ ] Check Wine installation
- [ ] Check vkd3d-proton installation

---

### 10. **Platform-Specific Optimization**
**Purpose:** Maximize FPS on CMP 170HX + 32GB DDR5 + B650E

**Already done:**
- [x] FMA disable for CMP
- [x] Belobug ports (enabled)
- [x] Texture staging auto-tuning
- [x] vkd3d-proton tuning

**Still needed:**
- [ ] GPU frequency/power state control (`nvidia-smi -pl`)
- [ ] CPU affinity (pin Wine to NUMA node 0)
- [ ] Swap prefetch (preload hot pages into RAM)
- [ ] Dynamic resolution scaling (maintain 60 FPS)
- [ ] Frame pacing (avoid stutters)

---

## Architecture

```
┌─────────────────────────────────────────┐
│   Launcher UI (Tauri + React/Leptos)   │
│   - Game library                        │
│   - Settings                            │
│   - Overlay (FPS, stats)               │
│   - Performance graphs                  │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Launcher Core (Rust)                  │
│   - Config manager                      │
│   - Game launcher (victory-linux)       │
│   - Profiler (GPU/CPU/RAM telemetry)   │
│   - Crash reporter                      │
│   - Update manager                      │
└──────────────┬──────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Platform Layer (Linux + NVIDIA)       │
│   - Wine + vkd3d-proton                │
│   - nvidia-smi queries                  │
│   - GPU profiling (nvidia-smi, perf)   │
│   - System telemetry                    │
└─────────────────────────────────────────┘
               │
┌──────────────▼──────────────────────────┐
│   Game (victory.exe via Wine)           │
│   - Windows binary                      │
│   - D3D12 → Vulkan (vkd3d-proton)      │
│   - NVIDIA GPU rendering                │
│   - Performance hooks                   │
└─────────────────────────────────────────┘
```

---

## Implementation Phases

### Phase 1: MVP (Now → 2 weeks)
- [x] Linux launcher script (victory-linux)
- [x] NVIDIA auto-tuning
- [x] Basic performance monitoring (overlay FPS)
- [x] Cross-platform code builds
- [ ] **NEW:** Web-based launcher UI (Tauri + React)
- [ ] **NEW:** Game library directory
- [ ] **NEW:** Settings persistence

### Phase 2: Core Features (2–4 weeks)
- [ ] Real-time profiler (GPU/CPU/RAM overlay)
- [ ] Package manager (wine, vkd3d-proton versions)
- [ ] Crash reporting
- [ ] Controller remapping UI
- [ ] Save backup

### Phase 3: Polish (4–8 weeks)
- [ ] Auto-update mechanism
- [ ] Leaderboards
- [ ] Screenshot/video capture
- [ ] Social sharing
- [ ] Mobile launcher (web-based, works on iPad/Android)

---

## Why Bun Here?

**For Launcher UI development:**
1. **Fast bundling** - Bun's JavaScript bundler is 10× faster than webpack
2. **Dev environment** - Hot reload, instant feedback
3. **Small runtime** - Tauri embeds Bun (or native JS engine)
4. **Package manager** - `bun install` is faster than npm/yarn

**For build tooling:**
- `bun build` for bundling UI
- `bun run` for scripts
- `bun test` for launcher tests

**For scripting:**
- Lightweight package management scripts
- Quick prototyping of launcher features

---

## What's Changed From Now

**Before:** Game binary only, manual launcher script

**After:** Complete launcher platform
- Launcher UI (web-based, Tauri)
- Game library management
- Real-time profiling overlay
- Auto-update system
- Crash reporting
- Telemetry and debugging

**Result:** Users download ONE app, click "Play GTA VI", everything else works automatically.

---

## Next Steps

1. **Choose UI framework:** Tauri + React vs. GTK vs. Qt
2. **Set up Bun** for launcher project (separate from game Rust code)
3. **Build MVP launcher UI** (game selector, launch button, settings)
4. **Integrate victory-linux** as launcher backend
5. **Add overlay** (real-time FPS/GPU/CPU monitoring)
6. **Build package manager** (wine/vkd3d-proton version control)

---

**Scope:** This turns a game into a game *platform*. Significant undertaking, but necessary for 10M users on a custom launcher.
