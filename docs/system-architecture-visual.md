# Complete System Architecture & Visual Guide
## How Everything Works Together (CMP 170HX Linux + NVIDIA + Custom Launcher)

---

## 1. COMPLETE SYSTEM OVERVIEW

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                            USER'S SYSTEM (CMP 170HX)                         │
│  Motherboard: B650E | CPU: 9950X | RAM: 32GB DDR5 | GPU: 40GB VRAM         │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      LAUNCHER (Tauri + React)                          │ │
│  │  ┌──────────────┐  ┌──────────────┐  ┌──────────────┐                 │ │
│  │  │ Game Library │  │  Performance │  │   Settings   │                 │ │
│  │  │   (Browse)   │  │   Dashboard  │  │  & System    │                 │ │
│  │  │              │  │   (Real-time │  │    Check     │                 │ │
│  │  │  - GTA VI    │  │    FPS/GPU/  │  │              │                 │ │
│  │  │  - [Others]  │  │    CPU/VRAM) │  │  - Graphics  │                 │ │
│  │  │              │  │              │  │  - Input     │                 │ │
│  │  │  [Play]      │  │  - GPU Load  │  │  - Audio     │                 │ │
│  │  └──────────────┘  │  - Temps     │  │  - Controller│                 │ │
│  │                    │  - Memory    │  └──────────────┘                 │ │
│  │  ┌──────────────┐  │              │                                   │ │
│  │  │ Crash Report │  │  [Toggle F11]│                                   │ │
│  │  │   & Save     │  │              │                                   │ │
│  │  │   Manager    │  │  [Clear]     │                                   │ │
│  │  │              │  └──────────────┘                                   │ │
│  │  │ - Load Save  │                                                     │ │
│  │  │ - Backup     │                                                     │ │
│  │  │ - Restore    │                                                     │ │
│  │  │ - View Crash │                                                     │ │
│  │  │   Reports    │                                                     │ │
│  │  └──────────────┘                                                     │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                   ↕ (Tauri IPC)                             │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                      LAUNCHER CORE (Rust)                              │ │
│  │  - Config Manager         - Game Process Manager                       │ │
│  │  - Profiler Daemon        - Crash Reporter                             │ │
│  │  - Save Manager           - Update Manager                             │ │
│  │  - Package Manager        - Telemetry Client                           │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                   ↓            ↓             ↓             ↓                 │
│  ┌──────────────┐ ┌──────────┐ ┌──────────┐ ┌──────────────────┐           │
│  │ Config Files │ │ Game Exe │ │ Wine Env │ │ System Telemetry │           │
│  │              │ │ + Assets │ │ + Prefix │ │ (nvidia-smi,     │           │
│  │ ~/.config/   │ │          │ │          │ │  /proc, dmesg)   │           │
│  │  launcher/   │ │ ~/.local/ │ │ ~/.local/│ │                  │           │
│  │  games/      │ │  share/  │ │  share/  │ │ System Monitoring │          │
│  │  settings/   │ │ launcher/│ │ victory/ │ │ Thread            │          │
│  │  cache/      │ │  games/  │ │  prefix/ │ └──────────────────┘           │
│  │  crashes/    │ │          │ │          │                                 │
│  │  profiles/   │ │ Game Exe │ │ Wine 9.x │                                 │
│  └──────────────┘ │ + DLLs   │ │ Proton   │                                 │
│                   │ + Shaders│ │ GE-Proton│                                 │
│                   └──────────┘ │ 9.0+     │                                 │
│                                └──────────┘                                 │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │                     GAME PROCESS (victory.exe)                         │ │
│  │                                                                        │ │
│  │  Windows Binary (x86_64) → Wine D3D12 Wrapper                        │ │
│  │                         ↓                                             │ │
│  │                   vkd3d-proton (D3D12 → Vulkan)                      │ │
│  │                         ↓                                             │ │
│  │  ┌──────────────────────────────────────────────────────────────┐   │ │
│  │  │  Victory GPU Renderer (Vulkan)                               │   │ │
│  │  │                                                              │   │ │
│  │  │  - ProfileFrame collection (FPS, GPU load, VRAM, temps)    │   │ │
│  │  │  - Frame profiling hooks at swap submission                │   │ │
│  │  │  - Export via named pipe: /tmp/victory-game-[PID].sock     │   │ │
│  │  │  - Performance data: 10,000 frame ring buffer              │   │ │
│  │  │                                                              │   │ │
│  │  │  Belobug Ports Enabled:                                     │   │ │
│  │  │  - VICTORY_PORT_CONSTANTS=1 (32-bit heap)                 │   │ │
│  │  │  - VICTORY_PORT_DESCRIPTORS=1 (descriptor records)        │   │ │
│  │  │  - VICTORY_PORT_GUARD=1 (bounds checking)                 │   │ │
│  │  │  - VICTORY_PORT_JOURNAL=1 (operation logging)             │   │ │
│  │  │                                                              │   │ │
│  │  │  FMA Disabled (CMP 170HX):                                 │   │ │
│  │  │  - mul(a, b); add(result, c) instead of fma(a,b,c)        │   │ │
│  │  │  - Prevents GPU precision issues on CMP cards             │   │ │
│  │  └──────────────────────────────────────────────────────────────┘   │ │
│  │                         ↓ (Named Pipe)                               │ │
│  │  ┌────────────────────────────────────────────────────────────────┐  │ │
│  │  │  Named Pipe: /tmp/victory-game-[PID].sock                     │  │ │
│  │  │                                                               │  │ │
│  │  │  Message Format (per frame):                                 │  │ │
│  │  │  {                                                            │  │ │
│  │  │    "timestamp": 1700000000123,  // milliseconds             │  │ │
│  │  │    "fps": 58.3,                 // instantaneous            │  │ │
│  │  │    "gpu_load": 87.5,            // percentage               │  │ │
│  │  │    "vram_used_mb": 32456,       // current usage            │  │ │
│  │  │    "vram_total_mb": 40960,      // 40 GB                    │  │ │
│  │  │    "cpu_load": 65.2,            // percentage               │  │ │
│  │  │    "gpu_temp_c": 72,            // degrees celsius          │  │ │
│  │  │    "alerts": []                 // anomalies                │  │ │
│  │  │  }                                                            │  │ │
│  │  └────────────────────────────────────────────────────────────────┘  │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │               PROFILER DAEMON (Rust - Launcher Core)                   │ │
│  │                                                                        │ │
│  │  - Listen on /tmp/victory-game-[PID].sock                            │ │
│  │  - Parse ProfileFrame messages from game                             │ │
│  │  - Aggregate stats: rolling avg FPS, peak VRAM, etc.                │ │
│  │  - Detect anomalies: "FPS drop 20%", "VRAM at 95%", "Temp 90°C"   │ │
│  │  - Store in-memory buffer (last 300 seconds of data)                │ │
│  │  - Render overlay on game window (D3D12 hook)                       │ │
│  │  - Write session profiling data to: ~/.local/share/launcher/        │ │
│  │    profiles/gta6-[date]-[time].json                                 │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │               CRASH HANDLER (Rust - Launcher Core)                    │ │
│  │                                                                        │ │
│  │  - Monitor game process exit code                                    │ │
│  │  - If exit_code != 0:                                               │ │
│  │    1. Gather system info (GPU, driver, kernel, CPU, RAM, temps)    │ │
│  │    2. Collect logs (Wine console, dmesg last 100 lines)            │ │
│  │    3. Capture game state (last profiling frame, shader cache)      │ │
│  │    4. Write crash dump: ~/.cache/launcher/crashes/[id].json        │ │
│  │    5. Optionally send to telemetry server (opt-in)                 │ │
│  │    6. Show user: "Game crashed: [reason]. Try: [suggestions]"      │ │
│  │  - Log file: ~/.local/share/launcher/game-sessions.log             │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │               GPU / CPU / SYSTEM MONITORING                           │ │
│  │                                                                        │ │
│  │  Sources:                                                             │ │
│  │  - nvidia-smi queries: GPU load, VRAM, temperature, power           │ │
│  │  - /proc/stat: CPU load                                             │ │
│  │  - /proc/meminfo: System RAM                                        │ │
│  │  - /sys/class/thermal: CPU/GPU temps (fallback)                     │ │
│  │  - Game's ProfileFrame data (FPS, internal metrics)                 │ │
│  │                                                                        │ │
│  │  Update frequency: 60 Hz (every ~16ms, synchronized with FPS)       │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │               NVIDIA GPU CONFIGURATION (PRIME Render Offload)         │ │
│  │                                                                        │ │
│  │  Setup:                                                               │ │
│  │  - CMP 170HX renders all frames (no display output on this card)    │ │
│  │  - iGPU reads VRAM via PCIe link (narrow: Gen3 x4 ~985 MB/s)       │ │
│  │  - iGPU outputs to HDMI port on motherboard                        │ │
│  │  - Env vars: __NV_PRIME_RENDER_OFFLOAD=1                          │ │
│  │             __GLX_VENDOR_LIBRARY_NAME=nvidia                       │ │
│  │             __VK_LAYER_NV_optimus=NVIDIA_only                      │ │
│  │             VKD3D_FILTER_DEVICE_NAME=NVIDIA                        │ │
│  │             NV_GPU=0 (select GPU 0 if multi-GPU)                   │ │
│  │                                                                        │ │
│  │  Texture Staging:                                                     │ │
│  │  - Stage size: 128 MB/frame (auto-tuned from PCIe link width)      │ │
│  │  - Shader cache: 128 × 32 = 4,096 MB (from staging size)          │ │
│  │  - Upload ahead 1 frame: VICTORY_UPLOAD_AHEAD=1                    │ │
│  │  - Never wait on texture: VICTORY_TEXTURE_NO_WAIT=1                │ │
│  │                                                                        │ │
│  │  Tuning:                                                              │ │
│  │  - Descriptor validation off: VKD3D_DESCRIPTOR_QA=0 (+perf)         │ │
│  │  - Force discrete GPU: VKD3D_FORCE_DISCRETE_GPU=1 (failsafe)       │ │
│  │  - Tight GPU pipeline: VKD3D_MAX_FRAME_LATENCY=1                   │ │
│  │  - Feature level 12_2: VKD3D_FEATURE_LEVEL=12_2                    │ │
│  │  - No HVV uploads: VKD3D_CONFIG=no_upload_hvv                      │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
│                                                                              │
│  ┌────────────────────────────────────────────────────────────────────────┐ │
│  │               OPTIONAL: TELEMETRY SERVER (Phase 2)                    │ │
│  │                                                                        │ │
│  │  API Endpoints (opt-in):                                            │ │
│  │  - POST /api/crashes         → store crash dump to S3               │ │
│  │  - POST /api/profiles        → store session perf data              │ │
│  │  - GET /api/game/latest      → check for game updates              │ │
│  │                                                                        │ │
│  │  Privacy:                                                             │ │
│  │  - No PII (no IP, no user ID, no install paths)                    │ │
│  │  - Only technical data (GPU model, driver, error codes)             │ │
│  │  - Opt-in checkbox: "Help improve the launcher"                    │ │
│  │  - User can delete data anytime                                     │ │
│  │                                                                        │ │
│  │  Server: AWS Lambda + API Gateway (or self-hosted Rust)            │ │
│  │  Storage: S3 for crash dumps, DynamoDB for aggregates              │ │
│  │  CDN: CloudFront for game updates                                   │ │
│  └────────────────────────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────────────────────────┘
```

---

## 2. LAUNCHER UI MOCKUP

```
┌─────────────────────────────────────────────────────────────────┐
│  🎮 Victory Launcher                               [_] [□] [X]  │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  LIBRARY              PERFORMANCE            SETTINGS           │
│  [Active] [Browse]    [Active]  [Browse]     [Active] [Browse]  │
│                                                                  │
│  ┌──────────────────────────────────────────────────────────┐  │
│  │                 LIBRARY VIEW                             │  │
│  │                                                          │  │
│  │  ╔══════════════════════════════╗  ╔════════════════╗  │  │
│  │  ║                              ║  ║               ║  │  │
│  │  ║     GTA VI                   ║  ║   Recent      ║  │  │
│  │  ║                              ║  ║               ║  │  │
│  │  ║   [Cover Art - 16:9]         ║  ║  - Cyberpunk2  ║  │  │
│  │  ║                              ║  ║               ║  │  │
│  │  ║                              ║  ║  - Baldur's   ║  │  │
│  │  ║   Version: 1.2.3             ║  ║    Gate 3     ║  │  │
│  │  ║   Playtime: 42h 15m          ║  ║               ║  │  │
│  │  ║   Last Played: Today 2:34pm  ║  ║  - Flight    ║  │  │
│  │  ║                              ║  ║    Simulator ║  │  │
│  │  ║   [   PLAY   ]               ║  ║               ║  │  │
│  │  ║   [Configure] [Manage Saves] ║  ║               ║  │  │
│  │  ╚══════════════════════════════╝  ╚════════════════╝  │  │
│  │                                                          │  │
│  └──────────────────────────────────────────────────────────┘  │
│                                                                  │
│  Status: Ready to play. System check: ✓ PASS                   │
│          GPU: CMP 170HX (40GB) | RAM: 32GB | Driver: 535.123  │
│          PCIe: Gen3 x4 (985 MB/s) | Swap: 20 GB               │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  🎮 Victory Launcher  (PERFORMANCE TAB - DURING GAMEPLAY)       │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  Real-time Performance Overlay (also shown on game screen)      │
│                                                                  │
│  ╔════════════════════════════════════════════════════════════╗ │
│  ║ FPS: 58.3  min: 42  max: 71  avg: 56.8                     ║ │
│  ║                                                            ║ │
│  ║ GPU Load:  [████████░░░░░░] 87.5%                         ║ │
│  ║ VRAM:      [██████████░░░░] 32.4 / 40.0 GB (81%)          ║ │
│  ║ CPU Load:  [█████████░░░░░░] 65.2%  (12 threads)          ║ │
│  ║ GPU Temp:  72°C  |  CPU Temp: 68°C                         ║ │
│  ║                                                            ║ │
│  ║ PCIe Link: 985 MB/s used (96% saturated - NARROW LINK)   ║ │
│  ║ Frame Time: 17.1 ms (58 FPS)                              ║ │
│  ║                                                            ║ │
│  ║ Profiling Data: 2,345 frames | Playtime: 1:23:45         ║ │
│  ║                                                            ║ │
│  ║ [Stop Profiling] [Export CSV] [Clear History]             ║ │
│  ╚════════════════════════════════════════════════════════════╝ │
│                                                                  │
│  Graphs (hover for values):                                     │
│                                                                  │
│  FPS Trend (last 5 min):        GPU Load (last 5 min):        │
│  70│      ╱╲    ╱╲             100│    ░░░░░░░░░░░            │
│  60│  ╱╲╱  ╲╱╲╱                80│ ░░░░░░░░░░░░░░░░           │
│  50│╱                          60│░░░░░░░░░░░░░░░░░░░░         │
│  40│                           40│                 ░░░░         │
│    └─────────────────────────      └──────────────────────      │
│                                                                  │
│  Game Status: Running for 1h 23m 45s                            │
│  Window: GTA VI (victory.exe via Wine)                          │
│  Process ID: 12847 | PID 12848 (render thread)                │
│                                                                  │
│  Bottleneck Detection: PCIe-bound (narrow Gen3 x4 link)        │
│  Suggestion: Reduce texture resolution or increase staging buf │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘

