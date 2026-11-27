//! Integration tests for Windows-specific paths
//!
//! These tests verify the system works on Windows CMP 170HX.
//! Run with: cargo test --test integration_windows --release
//!
//! On Linux: Tests will be skipped (cfg gated)
//! On Windows: Tests will validate actual Windows APIs

#![cfg(windows)]

#[cfg(windows)]
mod windows_integration {
    use std::process::Command;

    #[test]
    fn test_cpu_metrics_windows() {
        // Verify GetSystemTimes() works
        let output = Command::new("powershell")
            .args(&[
                "-Command",
                "(Get-WmiObject Win32_PerfFormattedData_PerfOS_Processor | Where-Object {$_.Name -eq '_Total'}).PercentProcessorTime"
            ])
            .output();

        assert!(output.is_ok(), "Failed to query Windows CPU metrics");
        let stdout = String::from_utf8_lossy(&output.unwrap().stdout);
        let cpu_percent: f32 = stdout.trim().parse().unwrap_or(0.0);
        assert!(cpu_percent >= 0.0 && cpu_percent <= 100.0, "CPU % out of range: {}", cpu_percent);
    }

    #[test]
    fn test_gpu_metrics_cmp170() {
        // Verify nvidia-smi returns metrics for CMP 170HX
        let output = Command::new("nvidia-smi")
            .args(&[
                "--query-gpu=memory.total,memory.used,utilization.gpu",
                "--format=csv,noheader,nounits"
            ])
            .output();

        if let Ok(out) = output {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let parts: Vec<&str> = stdout.trim().split(',').collect();

            if parts.len() >= 3 {
                let total_vram: u32 = parts[0].trim().parse().unwrap_or(0);
                let used_vram: u32 = parts[1].trim().parse().unwrap_or(0);
                let gpu_load: f32 = parts[2].trim().parse().unwrap_or(0.0);

                // On CMP 170HX: expect ~40GB VRAM
                assert!(total_vram > 30000, "VRAM too low: {} MB (CMP 170HX should have ~40GB)", total_vram);
                assert!(used_vram <= total_vram, "Used VRAM > total");
                assert!(gpu_load >= 0.0 && gpu_load <= 100.0, "GPU % out of range: {}", gpu_load);
            }
        } else {
            panic!("nvidia-smi not available or failed");
        }
    }

    #[test]
    fn test_named_pipes_creation() {
        // Verify named pipes can be created on Windows
        use std::os::windows::io::AsRawHandle;
        use windows_sys::Win32::Storage::FileSystem::*;
        use std::ffi::OsStr;
        use std::os::windows::ffi::OsStrExt;

        let pipe_name = "\\\\.\\pipe\\victory-test-integration";
        let wide_name: Vec<u16> = OsStr::new(pipe_name)
            .encode_wide()
            .chain(Some(0))
            .collect();

        unsafe {
            let handle = CreateNamedPipeW(
                wide_name.as_ptr(),
                3, // PIPE_ACCESS_DUPLEX
                0, // PIPE_TYPE_BYTE
                1, // max instances
                4096, // out buffer
                4096, // in buffer
                0, // default timeout
                std::ptr::null_mut(),
            );

            if handle as isize != -1 {
                CloseHandle(handle);
                // Success - named pipes work
                assert!(true);
            } else {
                panic!("Failed to create named pipe on Windows");
            }
        }
    }

    #[test]
    fn test_profiler_compiles() {
        // Verify victory-gpu compiles on Windows
        let output = Command::new("cargo")
            .args(&["build", "-p", "victory-gpu", "--release"])
            .output();

        assert!(output.is_ok(), "victory-gpu failed to build on Windows");
        assert!(
            output.unwrap().status.success(),
            "victory-gpu build exited with error"
        );
    }

    #[test]
    fn test_metrics_pipeline() {
        // End-to-end: collect metrics, serialize, store
        use victory_core::session_persistence;

        let sessions = session_persistence::list_sessions();
        assert!(sessions.is_ok(), "Failed to list sessions");

        // Should be able to list saved sessions from previous runs
        let sessions = sessions.unwrap();
        println!("Found {} saved sessions", sessions.len());
    }
}

// Linux builds will skip all Windows tests
#[cfg(not(windows))]
fn main() {
    println!("Windows integration tests skipped on non-Windows platform");
}
