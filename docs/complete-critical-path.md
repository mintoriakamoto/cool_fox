# Complete Critical Path to Ship
## Everything Needed Between Current State and Shipping Product

**Current state:** Game engine optimized for Linux + NVIDIA, cross-compiles, launchers work  
**Goal state:** Complete, shippable launcher platform ecosystem  
**Gap:** Everything in between

---

## Critical Path (Must-Have)

### 1. GAME ENGINE INTEGRATION

**Performance Profiling Hooks**
- [ ] Add `ProfileFrame` struct to victory-gpu (timestamp, FPS, GPU load, VRAM, CPU %, temps)
- [ ] Hook into swap submission to capture per-frame data
- [ ] Store ring buffer (last 10,000 frames in VRAM)
- [ ] Export profiling data to named pipe or shared memory

**Save Game Management**
- [ ] Implement game save scanner (find .sav files in game directory)
- [ ] Save metadata (timestamp, playtime at save, slot number)
- [ ] Implement save restore (load specific slot on game launch)
- [ ] Atomic save writes (no corruption on crash)

**Crash Handler Integration**
- [ ] Catch Wine/vkd3d crashes (exit code != 0)
- [ ] Capture game state (last N frames of rendering, shader cache state)
- [ ] Write crash dump to `~/.cache/launcher/crashes/`
- [ ] Include system info snapshot (GPU, driver, kernel, CPU, RAM)

**Input Passthrough**
- [ ] Xbox controller input routing through Wine
- [ ] Button mapping to game actions (A→Attack, etc.)
- [ ] Analog stick deadzone handling
- [ ] Vibration feedback (rumble to controller)

**Settings Serialization**
- [ ] Graphics settings → game launch args (resolution, shader detail, etc.)
- [ ] Performance settings → environment variables (VRAM target, FPS cap)
- [ ] Save/load settings from TOML or JSON
- [ ] Per-game profile storage

**Frame Rate Control**
- [ ] Implement FPS cap (target: 30/60/120 FPS)
- [ ] G-Sync / adaptive sync support
- [ ] Frame pacing (avoid stutters)
- [ ] Dynamic resolution scaling (maintain FPS target)

---

### 2. LAUNCHER ↔ GAME COMMUNICATION (IPC)

**Protocol Design**
```
Game Process (victory.exe)
    ↓
Named Pipe / Unix Socket
    ↓
Launcher Process (Tauri app)

Messages:
  GAME_STATE: { fps, gpu_load, vram_used, cpu_load, temp }
  CRASH: { exit_code, reason, timestamp }
  SAVE_LOADED: { slot, playtime }
  INPUT: { button, action }
  PERF_ALERT: { type: "low_fps"|"gpu_full"|"vram_critical" }
```

**Implementation**
- [ ] Game process writes profiling data to named pipe
- [ ] Launcher reads and displays overlay
- [ ] Bidirectional: launcher sends settings, game acknowledges
- [ ] Protocol versioning (compatible with future game versions)

**Overlay Rendering**
- [ ] In-game overlay hook (D3D12 → Vulkan translation layer)
- [ ] Real-time FPS counter at screen corner
- [ ] GPU/CPU/RAM bars
- [ ] Temperature display
- [ ] Toggle with F11 key

---

### 3. BACKEND SERVICES (Minimum)

**Telemetry Server**
- [ ] Simple HTTP API (AWS Lambda or self-hosted)
- [ ] Endpoint: `POST /api/crashes` → store crash dump to S3
- [ ] Endpoint: `GET /api/game/version/latest` → return version info
- [ ] Database: PostgreSQL or DynamoDB
- [ ] Privacy: opt-in, no PII, only technical data

**Update Server**
- [ ] Endpoint: `GET /api/game/gta6/delta/1.0.4→1.0.5` → return changed files
- [ ] Store delta patches on S3 (compressed diffs, not full files)
- [ ] Checksum verification (SHA-256)
- [ ] Atomic apply (all-or-nothing)

**Achievement/Leaderboard Server** (Phase 2, can be stub)
- [ ] Endpoint: `POST /api/leaderboards/submit` → store score
- [ ] Endpoint: `GET /api/leaderboards/top-100` → return top scores
- [ ] Database: simple table (user_id, score, timestamp, replay_data)

---

### 4. BUILD & DISTRIBUTION

**CI/CD Pipeline**
- [ ] GitHub Actions: on every push, run tests + cross-check
- [ ] On tag: build release
- [ ] Generate victory.exe (Windows cross-compile)
- [ ] Package as AppImage / Flatpak / .deb
- [ ] Sign binaries (GPG)
- [ ] Upload to S3 / GitHub Releases