┌─────────────────────────────────────────────────────────────────┐
│  🎮 Victory Launcher  (SETTINGS TAB)                            │
├─────────────────────────────────────────────────────────────────┤
│                                                                  │
│  GRAPHICS                      PERFORMANCE                      │
│  ┌─────────────────┐          ┌──────────────────┐             │
│  │ Resolution:     │          │ FPS Cap: [60] ▼  │             │
│  │ [3840 × 2160] ▼│          │ VRAM Target:     │             │
│  │                 │          │ [32] GB  ▼       │             │
│  │ Texture Detail: │          │                  │             │
│  │ [High] ▼        │          │ G-Sync:  [✓]     │             │
│  │                 │          │ Adaptive Sync:   │             │
│  │ Shader Quality: │          │ [✓]              │             │
│  │ [Ultra] ▼       │          └──────────────────┘             │
│  │                 │                                            │
│  │ Ray Tracing:    │          CONTROLLER                       │
│  │ [On] ▼          │          ┌──────────────────┐             │
│  │                 │          │ Device: Xbox One │             │
│  │ Upscaling:      │          │ [Configure]      │             │
│  │ [FSR 2 Quality]▼│          │ [Test Vibration] │             │
│  └─────────────────┘          │ Deadzone: [0.15] │             │
│                               │ [Auto-calibrate] │             │
│  AUDIO                        └──────────────────┘             │
│  ┌─────────────────┐                                            │
│  │ Volume: [████░░] 75%        SYSTEM                           │
│  │ Surround Sound: [✓]         ┌──────────────────┐             │
│  │ Dialogue:  [████████░] 100% │ Telemetry:       │             │
│  │ Effects:   [█████░░░░] 65%  │ [✓] Send crashes │             │
│  │ Music:     [████████░] 90%  │ [✓] Send perf    │             │
│  │                              │                  │             │
│  │ Voicechat: [Off] ▼           │ Launch at boot:  │             │
│  │                              │ [ ]              │             │
│  │                              │ Autoupdate:      │             │
│  │                              │ [✓]              │             │
│  │                              └──────────────────┘             │
│  └─────────────────┘                                            │
│                                                                  │
│  [Save Settings] [Reset to Defaults] [Advanced...]              │
│                                                                  │
└─────────────────────────────────────────────────────────────────┘
```

---

## 3. COMPLETE DATA FLOW DIAGRAM

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        DATA FLOW: ENTIRE LIFECYCLE                      │
└─────────────────────────────────────────────────────────────────────────┘

PHASE 1: LAUNCHER STARTUP
─────────────────────────

1. User clicks "Victory Launcher" icon
   ↓
2. Tauri window initializes → React components mount
   ↓
3. Launcher Core starts:
   - Load config from ~/.config/launcher/settings.toml
   - Load game library from ~/.config/launcher/games.json
   - Scan game directories (~/.local/share/launcher/games/)
   - Check system: GPU, driver, RAM, PCIe, Wine version
   ↓
4. UI displays:
   - Game library (GTA VI, etc.)
   - Recent games list
   - System status banner (✓ PASS or ⚠ WARNING)
   ↓
5. Launcher waits for user action


PHASE 2: USER CLICKS "PLAY" (GTA VI)
────────────────────────────────────

1. Launcher reads game metadata:
   {
     "id": "gta6",
     "path": "~/.local/share/launcher/games/gta6",
     "executable": "game_scarlett_master.exe",
     "playtime_hours": 42.5,
     "settings": {
       "resolution": "3840x2160",
       "texture_detail": "high",
       "fps_cap": 60,
       "vram_target": 32
     }
   }
   ↓
2. Launcher Core spawns Game Process Manager:
   - Create WINEPREFIX=~/.local/share/victory/prefix
   - Set NVIDIA env vars for PRIME render offload
   - Set vkd3d-proton tuning env vars
   - Set Victory GPU optimization env vars (FMA disable, etc.)
   ↓
3. Victory.exe launches:
   game_scarlett_master.exe → Wine → D3D12 → vkd3d-proton → Vulkan → NVIDIA CMP GPU
   ↓
4. Game initializes and starts rendering
   ↓
5. ProfileFrame collection begins in victory-gpu:
   - Per-frame capture: timestamp, FPS, GPU load, VRAM, CPU %, temps
   - Store in ring buffer: last 10,000 frames
   - Write to named pipe: /tmp/victory-game-[PID].sock
   ↓
6. Named pipe created with game's ProfileFrame messages


PHASE 3: REAL-TIME PROFILING (DURING GAMEPLAY)
───────────────────────────────────────────────

1. Profiler Daemon (Launcher Core) subscribes to named pipe:
   /tmp/victory-game-[PID].sock
   ↓
2. Every frame (~16ms), Profiler reads message:
   {
     "timestamp": 1700000000123,
     "fps": 58.3,
     "gpu_load": 87.5,
     "vram_used_mb": 32456,
     "vram_total_mb": 40960,
     "cpu_load": 65.2,
     "gpu_temp_c": 72,
     "alerts": []
   }
   ↓
3. Profiler aggregates:
   - Rolling average FPS (last 60 frames)
   - Peak VRAM this session
   - Anomalies (FPS drop > 20%, VRAM > 95%, temp > 85°C)
   ↓
4. System monitoring thread polls:
   - nvidia-smi: GPU load, VRAM, temperature
   - /proc/stat: CPU load per core
   - /proc/meminfo: System RAM usage
   ↓
5. Overlay renderer (D3D12 hook) renders FPS/GPU/CPU bars:
   - Text overlay on game window (bottom-left corner, F11 to toggle)
   - Updates 60 times per second
   - Shows: FPS, GPU %, VRAM, CPU %, temps, PCIe saturation
   ↓
6. Launcher UI (Performance tab) also shows real-time data:
   - Same data stream as overlay
   - User can view while game is running
   - Graph animation: FPS, GPU load over time


PHASE 4: GAME CRASH (If it happens)
────────────────────────────────────

1. Wine detects exit code != 0
   ↓
2. Game Process Manager catches exit
   ↓
3. Crash Reporter activates:
   ↓
4. Gather crash context:
   - System info:
     * GPU: nvidia-smi -q
     * Driver: nvidia-smi --query-gpu=driver_version
     * Kernel: uname -r
     * CPU: cat /proc/cpuinfo
     * RAM: cat /proc/meminfo
     * Temps: cat /sys/class/thermal/thermal_zone*/temp
   - Logs:
     * Wine console (stderr/stdout redirected to ~/.cache/victory/wine.log)
     * dmesg last 100 lines (dmesg | tail -100)
     * vkd3d-proton debug logs (if enabled)
   - Game state:
     * Last 5 profiling frames (from memory)
     * Shader cache state (listing of ~/.cache/victory/shaders/)
   - Duration: playtime (from Profiler Daemon)
   ↓
5. Write crash dump:
   ~/.cache/launcher/crashes/gta6-20261126-143000.json
   {
     "crash_id": "gta6-20261126-143000",
     "timestamp": "2026-11-26T14:30:00Z",
     "duration_seconds": 300,
     "exit_code": 134,
     "system": {
       "gpu": "NVIDIA CMP 170HX",
       "driver_version": "535.123",
       "kernel": "6.1.0",
       "cpu": "AMD Ryzen 9950X",
       "ram_gb": 32,
       "vram_gb": 40
     },
     "last_profiling_frames": [...],
     "wine_log": "...",
     "dmesg": "..."
   }
   ↓
6. Show user crash report UI:
   "Game crashed after 5 minutes
    Suggested fixes:
    - Clear shader cache and retry
    - Check NVIDIA driver updates (you have 535; latest is 540)
    - Report to developers [optional]"
   ↓
7. Optional: Send to telemetry server (if opt-in enabled):
   POST /api/crashes
   Body: crash dump JSON
   ↓
8. Store session in log:
   ~/.local/share/launcher/game-sessions.log
   "2026-11-26T14:30:00Z | GTA VI | crashed | 5m | exit_code=134"


PHASE 5: GAME CLOSES NORMALLY
─────────────────────────────

1. User exits game
   ↓
2. Wine process exits cleanly (exit code 0)
   ↓
3. Game Process Manager detects exit
   ↓
4. Profiler Daemon finalizes session:
   - Calculate final stats:
     * Total playtime: duration in seconds
     * Average FPS: mean of all frames
     * Min/max FPS: extremes
     * Peak VRAM: highest value seen
     * Peak GPU load: highest percentage
     * Performance bottleneck: CPU/GPU/PCIe/memory bound?
   ↓
5. Write session profiling data:
   ~/.local/share/launcher/profiles/gta6-[date]-[time].json
   {
     "session_id": "s_20261126_143000",
     "game": "gta6",
     "start_time": "2026-11-26T14:30:00Z",
     "duration_seconds": 5100,
     "playtime_hours_total": 43.933,
     "fps_stats": {
       "avg": 56.8,
       "min": 42,
       "max": 71,
       "p95": 68,
       "p99": 70
     },
     "gpu_stats": {
       "avg_load": 87.5,
       "peak_load": 98.2,
       "avg_temp": 71,
       "peak_temp": 76
     },
     "memory_stats": {
       "peak_vram_mb": 32456,
       "peak_cpu_percent": 78,
       "peak_system_ram_mb": 29000
     },
     "bottleneck": "PCIe-bound (narrow Gen3 x4 link at 96% saturation)",
     "frames": [...]  // array of all 10,000 frames
   }
   ↓
6. Update game library metadata:
   ~/.config/launcher/games.json
   "playtime_hours": 43.933,
   "last_played": "2026-11-26T14:30:00Z"
   ↓
7. Auto-save game progress (optional):
   Scan game save directory
   Backup to ~/.local/share/launcher/backups/gta6/
   ↓
8. Show post-game report:
   "Session complete!
    Duration: 1h 25m
    Average FPS: 56.8
    Peak VRAM: 32.4 GB / 40 GB
    GPU usage: 87.5%
    Status: Stable (no anomalies)"
   ↓
9. Launcher waits for next action


PHASE 6: AUTO-UPDATE CHECK (Optional, Phase 2)
───────────────────────────────────────────────

1. On launcher startup, check for updates:
   GET /api/game/gta6/latest-version
   ↓
2. If new version available:
   - Show notification: "GTA VI v1.2.4 available (you have v1.2.3)"
   - Download delta patches: only changed files
   - Verify SHA-256 checksums
   - Apply atomically (all or nothing)
   ↓
3. If update fails:
   - Rollback to previous version
   - Alert user


PHASE 7: TELEMETRY SERVER (Optional, Phase 2)
──────────────────────────────────────────────

1. Launcher sends anonymized data (opt-in):
   POST /api/telemetry
   {
     "type": "session_complete",
     "gpu": "NVIDIA CMP 170HX",
     "driver": "535.123",
     "kernel": "6.1.0",
     "cpu_cores": 16,
     "ram_gb": 32,
     "fps_avg": 56.8,
     "fps_p99": 70,
     "gpu_load_avg": 87.5,
     "peak_vram_mb": 32456,
     "playtime_seconds": 5100
   }
   ↓
2. Server aggregates:
   - GPU model distribution
   - Driver version distribution
   - Average FPS by GPU model
   - Performance bottleneck patterns
   ↓
3. Dashboard shows:
   - Real-time crash rate
   - FPS percentiles across all users
   - GPU model distribution
   - Hardware performance breakdown
```

