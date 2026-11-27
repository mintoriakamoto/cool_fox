# D3D12 Overlay Integration Plan

## Current State
- ✅ Metrics collection working (GPU, CPU, VRAM, temps)
- ✅ Frame-by-frame profiling pipeline complete
- ✅ IPC transport (Unix sockets on Linux, named pipes on Windows)
- ✅ Debug output: metrics written to `/tmp/victory-overlay.txt` per frame
- ❌ **D3D12 rendering: metrics NOT rendered in-game**

## Architecture

### How Overlay Rendering Must Work

```
DirectX 12 Game Process
├─ Game's render loop
│  ├─ Create command list
│  ├─ Render scene
│  ├─ [INJECT VICTORY OVERLAY HERE]
│  │   └─ Read metrics from named pipe
│  │   └─ Render text overlay (FPS, GPU%, CPU%, temps)
│  │   └─ Render alerts/bottleneck indicator
│  ├─ Execute command list
│  └─ Present
├─ Victory GPU DLL hooks IDXGISwapChain::Present()
│  └─ Gains control before presentation
│  └─ Can inject overlay rendering
└─ Launches Victory Launcher (separate process)
   └─ Reads metrics from named pipe in background
```

### Implementation Approach

**Option 1: DirectX 12 Present Hook (Recommended)**
```rust
#[cfg(windows)]
unsafe extern "system" fn hooked_present(
    this: *mut IDXGISwapChain,
    sync_interval: u32,
    flags: u32,
) -> HRESULT {
    // 1. Get metrics from named pipe (non-blocking read)
    if let Some(metrics) = read_latest_metrics_from_pipe() {
        // 2. Get device from swap chain
        let device: ID3D12Device = this.GetDevice();
        
        // 3. Create overlay render pass
        render_text_overlay(&device, &metrics)?;
    }
    
    // 4. Call original Present
    original_present(this, sync_interval, flags)
}
```

**Option 2: Frame Annotation with NVIDIA FCAT**
- Less invasive: marks frame boundaries
- Doesn't require rendering overlay
- Used by NVIDIA's performance tools

## Implementation Steps

### Phase 1: Named Pipe Reader (Windows-only code path)
```rust
// crates/victory-gpu/src/d3d12_overlay.rs
#[cfg(windows)]
pub mod d3d12_overlay {
    use windows::Win32::Storage::FileSystem::*;
    use std::ffi::CString;
    
    pub unsafe fn read_metrics_from_pipe() -> Option<ProfileFrame> {
        // Open \\.\pipe\victory-game-[PID]
        // Read JSON frame (non-blocking)
        // Deserialize to ProfileFrame
        // Return or None
    }
    
    pub unsafe fn render_text_overlay(
        device: &ID3D12Device,
        metrics: &ProfileFrame,
    ) -> windows::core::Result<()> {
        // 1. Get current render target
        // 2. Create D2D text renderer
        // 3. Format text: "FPS: 120 | GPU: 85% 72°C | CPU: 45%"
        // 4. Render to overlay surface
        // 5. Composite to back buffer
        Ok(())
    }
}
```

### Phase 2: Swap Chain Present Hook
```rust
// Hook IDXGISwapChain::Present via IAT patching or detours
// When game calls Present(), our hook fires first
// Read metrics, render overlay, then call original Present
```

### Phase 3: Text Rendering (using D2D1 or Direct3D 12 directly)
- Option A: Use Direct2D (D2D1) - easier for text
- Option B: Use DirectWrite - native font rendering
- Option C: Pre-render text to texture - maximum control

## Code Location

New files needed:
```
crates/victory-gpu/
├─ src/
│  └─ d3d12_overlay.rs          (new - overlay rendering)
│  └─ mod.rs                    (add: pub mod d3d12_overlay)
│
crates/victory-core/
└─ src/
   └─ overlay_hook.rs           (new - present hook setup)
```

## Testing Requirements

### Linux (this environment)
- ✅ Compilation check only (feature-gated with `#[cfg(windows)]`)
- ✅ Unit tests of text formatting
- ⚠️ **Cannot test actual rendering** (no DirectX on Linux)

### Windows (CMP 170HX 40GB GPU)
- 🔴 **MUST TEST**: Hook properly intercepts Present()
- 🔴 **MUST TEST**: Metrics readable from named pipe in real-time
- 🔴 **MUST TEST**: Text renders at 60+ FPS (no stuttering)
- 🔴 **MUST TEST**: Works with popular engines:
  - Unreal Engine 5 (D3D12)
  - Godot 4.x (D3D12)
  - Unity 2022+ (D3D12)
- 🔴 **MUST TEST**: Overlay visible in fullscreen exclusive mode
- 🔴 **MUST TEST**: Performance overhead < 1ms per frame

## Fallback Strategy

**Current state works without D3D12 rendering:**
- ✅ Metrics collected and stored in ProfileFrame
- ✅ Metrics written to file: `/tmp/victory-overlay.txt`
- ✅ Launcher can read session data for post-game analysis
- ✅ Dashboard displays historical metrics

**D3D12 rendering is "nice to have" that:**
- Gives real-time FPS/GPU% feedback in-game
- Enables quick validation of overlay accuracy
- Demonstrates full integration

## Success Criteria

✅ = Can verify on Linux/in this environment
🔴 = Requires Windows CMP 170HX hardware

| Feature | Criteria | Environment |
|---------|----------|-------------|
| Code compiles | No errors/warnings | ✅ Linux |
| Named pipe read | Reads ProfileFrame correctly | 🔴 Windows |
| Present hook | Intercepts game's Present() | 🔴 Windows |
| Text rendering | FPS/GPU% visible in-game | 🔴 Windows |
| Performance | < 1ms overhead | 🔴 Windows |
| Works with UE5 | Overlay renders in Fortnite | 🔴 Windows CMP 170HX |

## Risk Analysis

**High Risk:**
- Some games use custom Present implementations (not standard DXGI)
- Anti-cheat engines (EAC, BattlEye) may block DLL injection
- Fullscreen exclusive mode vs. windowed mode differences

**Medium Risk:**
- Text rendering pipeline may need game-specific tuning
- Performance overhead varies by game engine

**Low Risk:**
- Named pipe communication (already tested on Linux sockets)
- Metrics collection accuracy (already validated)

## Next Steps

1. **Now (Linux):** Create stub D3D12 overlay module with `#[cfg(windows)]` gates
2. **Windows validation:** Test Present hook interception
3. **Windows validation:** Verify metrics pipe communication works under load
4. **Windows validation:** Render text overlay with format: `FPS: 120 │ GPU: 85% 72°C │ CPU: 45%`
5. **Windows validation:** Test on real CMP 170HX hardware with gaming workload
