//! Real-time overlay: renders FPS, GPU load, temps, bottleneck on-screen

use crate::profiler::ProfileFrame;

/// Simple text-based overlay metrics
pub struct Overlay {
    enabled: bool,
    frame_history: Vec<f32>,
    max_history: usize,
}

impl Overlay {
    pub fn new() -> Self {
        Self {
            enabled: should_show_overlay(),
            frame_history: Vec::new(),
            max_history: 60,  // Keep 60 frames for average calc
        }
    }

    /// Format a ProfileFrame as overlay text
    pub fn format_metrics(frame: &ProfileFrame) -> String {
        let bottleneck = detect_bottleneck(frame);

        format!(
            "FPS: {:.1}\nGPU: {:.0}% / {:.0}°C\nVRAM: {} / {} MB\nCPU: {:.0}%\nBottleneck: {}\n",
            frame.fps,
            frame.gpu_load_percent,
            frame.gpu_temp_celsius,
            frame.vram_used_mb,
            frame.vram_total_mb,
            frame.cpu_load_percent,
            bottleneck,
        )
    }

    /// Track frame time for overlay history
    pub fn record_frame(&mut self, frame_time_ms: f32) {
        if !self.enabled {
            return;
        }

        self.frame_history.push(frame_time_ms);
        if self.frame_history.len() > self.max_history {
            self.frame_history.remove(0);
        }
    }

    /// Get average frame time from last N frames
    pub fn avg_frame_time(&self) -> Option<f32> {
        if self.frame_history.is_empty() {
            return None;
        }

        let sum: f32 = self.frame_history.iter().sum();
        Some(sum / self.frame_history.len() as f32)
    }

    pub fn is_enabled(&self) -> bool {
        self.enabled
    }
}

fn should_show_overlay() -> bool {
    std::env::var("VICTORY_OVERLAY")
        .map(|v| v != "0")
        .unwrap_or(true)  // Enabled by default
}

fn detect_bottleneck(frame: &ProfileFrame) -> &'static str {
    if frame.gpu_load_percent > 95.0 && frame.cpu_load_percent < 50.0 {
        "GPU-bound"
    } else if frame.cpu_load_percent > 95.0 && frame.gpu_load_percent < 80.0 {
        "CPU-bound"
    } else if frame.vram_used_mb as f32 / frame.vram_total_mb as f32 > 0.95 {
        "VRAM-critical"
    } else if frame.gpu_temp_celsius > 85.0 {
        "Thermal-limited"
    } else {
        "Balanced"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_overlay_format() {
        let frame = ProfileFrame {
            timestamp_ms: 1000,
            fps: 120.0,
            gpu_load_percent: 85.0,
            vram_used_mb: 30000,
            vram_total_mb: 40960,
            cpu_load_percent: 45.0,
            gpu_temp_celsius: 70.0,
            frame_time_ms: 8.333,
            alerts: vec![],
        };

        let text = Overlay::format_metrics(&frame);
        assert!(text.contains("FPS: 120.0"));
        assert!(text.contains("GPU: 85%"));
        assert!(text.contains("VRAM: 30000"));
        assert!(text.contains("70°C"));
    }

    #[test]
    fn test_bottleneck_detection() {
        let gpu_bound = ProfileFrame {
            timestamp_ms: 1000,
            fps: 60.0,
            gpu_load_percent: 97.0,
            vram_used_mb: 20000,
            vram_total_mb: 40960,
            cpu_load_percent: 30.0,
            gpu_temp_celsius: 60.0,
            frame_time_ms: 16.667,
            alerts: vec![],
        };

        assert_eq!(detect_bottleneck(&gpu_bound), "GPU-bound");

        let cpu_bound = ProfileFrame {
            gpu_load_percent: 70.0,
            cpu_load_percent: 98.0,
            ..gpu_bound
        };
        assert_eq!(detect_bottleneck(&cpu_bound), "CPU-bound");

        let vram_critical = ProfileFrame {
            vram_used_mb: 39000,
            vram_total_mb: 40960,
            ..gpu_bound
        };
        assert_eq!(detect_bottleneck(&vram_critical), "VRAM-critical");
    }

    #[test]
    fn test_overlay_frame_history() {
        let mut overlay = Overlay::new();

        for i in 0..100 {
            overlay.record_frame(16.667 + (i % 5) as f32);
        }

        assert_eq!(overlay.frame_history.len(), 60);
        let avg = overlay.avg_frame_time();
        assert!(avg.is_some());
        assert!(avg.unwrap() > 15.0);
    }
}
