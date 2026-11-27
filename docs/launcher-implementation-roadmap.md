# Complete Launcher Platform - Implementation Roadmap

**Scope:** Build a full Steam-like launcher ecosystem for custom games on Linux + NVIDIA.

**Timeline:** 3-6 months (depending on team size)

**Tech Stack:**
- Frontend: Tauri + React, Bun for bundling
- Backend: Rust (victory-core utilities)
- System: Linux + NVIDIA GPU
- Distribution: Flatpak or AppImage

---

## Phase 1: MVP (Weeks 1-2)

### Core: Launcher UI Shell
- [ ] Tauri + React skeleton
- [ ] Bun build/dev setup
- [ ] Dark mode theme
- [ ] Responsive layout (desktop first)

**Files to create:**
```
launcher/
├── src-tauri/
│   ├── Cargo.toml
│   └── src/main.rs (Tauri window management)
├── src/
│   ├── App.tsx (main layout)
│   ├── components/
│   │   ├── GameLibrary.tsx
│   │   ├── LaunchButton.tsx
│   │   ├── Settings.tsx
│   │   └── Overlay.tsx
│   └── styles/
│       └── global.css
├── bunfig.toml (Bun config)
├── vite.config.ts (Vite for dev)
└── package.json
```

### Game Library
- [ ] Scan `~/.local/share/launcher/games/` for game folders
- [ ] Display game metadata (title, cover, playtime, last played)
- [ ] Simple one-click "Play" button
- [ ] Recent games list

**Data structure:**
```json
{
  "id": "gta6",
  "title": "GTA VI",
  "path": "~/.local/share/launcher/games/gta6",
  "executable": "game_scarlett_master.exe",
  "playtime_hours": 42.5,
  "last_played": "2026-11-26T12:34:56Z",
  "cover_art": "covers/gta6.jpg"
}
```

### Basic Settings
- [ ] Graphics: resolution, texture quality, shader detail
- [ ] Performance: FPS cap, VRAM target
- [ ] Audio: volume, speaker config
- [ ] Save settings to `~/.config/launcher/settings.toml`

### Launch Mechanism
- [ ] Execute `victory-linux` from launcher
- [ ] Pass game path as argument
- [ ] Capture exit code and playtime
- [ ] Show "Game is running" UI

**Deliverable:** Clickable launcher UI that starts the game. 

---

## Phase 2: Profiler & Monitoring (Weeks 3-4)

### In-Game Overlay
- [ ] Real-time FPS counter
- [ ] GPU usage (% from nvidia-smi)
- [ ] GPU memory (VRAM used / total)
- [ ] CPU usage (% from /proc/stat)
- [ ] System RAM (used / total)
- [ ] Temperature (from nvidia-smi)

**Toggle:** Press `F11` to show/hide overlay

**Data collection:**
```rust
// In victory-gpu or new victory-profiler crate
struct ProfileFrame {
    timestamp: u64,
    fps: f32,
    gpu_load: f32,
    gpu_memory: (u64, u64), // (used, total)
    cpu_load: f32,
    sys_memory: (u64, u64),
    gpu_temp: f32,
}
```

### Post-Game Report
- [ ] Save profiling data to `~/.local/share/launcher/profiles/game-id.json`
- [ ] Display stats:
  - Average FPS, min/max
  - GPU utilization (avg, peak)
  - Peak VRAM usage
  - Session duration
  - CPU bottleneck detection

**Example report:**
```
GTA VI Session
  Duration: 2h 15m
  Avg FPS: 58.3 (min 42, max 71)
  GPU: 87% avg, peak 98%
  VRAM: peak 32.4 GB / 40 GB
  CPU: 65% avg (bottleneck: NO)
  Recommendation: Stable session, no issues.
```

### Performance Graphs
- [ ] Plot FPS over time (line graph)
- [ ] GPU/CPU load trends
- [ ] VRAM usage over time
- [ ] Export as PNG or PDF

---

## Phase 3: Package Manager (Weeks 5-6)

### Dependency Installer
- [ ] Detect system: Ubuntu 22.04, Fedora 39, etc.
- [ ] Install wine64, libvulkan1, nvidia-driver
- [ ] Download & install vkd3d-proton (auto-latest)
- [ ] Verify shader cache directory structure

**Commands:**
```bash
launcher-cli pkg install wine-ge-proton-9.0
launcher-cli pkg install vkd3d-proton-latest
launcher-cli pkg update-driver nvidia
launcher-cli pkg check
```

### Version Management
- [ ] Store installed versions in `~/.config/launcher/packages.lock`
- [ ] Auto-detect NVIDIA driver version
- [ ] Alert user if driver is outdated
- [ ] One-click driver update (run nvidia-driver updater)