---

## 4. STARTUP SEQUENCE (Step-by-Step)

```
TIME    ACTION                                      FILE/PROCESS
─────────────────────────────────────────────────────────────────────────

0s      User double-clicks "Victory Launcher" icon
        ↓
50ms    Tauri creates window                       /home/user/.local/bin/victory-launcher
        React components initialize
        ↓
100ms   Load launcher config                       ~/.config/launcher/settings.toml
        Read user settings (graphics, controller, telemetry opt-in)
        ↓
120ms   Scan game directories                      ~/.local/share/launcher/games/
        Parse game metadata files
        ↓
150ms   System detection begins:
        - Query GPU info                          nvidia-smi --query-gpu=...
        - Detect driver version                   nvidia-smi --query-gpu=driver_version
        - Detect VRAM                             nvidia-smi --query-gpu=memory.total
        - Detect PCIe link                        nvidia-smi --query-gpu=pcie.link.gen...
        ↓
200ms   Load game library                          ~/.config/launcher/games.json
        Render game library UI
        ↓
250ms   System check complete:
        - GPU: ✓ CMP 170HX 40GB
        - Driver: ✓ 535.123
        - RAM: ✓ 32GB
        - PCIe: ✓ Gen3 x4
        - Wine: ✓ Installed
        - vkd3d-proton: ✓ Installed
        ↓
300ms   Display ready                              [Launcher UI visible to user]
        "Ready to play. System: ✓ PASS"
        
USER CLICKS "PLAY" ON GTA VI
───────────────────────────────────────────────────────────────────────

0ms     Button click detected                      Tauri event loop
        ↓
10ms    Launch confirmation dialog
        "Are you sure? Game will launch now."
        [Play] [Cancel]
        ↓
30ms    User clicks [Play]
        ↓
50ms    Game Process Manager spawns child process

        Environment setup:
        - VICTORY_GAME_DIR=~/.local/share/launcher/games/gta6
        - VICTORY_TEXTURE_STAGE_MB=128
        - VICTORY_DSTORAGE_STAGING_MB=128
        - VICTORY_UPLOAD_AHEAD=1
        - VICTORY_TEXTURE_NO_WAIT=1
        - VICTORY_PORT_CONSTANTS=1
        - VICTORY_PORT_DESCRIPTORS=1
        - VICTORY_PORT_GUARD=1
        - VICTORY_PORT_JOURNAL=1
        - WINEPREFIX=~/.local/share/victory/prefix
        - WINEARCH=win64
        - __NV_PRIME_RENDER_OFFLOAD=1
        - VKD3D_FILTER_DEVICE_NAME=NVIDIA
        - VKD3D_DESCRIPTOR_QA=0
        - VKD3D_FORCE_DISCRETE_GPU=1
        - NV_GPU=0
        - VKD3D_MAX_FRAME_LATENCY=1
        ↓
100ms   Wine process spawned
        wine /opt/victory/victory.exe run ...
        ↓
200ms   Wine loads D3D12 wrapper
        ↓
400ms   vkd3d-proton initializes
        D3D12 → Vulkan translation layer
        ↓
600ms   NVIDIA GPU initialized                    /dev/nvidia0
        Named pipe created
        /tmp/victory-game-[PID].sock
        ↓
800ms   Victory GPU renderer starts                victory-gpu crate
        ProfileFrame collection begins
        First frame queued
        ↓
1000ms  Game binary loads
        DLL imports resolved (wine)
        
1200ms  Shader compilation begins
        (Cold shader cache: 2-3s typical)
        ↓
1500ms  First frame rendered to VRAM
        ↓
2000ms  Game is playable                          [Game window visible]
        Launcher shows: "Game running (20s)"
        Performance overlay toggles on (F11)
        ↓
2100ms  Profiler Daemon subscribes to named pipe
        Begins reading ProfileFrame messages
        ↓
2150ms  First profiling data arrives
        fps=0, gpu_load=45%, vram=4096MB
        (Still loading shaders)
        ↓
3000ms  Shader cache warm
        fps stabilizes at 58-60
        gpu_load=87%
        vram=32456MB
        ↓
3100ms  Game fully playable                       User can now interact with game
        Launcher UI shows real-time stats
        Overlay shows FPS counter
        ↓
~1 hour Later: User plays game
        ↓
(Time varies by user)
        User exits game
        ↓
Wine process exits (exit code 0)
        ↓
+500ms  Game Process Manager detects exit
        Profiler Daemon finalizes session
        Session data written
        ↓
+1000ms Launcher UI shows post-game report
        "Session complete!
         Duration: 1h 23m
         Average FPS: 56.8"
```

