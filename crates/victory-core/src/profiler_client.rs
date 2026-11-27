//! Launcher-side profiler client: reads ProfileFrame data from game via named pipe
//! Renders real-time overlay (FPS, GPU, CPU, VRAM, temps) on game window

use std::io::{BufRead, BufReader};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// ProfileFrame as sent from game (matches victory-gpu)
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileFrame {
    pub timestamp_ms: u64,
    pub fps: f32,
    pub gpu_load_percent: f32,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub cpu_load_percent: f32,
    pub gpu_temp_celsius: f32,
    pub frame_time_ms: f32,
    pub alerts: Vec<String>,
}

/// Session statistics aggregated from frames
#[derive(Clone, Debug, Default)]
pub struct SessionStats {
    pub frame_count: u64,
    pub fps_avg: f32,
    pub fps_min: f32,
    pub fps_max: f32,
    pub fps_p95: f32,
    pub gpu_load_avg: f32,
    pub gpu_load_peak: f32,
    pub vram_peak_mb: u64,
    pub cpu_load_avg: f32,
    pub cpu_load_peak: f32,
    pub gpu_temp_peak: f32,
    pub alerts_total: usize,
    pub bottleneck: String,
}

/// Profiler client: connects to game, reads frames, aggregates stats
pub struct ProfilerClient {
    frames: Arc<Mutex<Vec<ProfileFrame>>>,
    latest: Arc<Mutex<Option<ProfileFrame>>>,
    stats: Arc<Mutex<SessionStats>>,
    is_running: Arc<Mutex<bool>>,
    game_pid: u32,
}

impl ProfilerClient {
    pub fn new(game_pid: u32) -> Self {
        Self {
            frames: Arc::new(Mutex::new(Vec::new())),
            latest: Arc::new(Mutex::new(None)),
            stats: Arc::new(Mutex::new(SessionStats::default())),
            is_running: Arc::new(Mutex::new(false)),
            game_pid,
        }
    }