**Signing & Verification**
- [ ] Generate GPG key for releases
- [ ] Sign launcher binary
- [ ] Publish public key
- [ ] Launcher verifies game exe checksum before launch

**Package Creation**
- [ ] AppImage: single-file portable executable
- [ ] Flatpak: sandboxed, easy uninstall
- [ ] Deb: standard Ubuntu/Debian installation
- [ ] All include: launcher UI + victory.exe + wine + vkd3d-proton

---

### 5. INTEGRATION LAYER

**Settings Manager**
- [ ] Store in `~/.config/launcher/settings.toml`
- [ ] Game settings (resolution, quality, keybinds)
- [ ] Launcher settings (theme, overlay enabled, telemetry opt-in)
- [ ] Per-game overrides (different GPU settings for different games)

**Game Library Manager**
- [ ] Scan `~/.local/share/launcher/games/` for game folders
- [ ] Store metadata in `~/.config/launcher/games.json`
- [ ] Track: title, path, playtime, last_played, cover_art
- [ ] Update playtime on each game close

**Save Manager**
- [ ] Automatic backup on game close
- [ ] One-click manual backup
- [ ] Show save list in UI (time, playtime, slot)
- [ ] One-click restore

**Version Management**
- [ ] Track installed game version
- [ ] Track installed wine version
- [ ] Track installed vkd3d-proton version
- [ ] Store in `~/.config/launcher/versions.lock`

---

### 6. TESTING INFRASTRUCTURE

**Performance Benchmarking**
- [ ] Benchmark suite: standard scene, 5-minute run
- [ ] Measure: avg FPS, min/max, GPU load, VRAM peak, CPU load
- [ ] Baseline: known-good result for CMP 170HX
- [ ] Regression detection: alert if FPS drops > 5%

**Compatibility Matrix**
- [ ] Test on:
  - CMP 170HX (target system)
  - RTX 4090 (high-end)
  - RTX 4070 (mid-range)
  - RTX 4060 (low-end, if supported)
- [ ] Test drivers: 535, 540, 550 (latest 3)
- [ ] Test Ubuntu 22.04, 24.04; Fedora 39, 40

**Load Testing**
- [ ] Telemetry server: simulate 10,000 crash reports/hour
- [ ] Leaderboards: 1,000 score submissions/min
- [ ] Update server: 100 concurrent downloads

**Stress Testing**
- [ ] Run game for 24 hours without crash
- [ ] Measure memory leaks (VRAM, system RAM)
- [ ] Monitor GPU/CPU temperatures
- [ ] Check for frame pacing glitches

---

### 7. SYSTEM COMPONENTS (Detailed)

**Component: Game Process Manager**
- [ ] Launch game with correct Wine prefix
- [ ] Monitor exit code and exit reason
- [ ] Kill process if unresponsive (timeout)
- [ ] Capture output logs to file
- [ ] Track session duration

**Component: Profiler Daemon**
- [ ] Read profiling data from game (named pipe)
- [ ] Aggregate stats (rolling average FPS, peak VRAM, etc.)
- [ ] Detect anomalies (FPS drops, VRAM spikes, temp warnings)
- [ ] Store historical data for graphs
- [ ] Render overlay on game window

**Component: Crash Reporter**
- [ ] Detect crash on game exit
- [ ] Gather system info (GPU, driver, kernel, CPU, RAM, temps)
- [ ] Collect logs (Wine, dmesg, vkd3d-proton debug)
- [ ] Send to telemetry server (or offer manual send)
- [ ] Show user: "What went wrong + suggestions"

**Component: Update Manager**
- [ ] Check for new version at launcher startup
- [ ] Download delta patches (not full game)
- [ ] Verify checksums
- [ ] Apply atomically (no partial updates)
- [ ] Rollback on failure

**Component: Package Manager (Dependency Tracking)**
- [ ] Detect Wine version
- [ ] Detect vkd3d-proton version
- [ ] Check NVIDIA driver version
- [ ] Alert if versions are outdated
- [ ] One-click update for each

---

### 8. DEPLOYMENT INFRASTRUCTURE

**Docker Image for Build**
- [ ] Base: Ubuntu 22.04 + NVIDIA CUDA
- [ ] Install: Rust, Wine, vkd3d-proton, build tools
- [ ] Volumes: /home/user/cool_fox (source code)
- [ ] Command: `cargo build --release --locked`