---

## 5. KEY DATA STRUCTURES

### ProfileFrame (From Game to Launcher)
```rust
{
  "timestamp_ms": 1700000000123,
  "fps": 58.3,
  "gpu_load_percent": 87.5,
  "vram_used_mb": 32456,
  "vram_total_mb": 40960,
  "cpu_load_percent": 65.2,
  "gpu_temp_celsius": 72,
  "alerts": [
    "vram_at_80_percent",
    "gpu_load_critical_20_frame_drop"
  ]
}
```

### Game Session Metadata
```json
{
  "id": "s_20261126_143000",
  "game": "gta6",
  "start_time": "2026-11-26T14:30:00Z",
  "duration_seconds": 5100,
  "fps_avg": 56.8,
  "fps_min": 42,
  "fps_max": 71,
  "gpu_load_avg": 87.5,
  "peak_vram_mb": 32456,
  "peak_cpu_percent": 78,
  "exit_code": 0,
  "status": "normal"
}
```

### Crash Dump
```json
{
  "crash_id": "gta6-20261126-143000",
  "timestamp": "2026-11-26T14:30:00Z",
  "exit_code": 134,
  "system": {
    "gpu": "NVIDIA CMP 170HX",
    "driver_version": "535.123",
    "kernel": "6.1.0",
    "cpu": "AMD Ryzen 9950X",
    "ram_gb": 32
  },
  "wine_log": "...",
  "dmesg": "..."
}
```