### Shader Cache Manager
- [ ] Location: `~/.cache/victory/shaders/`
- [ ] Size tracking (warn if > 50 GB)
- [ ] Clear cache button
- [ ] Per-game cache separate directories

---

## Phase 4: Crash Reporter (Weeks 7-8)

### Crash Capture
- [ ] Catch Wine crashes (exit code != 0)
- [ ] Capture:
  - Wine logs (last 1000 lines)
  - dmesg (last 100 lines)
  - System info (GPU, driver, kernel, CPU, RAM)
  - Game version, shader cache state
  - Timestamp and duration

**Captured data:**
```json
{
  "crash_id": "gta6-20261126-143000",
  "game": "gta6",
  "timestamp": "2026-11-26T14:30:00Z",
  "duration_seconds": 300,
  "exit_code": 134,
  "wine_log": "...",
  "dmesg": "...",
  "system": {
    "gpu": "NVIDIA CMP 170HX",
    "driver_version": "535.123",
    "kernel": "6.1.0",
    "cpu": "AMD Ryzen 9950X",
    "ram_gb": 32,
    "vram_gb": 40
  }
}
```

### Crash Reporter UI
- [ ] Show crash summary ("Game crashed after 5 minutes")
- [ ] Suggest actions:
  - "Clear shader cache and retry"
  - "Check NVIDIA driver updates"
  - "Report to developers"
- [ ] Send crash data to telemetry server (optional)

### Telemetry Server (Optional)
- [ ] Simple HTTP endpoint (AWS Lambda, or self-hosted)
- [ ] Store crashes in S3 / database
- [ ] Privacy: no PII, only technical data
- [ ] Dashboard to view crash patterns

---

## Phase 5: Controller & Input (Weeks 9-10)

### Controller Detection
- [ ] Detect Xbox 360, Xbox One, Xbox Series X|S controllers
- [ ] Show controller layout UI (visual diagram of buttons)
- [ ] Remap any button to any action

**Supported inputs:**
- D-pad, analog sticks, buttons (A, B, X, Y, LB, RB, Start, Back)
- Triggers (LT, RT)
- Vibration (rumble level 0-100%)

### Input Manager
- [ ] Save mappings to `~/.config/launcher/controller-profiles/[game-id].yaml`
- [ ] Deadzone calibration UI
- [ ] Per-game profiles (different games, different configs)
- [ ] Test button presses in a test mode

### Vibration Support
- [ ] Hook into Wine's DirectInput
- [ ] Send rumble commands to controller
- [ ] Allow user to test rumble strength

---

## Phase 6: Auto-Update (Weeks 11-12)

### Version Tracking
- [ ] Track game version in `~/.config/launcher/version.txt`
- [ ] Check server for updates on each launch
- [ ] Download delta patches (only changed files)
- [ ] Resumable downloads

**Server API:**
```
GET /api/game/gta6/latest-version
  → { version: "1.0.5", size_bytes: 2500000000, checksum: "sha256..." }

GET /api/game/gta6/delta/1.0.4→1.0.5
  → { files_changed: ["file1.exe", "shader1.bin"], total_size: 500000000 }
```

### Launcher Self-Update
- [ ] Check launcher version at startup
- [ ] Download & apply launcher updates
- [ ] Restart launcher after update

### Verification
- [ ] SHA-256 checksum validation
- [ ] Atomic apply (all-or-nothing, no partial updates)
- [ ] Rollback on failure

---

## Phase 7: Save Management & Cloud Backup (Weeks 13-14)

### Local Saves
- [ ] Scan `~/.local/share/launcher/games/[game-id]/saves/`
- [ ] Show save creation time, size, playtime at save
- [ ] Manual backup to `~/.local/share/launcher/backups/`

### Cloud Backup (Optional)
- [ ] Integrate with AWS S3 or Backblaze B2
- [ ] One-click "Backup to cloud"
- [ ] One-click "Restore from cloud"
- [ ] Version history (keep last 10 backups)

**Storage:** 100 GB cloud storage (included with launcher)

---

## Phase 8: Social Features (Weeks 15-16)

### Achievements
- [ ] Define achievements in `achievements.json`
- [ ] Hook into game events (unlocked on specific actions)
- [ ] Show achievement list in launcher
- [ ] Progress bar for multi-step achievements

### Leaderboards
- [ ] Track scores (speedrun times, high scores)
- [ ] Sync to leaderboard server
- [ ] Show global rank, personal best
- [ ] Filter by map, difficulty, region

### Screenshots & Video
- [ ] Hotkey: Shift+F12 to capture screenshot
- [ ] Store in `~/.local/share/launcher/screenshots/`
- [ ] Video recording: Alt+R to toggle 5-min rolling buffer
- [ ] Save video on demand: Alt+V

