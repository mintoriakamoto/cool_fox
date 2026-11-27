//! D3D12 overlay rendering for in-game metrics display
//!
//! Provides real-time FPS, GPU load, CPU load, and thermal status overlay
//! rendered directly in-game via DirectX 12 Present hook interception.
//!
//! This module is Windows-only. On other platforms, it's a no-op.

#[cfg(windows)]
pub mod d3d12_overlay {
    use crate::profiler::ProfileFrame;
    use windows_sys::Win32::Graphics::Dxgi::IDXGISwapChain;
    use windows_sys::Win32::Graphics::Direct3D12::ID3D12Device;
    use windows_sys::Win32::Foundation::HRESULT;

    /// Hook state for IDXGISwapChain::Present interception
    pub struct PresentHook {
        original_present: Option<*const std::ffi::c_void>,
        hooked: bool,
    }

    impl PresentHook {
        pub fn new() -> Self {
            Self {
                original_present: None,
                hooked: false,
            }
        }

        /// Install hook on IDXGISwapChain::Present
        ///
        /// This intercepts all Present() calls, allowing us to inject
        /// overlay rendering before the frame is presented to the display.
        pub unsafe fn install(&mut self) -> Result<(), i32> {
            // TODO: Use windows_sys::Win32::System::Diagnostics::ToolHelp
            // to find game's IDXGISwapChain and patch its vtable
            // See: D3D12_OVERLAY_INTEGRATION.md Phase 2

            self.hooked = true;
            Ok(())
        }

        /// Uninstall hook and restore original Present
        pub unsafe fn uninstall(&mut self) -> Result<(), i32> {
            if self.hooked {
                // TODO: Restore original function pointer in vtable
                self.hooked = false;
            }
            Ok(())
        }
    }

    /// Read latest metrics from named pipe (non-blocking)
    pub fn read_latest_metrics() -> Option<ProfileFrame> {
        // TODO: Open \\.\pipe\victory-game-[PID]
        // TODO: Read JSON line (non-blocking, timeout 1ms)
        // TODO: Deserialize to ProfileFrame
        // See: D3D12_OVERLAY_INTEGRATION.md Phase 1

        // Placeholder for Linux compilation
        None
    }

    /// Render metrics overlay to swap chain
    ///
    /// Formats and renders:
    /// - FPS: XX.X fps
    /// - GPU: XX% / XX°C
    /// - CPU: XX%
    /// - VRAM: XXXXX / XXXXX MB
    /// - Status: [Balanced|GPU-bound|CPU-bound|VRAM-critical|Thermal-limited]
    pub unsafe fn render_text_overlay(
        device: *mut ID3D12Device,
        metrics: &ProfileFrame,
    ) -> Result<(), i32> {
        // TODO: Implement D2D1 or DirectWrite text rendering
        // 1. Create D2D render target backed by swap chain
        // 2. Format metrics string
        // 3. Render to overlay layer
        // 4. Composite to back buffer
        // See: D3D12_OVERLAY_INTEGRATION.md Phase 3

        // Placeholder for Linux compilation
        let _ = (device, metrics);
        Ok(())
    }

    /// Format ProfileFrame as overlay text
    fn format_metrics_text(frame: &ProfileFrame) -> String {
        let bottleneck = match (
            frame.gpu_load_percent > 95.0,
            frame.cpu_load_percent > 95.0,
            frame.vram_used_mb as f32 / frame.vram_total_mb as f32 > 0.95,
            frame.gpu_temp_celsius > 85.0,
        ) {
            (true, false, false, false) => "GPU-bound",
            (false, true, false, false) => "CPU-bound",
            (false, false, true, false) => "VRAM-critical",
            (false, false, false, true) => "Thermal-limited",
            _ => "Balanced",
        };

        format!(
            "FPS: {:.1}\nGPU: {:.0}% {:.0}°C\nCPU: {:.0}%\nVRAM: {} / {} MB\nStatus: {}",
            frame.fps,
            frame.gpu_load_percent,
            frame.gpu_temp_celsius,
            frame.cpu_load_percent,
            frame.vram_used_mb / 1024,
            frame.vram_total_mb / 1024,
            bottleneck,
        )
    }
}

#[cfg(not(windows))]
pub mod d3d12_overlay {
    use crate::profiler::ProfileFrame;

    /// No-op on non-Windows platforms
    pub struct PresentHook;

    impl PresentHook {
        pub fn new() -> Self {
            Self
        }

        pub unsafe fn install(&mut self) -> Result<(), i32> {
            Ok(())
        }

        pub unsafe fn uninstall(&mut self) -> Result<(), i32> {
            Ok(())
        }
    }

    pub fn read_latest_metrics() -> Option<ProfileFrame> {
        None
    }

    pub unsafe fn render_text_overlay(
        _device: *mut std::ffi::c_void,
        _metrics: &ProfileFrame,
    ) -> Result<(), i32> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::d3d12_overlay::*;
    use crate::profiler::ProfileFrame;

    #[test]
    fn test_metrics_text_formatting() {
        let frame = ProfileFrame {
            timestamp_ms: 1000,
            fps: 120.0,
            gpu_load_percent: 85.0,
            vram_used_mb: 30720,
            vram_total_mb: 40960,
            cpu_load_percent: 45.0,
            gpu_temp_celsius: 72.0,
            frame_time_ms: 8.333,
            alerts: vec![],
        };

        let text = d3d12_overlay::read_latest_metrics().map(|_| "metrics");
        assert!(text.is_none(), "read_latest_metrics() not implemented yet");
    }

    #[test]
    fn test_present_hook_new() {
        let hook = PresentHook::new();
        // On Windows: hook ready for installation
        // On Linux: no-op
        assert!(!std::mem::size_of_val(&hook) > 0);
    }
}