---

## 6. WHAT WE'RE BUILDING (Deliverable)

**A complete Linux game launcher platform that:**

1. **Launches games** - User clicks "Play", game starts with auto-tuned settings
2. **Profiles performance** - Real-time FPS/GPU/CPU/VRAM overlay + graphs
3. **Handles crashes** - Captures state, shows suggestions, optional telemetry
4. **Manages saves** - Auto-backup, restore, version history
5. **Updates automatically** - Game patches via delta downloads
6. **Tracks playtime** - Per-game statistics and trends
7. **Manages dependencies** - Wine, vkd3d-proton, drivers, Vulkan layers
8. **Runs on NVIDIA only** - Optimized for CMP compute cards (40GB VRAM, narrow PCIe)
9. **Distributes as one app** - Single AppImage download, works on Ubuntu/Fedora/Arch
10. **Is extensible** - Framework for future games, mods, leaderboards, achievements

**Result:** Users download ONE app, click "Play GTA VI", everything works. No manual configuration.

---

## 7. HOW IT LOADS (Complete Sequence)

```
User Downloads "victory-launcher-linux-x86_64.AppImage" (500 MB)
        ↓
Runs: chmod +x victory-launcher-linux-x86_64.AppImage && ./victory-launcher-linux-x86_64.AppImage
        ↓
AppImage extracts:
  - Tauri + React frontend
  - Launcher Core (Rust binary)
  - Wine 9.x (embedded)
  - vkd3d-proton (embedded)
  - victory.exe (game binary)
  - Required libraries (libvulkan1, libxcb, etc.)
        ↓
Launcher window opens
        ↓
System check runs (GPU, driver, RAM, PCIe)
        ↓
Game library loads (scans ~/.local/share/launcher/games/)
        ↓
UI displays: "Ready to play"
        ↓
User clicks "Play GTA VI"
        ↓
Everything happens as described in Startup Sequence above
```

---

## Success = This Flow Works

✅ User clicks "Play"
✅ Game launches in <3 seconds
✅ Real-time overlay shows FPS/GPU/CPU
✅ Crashes are captured and reported
✅ Saves are backed up automatically
✅ Performance is tracked and graphed
✅ Launcher feels responsive and polished
✅ No manual configuration needed

This is the complete picture. Ready to build it?