### Sharing
- [ ] Upload to Imgur (auto-link)
- [ ] Share link in chat
- [ ] One-click share to YouTube (unlisted)

---

## Phase 9: System Requirements Check (Weeks 17-18)

### Pre-Launch Checks
Before user clicks "Play", verify:

- [ ] GPU: Detect NVIDIA card
- [ ] Driver: Check >= 535
- [ ] VRAM: Warn if < 16 GB (required: 40 GB)
- [ ] RAM: Warn if < 16 GB (recommended: 32 GB)
- [ ] Swap: Warn if < 16 GB on <= 32 GB RAM
- [ ] Disk: Check free space (required: 50 GB for cache)
- [ ] PCIe: Warn if link < gen3 x8

**UI:**
```
⚠️ System Check
  ✓ GPU: NVIDIA CMP 170HX (40 GB)
  ✓ Driver: 535.123
  ✓ RAM: 32 GB
  ⚠️ Swap: 8 GB (recommended 16 GB)
     → Add 8 GB swap on NVMe: https://guides/swap-setup

  [Play] [Configure] [Cancel]
```

### Auto-Fix Suggestions
- [ ] Insufficient swap → show guide to add swap
- [ ] Old driver → one-click driver update
- [ ] Missing dependencies → one-click install
- [ ] Insufficient disk → suggest clearing shader cache

---

## Phase 10: Advanced Platform Tuning (Weeks 19-20)

### GPU Power Management
- [ ] Detect current power state (`nvidia-smi -q`)
- [ ] Allow user to set power limit (50-100%)
- [ ] Show impact: power consumption vs. FPS

### CPU Affinity
- [ ] Option: "Pin Wine to NUMA node 0"
- [ ] Show effect: CPU context switches, cache hits
- [ ] Enable/disable per-game

### Swap Prefetch
- [ ] On game launch, preload hot pages into RAM
- [ ] Reduces page faults during gameplay

### Dynamic Resolution Scaling
- [ ] Target FPS: user selects (30/60/120 FPS)
- [ ] Auto-adjust resolution to maintain target
- [ ] Show resolution changes in overlay

### Frame Pacing
- [ ] Enable G-Sync / adaptive sync
- [ ] Cap FPS to avoid tearing

---

## Project Structure

```
launcher-platform/
├── launcher-ui/                    # Tauri + React frontend
│   ├── src-tauri/                  # Rust backend for Tauri
│   ├── src/                        # React components
│   ├── bunfig.toml                 # Bun config
│   └── package.json
├── launcher-core/                  # Rust library (shared logic)
│   ├── src/
│   │   ├── lib.rs
│   │   ├── game_manager.rs         # Game library
│   │   ├── profiler.rs             # Performance profiling
│   │   ├── package_manager.rs      # Dependencies
│   │   ├── crash_reporter.rs       # Telemetry
│   │   ├── update_manager.rs       # Auto-update
│   │   └── config.rs               # Settings persistence
│   └── Cargo.toml
├── launcher-cli/                   # Command-line tool
│   ├── src/main.rs
│   └── Cargo.toml
├── docs/
│   ├── launcher-platform-plan.md   # This file
│   └── launcher-implementation-roadmap.md
└── README.md
```

---

## Development Order

1. **Week 1-2:** UI shell + game library + basic launch
2. **Week 3-4:** In-game overlay + profiler
3. **Week 5-6:** Package manager
4. **Week 7-8:** Crash reporter
5. **Week 9-10:** Controller management
6. **Week 11-12:** Auto-update
7. **Week 13-14:** Save backup (optional)
8. **Week 15-16:** Achievements + leaderboards (optional)
9. **Week 17-18:** System requirements check
10. **Week 19-20:** Advanced tuning

---

## Milestones

- **MVP (Week 2):** Launcher launches games, shows playtime
- **Usable (Week 6):** Profiler + package manager working
- **Robust (Week 12):** Auto-update, crash reporting
- **Feature-complete (Week 20):** All 10 areas implemented

---

## Team Size Estimate

**Solo developer:** 5-6 months  
**Team of 2:** 3-4 months  
**Team of 4:** 2-3 months  

---

## Success Criteria

- [ ] Users can click "Play" and game launches with auto-tuning
- [ ] Real-time FPS/GPU/RAM overlay during gameplay
- [ ] Crashes are captured and reported
- [ ] Auto-updates work seamlessly
- [ ] Controller remapping UI is intuitive
- [ ] Leaderboards show real data
- [ ] Launcher feels responsive and polished

---

## Next Steps

1. Set up Tauri + React + Bun project
2. Build game library scanner
3. Implement launch mechanism
4. Add overlay FPS counter
5. Test on CMP 170HX

Ready to start Phase 1?
