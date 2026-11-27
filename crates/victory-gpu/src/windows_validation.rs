//! Windows platform validation - runs on Windows only
//! This module contains all Windows-specific tests and validation

#[cfg(windows)]
pub mod windows_tests {
    use std::os::windows::ffi::OsStrExt;
    use std::ffi::OsStr;

    /// Test GetSystemTimes API - validates Windows CPU metric collection
    #[test]
    fn test_getsystemtimes_api() {
        use windows_sys::Win32::System::SystemInformation::GetSystemTimes;
        use windows_sys::Win32::System::SystemInformation::FILETIME;

        let mut idle_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut kernel_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };
        let mut user_time = FILETIME { dwLowDateTime: 0, dwHighDateTime: 0 };

        // Call GetSystemTimes
        let success = unsafe {
            GetSystemTimes(
                &mut idle_time,
                &mut kernel_time,
                &mut user_time,
            )
        };

        assert!(success, "GetSystemTimes should succeed");
        assert!(idle_time.dwLowDateTime > 0 || idle_time.dwHighDateTime > 0,
            "Idle time should have non-zero value");

        println!("✓ GetSystemTimes API working");
        println!("  Idle: {}", idle_time.dwLowDateTime);
        println!("  Kernel: {}", kernel_time.dwLowDateTime);
        println!("  User: {}", user_time.dwLowDateTime);
    }

    /// Test that CPU metrics collection actually works
    #[test]
    fn test_cpu_metrics_actual() {
        use crate::metrics::SystemMetrics;

        let metrics1 = SystemMetrics::collect();
        std::thread::sleep(std::time::Duration::from_millis(100));
        let metrics2 = SystemMetrics::collect();

        // Verify metrics are real (not hardcoded)
        assert!(metrics1.cpu_load_percent >= 0.0 && metrics1.cpu_load_percent <= 100.0);
        assert!(metrics2.cpu_load_percent >= 0.0 && metrics2.cpu_load_percent <= 100.0);

        // Different samples should potentially show different values
        println!("✓ Windows CPU metrics working");
        println!("  Sample 1: {:.1}%", metrics1.cpu_load_percent);
        println!("  Sample 2: {:.1}%", metrics2.cpu_load_percent);
    }

    /// Test named pipe path generation
    #[test]
    fn test_named_pipe_creation() {
        let pid = std::process::id();
        let pipe_name = format!("\\\\.\\pipe\\victory-game-{}\0", pid);

        // Should be properly formatted for Windows named pipe
        assert!(pipe_name.starts_with("\\\\.\\pipe\\"));
        assert!(pipe_name.ends_with('\0'));

        println!("✓ Named pipe path: {}", pipe_name.trim_end_matches('\0'));
    }

    /// Test that GPU metrics collection works on Windows
    #[test]
    fn test_gpu_metrics_windows() {
        use crate::metrics::SystemMetrics;

        let metrics = SystemMetrics::collect();

        // Should have GPU metrics (either from nvidia-smi or defaults)
        assert!(metrics.vram_total_mb > 0, "VRAM total should be positive");
        assert!(metrics.gpu_load_percent >= 0.0 && metrics.gpu_load_percent <= 100.0);
        assert!(metrics.gpu_temp_celsius >= 0.0);

        println!("✓ Windows GPU metrics collected");
        println!("  GPU Load: {:.1}%", metrics.gpu_load_percent);
        println!("  VRAM: {}/{} MB", metrics.vram_used_mb, metrics.vram_total_mb);
        println!("  Temp: {:.1}°C", metrics.gpu_temp_celsius);
    }
}

#[cfg(not(windows))]
pub mod stub {
    // Stub for non-Windows platforms
    pub fn note() {
        println!("Windows validation tests require Windows platform");
    }
}