**Artifact Storage**
- [ ] S3 bucket: `launcher-releases/`
  - `game/gta6/1.0.0/victory.exe`
  - `game/gta6/1.0.0/launcher-linux-x86_64.AppImage`
  - `game/gta6/1.0.1-delta/files-changed.json`
- [ ] CloudFront CDN for fast downloads
- [ ] Signed URLs (limited time, one-time use)

**Server Infrastructure**
- [ ] API server: AWS Lambda + API Gateway (or self-hosted)
- [ ] Database: RDS PostgreSQL or DynamoDB
- [ ] File storage: S3 for game updates and crash dumps
- [ ] Logging: CloudWatch or self-hosted ELK

---

### 9. DOCUMENTATION (Critical for Users)

**Installation Guide**
- [ ] Step-by-step install for Linux (Ubuntu, Fedora, etc.)
- [ ] System requirements check
- [ ] Screenshots of each step
- [ ] Troubleshooting section

**User Guide**
- [ ] How to launch game
- [ ] How to adjust graphics settings
- [ ] How to view performance stats
- [ ] How to manage saves
- [ ] How to report bugs

**Developer Documentation**
- [ ] Architecture overview (diagrams)
- [ ] IPC protocol specification
- [ ] API documentation (telemetry, updates, leaderboards)
- [ ] Deployment guide (how to run your own launcher)

---

### 10. MONITORING & OBSERVABILITY

**Telemetry Dashboard**
- [ ] Real-time crash count
- [ ] Average FPS across all users
- [ ] Peak VRAM usage distribution
- [ ] Driver version breakdown
- [ ] GPU model breakdown

**Alerting**
- [ ] Alert if crash rate > 5%
- [ ] Alert if avg FPS drops > 10%
- [ ] Alert if leaderboard server down
- [ ] Alert if update server latency > 1s

**Metrics to Track**
- [ ] Session duration (playtime)
- [ ] Crash rate (crashes per session)
- [ ] Performance distribution (FPS percentiles)
- [ ] Hardware distribution (GPUs, drivers, OS versions)

---

## Implementation Priority

### MVP (Must have to ship)
1. ✅ Game engine optimized
2. ✅ Linux launcher working
3. **Performance profiling hooks** ← NEXT
4. **Save game management**
5. **Crash handler**
6. **Telemetry server** (simple)
7. **Package creation** (AppImage)
8. **CI/CD pipeline**

### Phase 2 (Nice to have)
9. Update server (delta patches)
10. Cloud saves
11. Controller remapping UI
12. Achievement tracking

### Phase 3 (Polish)
13. Leaderboards
14. Performance graphs
15. Mobile launcher
16. Social sharing

---

## Realistic Timeline

**MVP (Shipping):** 8 weeks
- Weeks 1-2: Profiling hooks + save management
- Weeks 3-4: Crash handler + telemetry
- Weeks 5-6: Package + CI/CD
- Weeks 7-8: Testing + fixes

**Phase 2:** 4 weeks
- Updates, cloud saves, controller UI

**Phase 3:** 4 weeks
- Polish, social features

**Total:** 16 weeks to full platform

---

## What to Build First (This Week)

**Priority 1: Performance Profiling Hooks**
- Add `ProfileFrame` struct to victory-gpu
- Hook swap submission to capture data
- Export to named pipe
- Launcher reads and displays FPS overlay

**Why:** Everything else (overlay, monitoring, profiling) depends on this.

**Priority 2: Save Game Management**
- Scanner for .sav files
- Save metadata storage
- Restore on launch

**Why:** Users need to persist progress.

**Priority 3: Crash Handler**
- Detect crashes
- Capture state
- Store crash dump

**Why:** Essential for debugging on user systems.

---

## Success Criteria for MVP

- [ ] Users can launch game and see FPS/GPU/RAM overlay
- [ ] Game crashes are captured and reported
- [ ] Game saves are automatically backed up
- [ ] Performance data is tracked and graphed
- [ ] Launcher can be packaged as single-file AppImage
- [ ] CI/CD builds and signs releases automatically
- [ ] 24-hour stress test passes without crashes

---

## What We're Not Building (Phase 3+)

- Leaderboards (server infrastructure exists, but feature not critical)
- Cloud saves (nice to have, but local backup is enough)
- Social sharing (can be added later)
- Mobile launcher (web-based launcher works on mobile already)
- Advanced controller configuration (basic passthrough is enough for MVP)

---

## Next Action

**Start with Performance Profiling Hooks this week.**

This unblocks:
- Overlay rendering
- Performance monitoring
- Profiler graphs
- Performance-based alerts

Ready to begin?
