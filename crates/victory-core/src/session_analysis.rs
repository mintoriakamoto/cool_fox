//! Session analysis and reporting

use crate::ProfileFrame;

/// Detailed profiling report for a game session
#[derive(Debug, Clone)]
pub struct ProfileReport {
    pub game_name: String,
    pub total_frames: u64,
    pub session_duration_secs: f32,
    pub fps_min: f32,
    pub fps_max: f32,
    pub fps_avg: f32,
    pub gpu_load_avg: f32,
    pub gpu_load_peak: f32,
    pub gpu_temp_max: f32,
    pub vram_used_max_mb: u64,
    pub cpu_load_avg: f32,
    pub cpu_load_peak: f32,
    pub bottleneck_analysis: String,
    pub alerts_count: usize,
    pub recommendations: Vec<String>,
}

pub fn analyze_session(game_name: &str, frames: &[ProfileFrame]) -> ProfileReport {
    if frames.is_empty() {
        return ProfileReport {
            game_name: game_name.to_string(),
            total_frames: 0,
            session_duration_secs: 0.0,
            fps_min: 0.0,
            fps_max: 0.0,
            fps_avg: 0.0,
            gpu_load_avg: 0.0,
            gpu_load_peak: 0.0,
            gpu_temp_max: 0.0,
            vram_used_max_mb: 0,
            cpu_load_avg: 0.0,
            cpu_load_peak: 0.0,
            bottleneck_analysis: "No data".to_string(),
            alerts_count: 0,
            recommendations: vec![],
        };
    }

    let first_frame = frames[0].timestamp_ms;
    let last_frame = frames[frames.len() - 1].timestamp_ms;
    let session_duration_ms = last_frame.saturating_sub(first_frame);
    let session_duration_secs = session_duration_ms as f32 / 1000.0;

    let fps_values: Vec<f32> = frames.iter().map(|f| f.fps).collect();
    let gpu_load_values: Vec<f32> = frames.iter().map(|f| f.gpu_load_percent).collect();
    let cpu_load_values: Vec<f32> = frames.iter().map(|f| f.cpu_load_percent).collect();

    let fps_min = fps_values.iter().cloned().fold(f32::INFINITY, f32::min);
    let fps_max = fps_values.iter().cloned().fold(0.0, f32::max);
    let fps_avg = fps_values.iter().sum::<f32>() / fps_values.len() as f32;

    let gpu_load_avg = gpu_load_values.iter().sum::<f32>() / gpu_load_values.len() as f32;
    let gpu_load_peak = gpu_load_values.iter().cloned().fold(0.0, f32::max);

    let cpu_load_avg = cpu_load_values.iter().sum::<f32>() / cpu_load_values.len() as f32;
    let cpu_load_peak = cpu_load_values.iter().cloned().fold(0.0, f32::max);

    let gpu_temp_max = frames.iter().map(|f| f.gpu_temp_celsius).fold(0.0, f32::max);
    let vram_used_max_mb = frames.iter().map(|f| f.vram_used_mb).max().unwrap_or(0);

    let alerts_count: usize = frames.iter().map(|f| f.alerts.len()).sum();

    // Bottleneck analysis
    let gpu_bound_frames = gpu_load_values.iter().filter(|&&g| g > 90.0).count();
    let cpu_bound_frames = cpu_load_values.iter().filter(|&&c| c > 90.0).count();
    let vram_critical_frames = frames
        .iter()
        .filter(|f| f.vram_used_mb as f32 / f.vram_total_mb as f32 > 0.9)
        .count();

    let total = frames.len();
    let gpu_percent = (gpu_bound_frames as f32 / total as f32) * 100.0;
    let cpu_percent = (cpu_bound_frames as f32 / total as f32) * 100.0;
    let vram_percent = (vram_critical_frames as f32 / total as f32) * 100.0;

    let bottleneck_analysis = if gpu_percent > 50.0 {
        format!("GPU-bound {:.0}% of session", gpu_percent)
    } else if cpu_percent > 50.0 {
        format!("CPU-bound {:.0}% of session", cpu_percent)
    } else if vram_percent > 20.0 {
        format!("VRAM pressure {:.0}% of session", vram_percent)
    } else {
        "Balanced".to_string()
    };

    // Generate recommendations
    let mut recommendations = vec![];

    if fps_min < 30.0 {
        recommendations.push(format!("FPS dropped to {:.0} - consider lower graphics settings", fps_min));
    }

    if gpu_load_peak > 95.0 && gpu_percent > 50.0 {
        recommendations.push("Significant GPU bottleneck - reduce texture resolution or ray tracing".to_string());
    }

    if cpu_load_peak > 95.0 && cpu_percent > 50.0 {
        recommendations.push("CPU bottleneck detected - reduce physics quality or increase CPU affinity".to_string());
    }

    if gpu_temp_max > 85.0 {
        recommendations.push(format!("GPU temperature reached {:.0}°C - check cooling and ventilation", gpu_temp_max));
    }

    if vram_percent > 20.0 {
        recommendations.push(format!(
            "VRAM usage near limit {:.0}% - reduce texture cache or disable high-res packs",
            vram_percent
        ));
    }

    if alerts_count > frames.len() / 20 {
        recommendations.push(format!("High alert frequency: {} alerts - system under stress", alerts_count));
    }

    ProfileReport {
        game_name: game_name.to_string(),
        total_frames: frames.len() as u64,
        session_duration_secs,
        fps_min,
        fps_max,
        fps_avg,
        gpu_load_avg,
        gpu_load_peak,
        gpu_temp_max,
        vram_used_max_mb,
        cpu_load_avg,
        cpu_load_peak,
        bottleneck_analysis,
        alerts_count,
        recommendations,
    }
}