    /// Start reading profiling data from game (non-blocking)
    pub fn start_reading(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            if !*running {
                *running = true;
                self.spawn_reader_thread();
            }
        }
    }

    /// Stop reading
    pub fn stop_reading(&self) {
        if let Ok(mut running) = self.is_running.lock() {
            *running = false;
        }
    }

    /// Get latest frame (for overlay rendering)
    pub fn latest_frame(&self) -> Option<ProfileFrame> {
        self.latest.lock().ok().and_then(|l| l.clone())
    }

    /// Get current session statistics
    pub fn session_stats(&self) -> Option<SessionStats> {
        self.stats.lock().ok().map(|s| s.clone())
    }

    /// Get all frames (for export/analysis)
    pub fn all_frames(&self) -> Vec<ProfileFrame> {
        self.frames.lock().unwrap_or_else(|e| e.into_inner()).clone()
    }

    /// Spawn background thread to read from named pipe
    fn spawn_reader_thread(&self) {
        let frames = Arc::clone(&self.frames);
        let latest = Arc::clone(&self.latest);
        let stats = Arc::clone(&self.stats);
        let running = Arc::clone(&self.is_running);
        let pid = self.game_pid;

        std::thread::spawn(move || {
            #[cfg(windows)]
            Self::read_windows_pipe(&frames, &latest, &stats, &running, pid);

            #[cfg(target_os = "linux")]
            Self::read_unix_socket(&frames, &latest, &stats, &running, pid);
        });
    }

    #[cfg(windows)]
    fn read_windows_pipe(
        frames: &Arc<Mutex<Vec<ProfileFrame>>>,
        latest: &Arc<Mutex<Option<ProfileFrame>>>,
        stats: &Arc<Mutex<SessionStats>>,
        running: &Arc<Mutex<bool>>,
        pid: u32,
    ) {
        use windows::Win32::Storage::FileSystem::{
            CreateFileA, OPEN_EXISTING, FILE_GENERIC_READ, FILE_SHARE_READ, FILE_ATTRIBUTE_NORMAL,
        };
        use windows::Win32::Foundation::INVALID_HANDLE_VALUE;
        use windows::Win32::System::Pipes::WaitNamedPipeA;
        use std::ffi::CString;
        use std::os::windows::io::FromRawHandle;

        loop {
            if !running.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                break;
            }

            let pipe_name = format!("\\\\.\\pipe\\victory-game-{}\0", pid);

            // Wait for pipe to be available (up to 1 second)
            if let Ok(pipe_name_cstr) = CString::new(pipe_name.clone()) {
                unsafe {
                    let _ = WaitNamedPipeA(pipe_name_cstr.as_ptr(), 1000);
                }
            }

            // Try to open named pipe
            if let Ok(cstr) = CString::new(pipe_name.trim_end_matches('\0')) {
                unsafe {
                    let handle = CreateFileA(
                        cstr.as_ptr(),
                        FILE_GENERIC_READ,
                        FILE_SHARE_READ,
                        None,
                        OPEN_EXISTING,
                        FILE_ATTRIBUTE_NORMAL,
                        None,
                    );

                    if handle != INVALID_HANDLE_VALUE {
                        let pipe = std::fs::File::from_raw_handle(handle.0 as *mut _);
                        let reader = BufReader::new(pipe);

                        for line in reader.lines() {
                            if !running.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                                break;
                            }

                            if let Ok(line_str) = line {
                                if let Ok(frame) = serde_json::from_str::<ProfileFrame>(&line_str) {
                                    if let Ok(mut f) = frames.lock() {
                                        f.push(frame.clone());
                                    }
                                    if let Ok(mut l) = latest.lock() {
                                        *l = Some(frame.clone());
                                    }
                                    if let Ok(mut s) = stats.lock() {
                                        Self::update_stats(&mut s, &frame);
                                    }
                                }
                            }
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    #[cfg(target_os = "linux")]
    fn read_unix_socket(
        frames: &Arc<Mutex<Vec<ProfileFrame>>>,
        latest: &Arc<Mutex<Option<ProfileFrame>>>,
        stats: &Arc<Mutex<SessionStats>>,
        running: &Arc<Mutex<bool>>,
        pid: u32,
    ) {
        use std::os::unix::net::UnixStream;

        loop {
            if !running.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                break;
            }

            let socket_path = format!("/tmp/victory-game-{}.sock", pid);

            if let Ok(stream) = UnixStream::connect(&socket_path) {
                let reader = BufReader::new(stream);

                for line in reader.lines() {
                    if !running.lock().unwrap_or_else(|e| e.into_inner()).clone() {
                        break;
                    }

                    if let Ok(line_str) = line {
                        if let Ok(frame) = serde_json::from_str::<ProfileFrame>(&line_str) {
                            if let Ok(mut f) = frames.lock() {
                                f.push(frame.clone());
                            }
                            if let Ok(mut l) = latest.lock() {
                                *l = Some(frame.clone());
                            }
                            if let Ok(mut s) = stats.lock() {
                                Self::update_stats(&mut s, &frame);
                            }
                        }
                    }
                }
            }

            std::thread::sleep(Duration::from_millis(100));
        }
    }

    fn update_stats(stats: &mut SessionStats, frame: &ProfileFrame) {
        stats.frame_count += 1;

        // FPS stats
        if stats.fps_avg == 0.0 {
            stats.fps_avg = frame.fps;
            stats.fps_min = frame.fps;
            stats.fps_max = frame.fps;
        } else {
            stats.fps_avg = (stats.fps_avg * (stats.frame_count - 1) as f32 + frame.fps) / stats.frame_count as f32;
            stats.fps_min = stats.fps_min.min(frame.fps);
            stats.fps_max = stats.fps_max.max(frame.fps);
        }

        // GPU stats
        stats.gpu_load_avg = (stats.gpu_load_avg * (stats.frame_count - 1) as f32 + frame.gpu_load_percent) / stats.frame_count as f32;
        stats.gpu_load_peak = stats.gpu_load_peak.max(frame.gpu_load_percent);
        stats.vram_peak_mb = stats.vram_peak_mb.max(frame.vram_used_mb);
        stats.gpu_temp_peak = stats.gpu_temp_peak.max(frame.gpu_temp_celsius);

        // CPU stats
        stats.cpu_load_avg = (stats.cpu_load_avg * (stats.frame_count - 1) as f32 + frame.cpu_load_percent) / stats.frame_count as f32;
        stats.cpu_load_peak = stats.cpu_load_peak.max(frame.cpu_load_percent);

        // Alerts
        stats.alerts_total += frame.alerts.len();

        // Detect bottleneck
        if frame.gpu_load_percent > 95.0 && frame.cpu_load_percent < 50.0 {
            stats.bottleneck = "GPU-bound".to_string();
        } else if frame.cpu_load_percent > 95.0 && frame.gpu_load_percent < 50.0 {
            stats.bottleneck = "CPU-bound".to_string();
        } else if frame.vram_used_mb as f32 > frame.vram_total_mb as f32 * 0.95 {
            stats.bottleneck = "VRAM-bound (overflow to swap)".to_string();
        } else if frame.vram_used_mb as f32 > frame.vram_total_mb as f32 * 0.80 {
            stats.bottleneck = "VRAM-bound (high pressure)".to_string();
        } else {
            stats.bottleneck = "Balanced".to_string();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_profile_frame_serde() {
        let frame = ProfileFrame {
            timestamp_ms: 1700000000123,
            fps: 58.3,
            gpu_load_percent: 87.5,
            vram_used_mb: 32456,
            vram_total_mb: 40960,
            cpu_load_percent: 65.2,
            gpu_temp_celsius: 72.0,
            frame_time_ms: 17.1,
            alerts: vec!["vram_at_80_percent".to_string()],
        };

        let json = serde_json::to_string(&frame).unwrap();
        let parsed = serde_json::from_str::<ProfileFrame>(&json).unwrap();

        assert_eq!(parsed.fps, 58.3);
        assert_eq!(parsed.vram_used_mb, 32456);
    }

    #[test]
    fn test_session_stats_aggregation() {
        let mut stats = SessionStats::default();

        let frame1 = ProfileFrame {
            timestamp_ms: 0,
            fps: 60.0,
            gpu_load_percent: 80.0,
            vram_used_mb: 30000,
            vram_total_mb: 40960,
            cpu_load_percent: 50.0,
            gpu_temp_celsius: 70.0,
            frame_time_ms: 16.667,
            alerts: vec![],
        };

        ProfilerClient::update_stats(&mut stats, &frame1);
        assert_eq!(stats.frame_count, 1);
        assert_eq!(stats.fps_avg, 60.0);
        assert_eq!(stats.gpu_load_peak, 80.0);

        let frame2 = ProfileFrame {
            timestamp_ms: 16,
            fps: 55.0,
            gpu_load_percent: 90.0,
            vram_used_mb: 35000,
            vram_total_mb: 40960,
            cpu_load_percent: 40.0,
            gpu_temp_celsius: 75.0,
            frame_time_ms: 18.0,
            alerts: vec!["fps_drop".to_string()],
        };

        ProfilerClient::update_stats(&mut stats, &frame2);
        assert_eq!(stats.frame_count, 2);
        assert!(stats.fps_avg > 55.0 && stats.fps_avg < 60.0);
        assert_eq!(stats.gpu_load_peak, 90.0);
        assert_eq!(stats.alerts_total, 1);
    }

    #[test]
    fn test_bottleneck_detection_gpu_bound() {
        let mut stats = SessionStats::default();
        let frame = ProfileFrame {
            timestamp_ms: 0,
            fps: 30.0,
            gpu_load_percent: 99.0,  // GPU maxed
            vram_used_mb: 20000,
            vram_total_mb: 40960,
            cpu_load_percent: 20.0,  // CPU idle
            gpu_temp_celsius: 85.0,
            frame_time_ms: 33.333,
            alerts: vec!["fps_drop".to_string()],
        };

        ProfilerClient::update_stats(&mut stats, &frame);
        assert_eq!(stats.bottleneck, "GPU-bound");
    }

    #[test]
    fn test_bottleneck_detection_cpu_bound() {
        let mut stats = SessionStats::default();
        let frame = ProfileFrame {
            timestamp_ms: 0,
            fps: 40.0,
            gpu_load_percent: 30.0,  // GPU idle
            vram_used_mb: 15000,
            vram_total_mb: 40960,
            cpu_load_percent: 98.0,  // CPU maxed
            gpu_temp_celsius: 60.0,
            frame_time_ms: 25.0,
            alerts: vec![],
        };

        ProfilerClient::update_stats(&mut stats, &frame);
        assert_eq!(stats.bottleneck, "CPU-bound");
    }

    #[test]
    fn test_bottleneck_detection_vram_critical() {
        let mut stats = SessionStats::default();
        let frame = ProfileFrame {
            timestamp_ms: 0,
            fps: 50.0,
            gpu_load_percent: 70.0,
            vram_used_mb: 39000,  // 95% of 40960
            vram_total_mb: 40960,
            cpu_load_percent: 50.0,
            gpu_temp_celsius: 75.0,
            frame_time_ms: 20.0,
            alerts: vec!["vram_critical".to_string()],
        };

        ProfilerClient::update_stats(&mut stats, &frame);
        assert_eq!(stats.bottleneck, "VRAM-bound (overflow to swap)");
    }
}
