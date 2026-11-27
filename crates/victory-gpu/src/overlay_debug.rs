//! Debug overlay output: writes metrics to file for verification

use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use crate::profiler::ProfileFrame;

static OVERLAY_FILE: Mutex<Option<std::fs::File>> = Mutex::new(None);

/// Initialize debug overlay file
pub fn init_debug_overlay() -> std::io::Result<()> {
    let path = PathBuf::from("/tmp/victory-overlay.txt");
    let file = OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&path)?;

    if let Ok(mut overlay) = OVERLAY_FILE.lock() {
        *overlay = Some(file);
    }
    Ok(())
}

/// Write frame metrics to overlay file (simulates on-screen rendering)
pub fn render_frame_metrics(frame: &ProfileFrame) {
    if let Ok(mut overlay_opt) = OVERLAY_FILE.lock() {
        if let Some(ref mut file) = *overlay_opt {
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
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_debug_overlay_file() {
        let _ = init_debug_overlay();

        let frame = ProfileFrame {
            timestamp_ms: 1000,
            fps: 120.0,
            gpu_load_percent: 85.0,
            vram_used_mb: 30000,
            vram_total_mb: 40960,
            cpu_load_percent: 45.0,
            gpu_temp_celsius: 72.0,
            frame_time_ms: 8.333,
            alerts: vec![],
        };

        render_frame_metrics(&frame);

        // Verify file was created
        assert!(std::path::Path::new("/tmp/victory-overlay.txt").exists());
    }
}
