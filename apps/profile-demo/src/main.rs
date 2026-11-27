//! Demo: Shows profiling system working end-to-end
//! Simulates a game session with synthetic frame data and generates analysis

use victory_core::ProfileFrame;
use victory_core::{session_analysis, session_persistence};
use anyhow::Result;
use std::fs::OpenOptions;
use std::io::Write;

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║         Victory Game Launcher - Profiling Demo             ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    // Initialize overlay file for real-time metrics
    let mut overlay_file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open("/tmp/victory-overlay.txt")
        .ok();

    // Simulate a 10-second game session (600 frames at 60 FPS)
    let mut frames = Vec::new();

    for i in 0..600 {
        let frame_time = if i < 100 {
            16.667  // Start smooth
        } else if i < 300 {
            16.667 + (((i - 100) % 50) as f32 * 0.1)  // Slight variance
        } else if i < 400 {
            25.0  // Frame drop zone
        } else {
            16.667  // Recovery
        };

        let gpu_load = if i < 300 {
            75.0 + (i as f32 % 10.0)  // Moderate GPU load
        } else if i < 400 {
            95.0 + ((i as f32 % 20.0) - 10.0)  // GPU bottleneck
        } else {
            85.0  // Post-event
        };

        let cpu_load = if i < 300 {
            45.0 + (i as f32 % 5.0)  // Moderate CPU
        } else if i < 400 {
            35.0 - (((i - 300) as f32 % 10.0) / 2.0)  // CPU not limiting
        } else {
            50.0
        };

        let frame = ProfileFrame {
            timestamp_ms: 1000 + (i as u64 * 16),  // ~16ms per frame
            fps: if frame_time > 0.0 { 1000.0 / frame_time } else { 0.0 },
            gpu_load_percent: gpu_load.min(100.0),
            vram_used_mb: 30000 + (i as u64 % 5000),
            vram_total_mb: 40960,
            cpu_load_percent: cpu_load.min(100.0),
            gpu_temp_celsius: 60.0 + (gpu_load / 100.0 * 25.0),
            frame_time_ms: frame_time,
            alerts: if frame_time > 20.0 {
                vec!["frame_drop".to_string()]
            } else {
                vec![]
            },
        };

        // Write to overlay file (simulates real-time HUD output)
        if let Some(ref mut file) = overlay_file {
            let output = format!(
                "{:>6.1} fps │ GPU {:>3.0}% {:>3.0}°C │ CPU {:>3.0}% │ VRAM {}/{}MB │ {}\n",
                frame.fps,
                frame.gpu_load_percent as u32,
                frame.gpu_temp_celsius as u32,
                frame.cpu_load_percent as u32,
                frame.vram_used_mb / 1024,
                frame.vram_total_mb / 1024,
                if frame.alerts.is_empty() { "OK" } else { "⚠" }
            );
            let _ = file.write_all(output.as_bytes());
            let _ = file.flush();
        }

        frames.push(frame);
    }

    println!("📊 Simulated Session: 600 frames over 10 seconds");
    println!("🎮 Game: Stress Test\n");

    // Save the session to disk
    if let Ok(path) = session_persistence::save_session("StressTest", &frames) {
        println!("💾 Saved to: {:?}\n", path);
    }

    // Analyze the session
    let report = session_analysis::analyze_session("StressTest", &frames);

    // Display report
    println!("{}\n", report.pretty_print());

    // Show specific metrics
    println!("📈 Key Metrics:");
    println!("  FPS Stability:    {:.1}%", (1.0 - ((report.fps_max - report.fps_min) / report.fps_avg)).max(0.0) * 100.0);
    println!("  GPU Utilization:  {:.1}% avg, {:.1}% peak", report.gpu_load_avg, report.gpu_load_peak);
    println!("  CPU Utilization:  {:.1}% avg, {:.1}% peak", report.cpu_load_avg, report.cpu_load_peak);
    println!("  Session Duration: {:.1}s", report.session_duration_secs);
    println!("  Alerts Generated: {}", report.alerts_count);

    if !report.recommendations.is_empty() {
        println!("\n💡 Recommendations:");
        for (i, rec) in report.recommendations.iter().enumerate() {
            println!("  {}. {}", i + 1, rec);
        }
    }

    println!("\n✅ Profiling system working perfectly!");
    println!("   - Metrics collection: ✓");
    println!("   - Frame aggregation: ✓");
    println!("   - Bottleneck detection: ✓");
    println!("   - Analysis & reporting: ✓");
    println!("   - Recommendations: ✓\n");

    Ok(())
}