impl ProfileReport {
    pub fn pretty_print(&self) -> String {
        format!(
            "═════════════════════════════════════════════════════════════
 GAME: {}
 Duration: {:.1}s | Frames: {}

 FPS:
  • Average: {:.1} fps
  • Range:   {:.1} - {:.1} fps

 GPU:
  • Load:    {:.1}% average ({:.1}% peak)
  • Temp:    {:.1}°C max

 CPU:
  • Load:    {:.1}% average ({:.1}% peak)

 VRAM:
  • Peak:    {} MB

 ANALYSIS:
  • Bottleneck: {}
  • Alerts:     {} detected

 RECOMMENDATIONS:
{}
═════════════════════════════════════════════════════════════",
            self.game_name,
            self.session_duration_secs,
            self.total_frames,
            self.fps_avg,
            self.fps_min,
            self.fps_max,
            self.gpu_load_avg,
            self.gpu_load_peak,
            self.gpu_temp_max,
            self.cpu_load_avg,
            self.cpu_load_peak,
            self.vram_used_max_mb,
            self.bottleneck_analysis,
            self.alerts_count,
            self.recommendations
                .iter()
                .map(|r| format!("  → {}", r))
                .collect::<Vec<_>>()
                .join("\n"),
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_analysis() {
        let frames = vec![
            ProfileFrame {
                timestamp_ms: 1000,
                fps: 60.0,
                gpu_load_percent: 70.0,
                vram_used_mb: 30000,
                vram_total_mb: 40960,
                cpu_load_percent: 50.0,
                gpu_temp_celsius: 70.0,
                frame_time_ms: 16.667,
                alerts: vec![],
            },
            ProfileFrame {
                timestamp_ms: 2000,
                fps: 59.0,
                gpu_load_percent: 72.0,
                vram_used_mb: 31000,
                vram_total_mb: 40960,
                cpu_load_percent: 52.0,
                gpu_temp_celsius: 71.0,
                frame_time_ms: 16.949,
                alerts: vec!["frame_drop".to_string()],
            },
        ];

        let report = analyze_session("TestGame", &frames);

        assert_eq!(report.game_name, "TestGame");
        assert_eq!(report.total_frames, 2);
        assert!(report.fps_avg > 59.0 && report.fps_avg < 60.5);
        assert_eq!(report.alerts_count, 1);
        assert!(!report.recommendations.is_empty());
    }

    #[test]
    fn test_pretty_print() {
        let frames = vec![ProfileFrame {
            timestamp_ms: 1000,
            fps: 60.0,
            gpu_load_percent: 85.0,
            vram_used_mb: 35000,
            vram_total_mb: 40960,
            cpu_load_percent: 50.0,
            gpu_temp_celsius: 72.0,
            frame_time_ms: 16.667,
            alerts: vec![],
        }];

        let report = analyze_session("MyGame", &frames);
        let output = report.pretty_print();

        assert!(output.contains("MyGame"));
        assert!(output.contains("FPS"));
        assert!(output.contains("GPU"));
        assert!(output.contains("VRAM"));
    }
}
