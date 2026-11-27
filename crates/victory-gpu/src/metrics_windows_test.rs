//! Windows metrics runtime test

#[cfg(windows)]
#[cfg(test)]
mod windows_metrics_tests {
    use crate::metrics::SystemMetrics;

    #[test]
    fn test_windows_cpu_metrics_collection() {
        // Collect metrics twice to test delta calculation
        let metrics1 = SystemMetrics::collect();

        // Brief delay to ensure time difference
        std::thread::sleep(std::time::Duration::from_millis(100));

        let metrics2 = SystemMetrics::collect();

        // Verify metrics are within valid ranges
        assert!(metrics1.cpu_load_percent >= 0.0 && metrics1.cpu_load_percent <= 100.0,
            "CPU load should be 0-100%, got {}", metrics1.cpu_load_percent);

        assert!(metrics2.cpu_load_percent >= 0.0 && metrics2.cpu_load_percent <= 100.0,
            "CPU load should be 0-100%, got {}", metrics2.cpu_load_percent);

        // GPU metrics should be valid too
        assert!(metrics1.gpu_load_percent >= 0.0 && metrics1.gpu_load_percent <= 100.0,
            "GPU load should be 0-100%, got {}", metrics1.gpu_load_percent);

        assert!(metrics1.vram_total_mb > 0,
            "VRAM total should be positive, got {}", metrics1.vram_total_mb);

        println!("✓ Windows CPU metrics verified:");
        println!("  First sample:  CPU {:.1}%, GPU {:.1}%",
            metrics1.cpu_load_percent, metrics1.gpu_load_percent);
        println!("  Second sample: CPU {:.1}%, GPU {:.1}%",
            metrics2.cpu_load_percent, metrics2.gpu_load_percent);
        println!("  VRAM: {} / {} MB",
            metrics1.vram_used_mb, metrics1.vram_total_mb);
    }

    #[test]
    fn test_nvidia_smi_fallback() {
        // Test that if nvidia-smi fails, we still get valid defaults
        let metrics = SystemMetrics::collect();

        // Even without nvidia-smi, we should have VRAM total set
        assert_eq!(metrics.vram_total_mb, 40960, "Should have CMP 170HX VRAM default");

        // Metrics should not panic or error
        println!("✓ Fallback metrics valid: {}", metrics.vram_total_mb);
    }
}

#[cfg(unix)]
#[cfg(test)]
mod linux_metrics_tests {
    use crate::metrics::SystemMetrics;

    #[test]
    fn test_linux_cpu_metrics_collection() {
        // On Linux, /proc/stat should provide real CPU metrics
        let metrics1 = SystemMetrics::collect();

        std::thread::sleep(std::time::Duration::from_millis(50));

        let metrics2 = SystemMetrics::collect();

        // Metrics should be valid ranges
        assert!(metrics1.cpu_load_percent >= 0.0 && metrics1.cpu_load_percent <= 100.0);
        assert!(metrics2.cpu_load_percent >= 0.0 && metrics2.cpu_load_percent <= 100.0);

        println!("✓ Linux CPU metrics verified:");
        println!("  Sample 1: {:.1}%", metrics1.cpu_load_percent);
        println!("  Sample 2: {:.1}%", metrics2.cpu_load_percent);
    }
}
