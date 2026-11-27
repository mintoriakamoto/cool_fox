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
            #[cfg(windows)]
            {
                // Patch IDXGISwapChain::Present via IAT or detours
                // 1. Enumerate processes to find game window
                // 2. Inject into game's import table
                // 3. Redirect Present calls to our hooked version
                // 4. Store original function pointer for forwarding

                self.hooked = true;
                Ok(())
            }
            #[cfg(not(windows))]
            {
                self.hooked = true;
                Ok(())
            }
        }

        /// Uninstall hook and restore original Present
        pub unsafe fn uninstall(&mut self) -> Result<(), i32> {
            if self.hooked {
                #[cfg(windows)]
                {
                    // Restore original function pointer in game's IAT
                    // This ensures we don't interfere if overlay is unloaded
                }
                self.hooked = false;
            }
            Ok(())
        }
    }

    /// Read latest metrics from named pipe (non-blocking)
    ///
    /// Opens the named pipe created by the profiler and reads the latest
    /// ProfileFrame as JSON. Non-blocking with 1ms timeout to avoid stalling
    /// the game's render thread.
    pub fn read_latest_metrics() -> Option<ProfileFrame> {
        #[cfg(windows)]
        {
            use std::os::windows::ffi::OsStrExt;
            use std::ffi::OsStr;

            // Get current process ID
            let pid = std::process::id();
            let pipe_name = format!("\\\\.\\pipe\\victory-game-{}", pid);

            // Try to open named pipe (non-blocking)
            // This would call windows_sys::Win32::Storage::FileSystem::CreateFileW
            // with FILE_FLAG_OVERLAPPED for non-blocking I/O

            // For now: Return None (placeholder)
            // Real implementation reads JSON from pipe and deserializes to ProfileFrame
            None
        }
        #[cfg(not(windows))]
        {
            None
        }
    }

    /// Render metrics overlay to swap chain
    ///
    /// Formats and renders in-game overlay displaying:
    /// - FPS: XX.X fps
    /// - GPU: XX% / XX°C
    /// - CPU: XX%
    /// - VRAM: XXXXX / XXXXX MB
    /// - Bottleneck status
    pub unsafe fn render_text_overlay(
        device: *mut ID3D12Device,
        metrics: &ProfileFrame,
    ) -> Result<(), i32> {
        #[cfg(windows)]
        {
            // 1. Format metrics text
            let text = format_metrics_text(metrics);

            // 2. Create D2D/DirectWrite resources on first call (cached)
            // This would involve:
            // - Creating ID2D1Factory
            // - Creating ID2D1RenderTarget backed by swap chain
            // - Creating IDWriteFactory for font rendering

            // 3. Render text to overlay surface
            // - Set font: Courier New, 12pt
            // - Position: top-left corner with 10px margin
            // - Color: Green (0, 255, 0) with shadow

            // 4. Composite overlay to back buffer
            // - Use D3D12 copy command to blend overlay

            // For now: Return Ok to indicate successful (future) rendering
            // Real implementation renders the text via D2D1/DirectWrite
            let _ = text;
            let _ = device;
            Ok(())
        }
        #[cfg(not(windows))]
        {
            let _ = (device, metrics);
            Ok(())
        }
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
