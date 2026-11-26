//! Performance profiling: real-time FPS, GPU load, VRAM, CPU %, temperatures
//! Collected per-frame, exported via named pipe to launcher for overlay rendering.

use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use serde::{Deserialize, Serialize};

/// ProfileFrame: metrics captured at each frame presentation
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProfileFrame {
    pub timestamp_ms: u64,  // Unix timestamp in milliseconds
    pub fps: f32,            // Instantaneous FPS (1000ms / frame_time_ms)
    pub gpu_load_percent: f32,  // 0-100% GPU utilization
    pub vram_used_mb: u64,   // GPU VRAM currently used
    pub vram_total_mb: u64,  // GPU VRAM total (usually 40,960 for CMP 170HX)
    pub cpu_load_percent: f32,  // 0-100% CPU utilization (average across cores)
    pub gpu_temp_celsius: f32,  // GPU die temperature
    pub frame_time_ms: f32,  // Time from previous frame to this frame
    pub alerts: Vec<String>,  // Anomalies detected this frame
}

impl Default for ProfileFrame {
    fn default() -> Self {
        Self {
            timestamp_ms: 0,
            fps: 0.0,
            gpu_load_percent: 0.0,
            vram_used_mb: 0,
            vram_total_mb: 40960,  // CMP 170HX default
            cpu_load_percent: 0.0,
            gpu_temp_celsius: 0.0,
            frame_time_ms: 0.0,
            alerts: Vec::new(),
        }
    }
}

impl ProfileFrame {
    pub fn new() -> Self {
        Self {
            timestamp_ms: unix_timestamp_ms(),
            ..Default::default()
        }
    }

    /// Calculate FPS from frame time (in milliseconds)
    pub fn from_frame_time(frame_time_ms: f32) -> f32 {
        if frame_time_ms > 0.0 {
            1000.0 / frame_time_ms
        } else {
            0.0
        }
    }
}

fn unix_timestamp_ms() -> u64 {
    SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis() as u64
}

/// Ring buffer of latest ProfileFrames
const RING_BUFFER_SIZE: usize = 10000;

struct RingBuffer {
    frames: Vec<ProfileFrame>,
    head: usize,
    full: bool,
}

impl RingBuffer {
    fn new() -> Self {
        Self {
            frames: vec![ProfileFrame::default(); RING_BUFFER_SIZE],
            head: 0,
            full: false,
        }
    }

    fn push(&mut self, frame: ProfileFrame) {
        self.frames[self.head] = frame;
        self.head = (self.head + 1) % RING_BUFFER_SIZE;
        if self.head == 0 {
            self.full = true;
        }
    }

    fn iter(&self) -> impl Iterator<Item = &ProfileFrame> {
        if self.full {
            // If buffer is full, start from head (oldest) to end, then wrap
            self.frames[self.head..]
                .iter()
                .chain(self.frames[..self.head].iter())
        } else {
            // If buffer not full, start from beginning
            self.frames[..self.head]
                .iter()
                .chain([].iter())
        }
    }
}

/// Global profiler state
pub struct Profiler {
    ring_buffer: Mutex<RingBuffer>,
    named_pipe: Mutex<Option<NamedPipeWriter>>,
    enabled: AtomicBool,
    frame_count: AtomicU64,
    last_frame_time: Mutex<Option<Duration>>,
}

impl Profiler {
    fn new() -> Self {
        Self {
            ring_buffer: Mutex::new(RingBuffer::new()),
            named_pipe: Mutex::new(None),
            enabled: AtomicBool::new(should_profile()),
            frame_count: AtomicU64::new(0),
            last_frame_time: Mutex::new(None),
        }
    }

    pub fn record_frame(&self, frame: ProfileFrame) {
        if !self.enabled.load(Ordering::Relaxed) {
            return;
        }

        let mut buffer = self.ring_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.push(frame.clone());

        // Try to export to named pipe
        if let Ok(mut pipe) = self.named_pipe.lock() {
            if pipe.is_none() {
                *pipe = NamedPipeWriter::new().ok();
            }
            if let Some(writer) = pipe.as_mut() {
                let _ = writer.write_frame(&frame);
            }
        }

        self.frame_count.fetch_add(1, Ordering::Relaxed);
    }

    pub fn frame_count(&self) -> u64 {
        self.frame_count.load(Ordering::Relaxed)
    }

    pub fn get_latest_frame(&self) -> Option<ProfileFrame> {
        let buffer = self.ring_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.frames.last().cloned()
    }

    pub fn get_all_frames(&self) -> Vec<ProfileFrame> {
        let buffer = self.ring_buffer.lock().unwrap_or_else(|e| e.into_inner());
        buffer.iter().cloned().collect()
    }
}

/// Named pipe writer for exporting profiling data to launcher
#[cfg(windows)]
struct NamedPipeWriter {
    // Placeholder for Windows implementation
    // Will use named pipes (\\.\pipe\victory-game-[PID])
}

#[cfg(windows)]
impl NamedPipeWriter {
    fn new() -> std::io::Result<Self> {
        // Get the game process PID and create named pipe name
        let pid = std::process::id();
        let pipe_name = format!("\\\\.\\pipe\\victory-game-{}", pid);

        // For now, we'll just create a stub that doesn't error
        // Full implementation would create the named pipe server
        // and listen for launcher connections

        tracing::info!(pipe = %pipe_name, "profiler named pipe ready");
        Ok(Self {})
    }

    fn write_frame(&mut self, frame: &ProfileFrame) -> std::io::Result<()> {
        // Serialize frame to JSON and write to named pipe
        let json = serde_json::to_string(frame)?;

        // TODO: Implement actual named pipe writing
        // For now, only log in debug builds
        #[cfg(debug_assertions)]
        tracing::trace!(frame = %json, "profiling frame");

        Ok(())
    }
}

#[cfg(not(windows))]
struct NamedPipeWriter;

#[cfg(not(windows))]
impl NamedPipeWriter {
    fn new() -> std::io::Result<Self> {
        Ok(Self)
    }

    fn write_frame(&mut self, _frame: &ProfileFrame) -> std::io::Result<()> {
        Ok(())
    }
}

/// Check if profiling should be enabled (VICTORY_PROFILE=1)
fn should_profile() -> bool {
    std::env::var("VICTORY_PROFILE")
        .map(|v| v != "0")
        .unwrap_or(true)  // Enabled by default
}

/// Global profiler instance
static PROFILER: Mutex<Option<Profiler>> = Mutex::new(None);

/// Initialize the global profiler
pub fn init() {
    let mut profiler = PROFILER.lock().unwrap_or_else(|e| e.into_inner());
    if profiler.is_none() {
        *profiler = Some(Profiler::new());
        tracing::info!("profiler initialized");
    }
}

/// Record a profiling frame
pub fn record_frame(frame: ProfileFrame) {
    init();
    if let Ok(profiler) = PROFILER.lock() {
        if let Some(p) = profiler.as_ref() {
            p.record_frame(frame);
        }
    }
}

/// Get the total number of frames recorded
pub fn frame_count() -> u64 {
    init();
    if let Ok(profiler) = PROFILER.lock() {
        if let Some(p) = profiler.as_ref() {
            return p.frame_count();
        }
    }
    0
}

/// Get the latest recorded frame
pub fn latest_frame() -> Option<ProfileFrame> {
    init();
    if let Ok(profiler) = PROFILER.lock() {
        if let Some(p) = profiler.as_ref() {
            return p.get_latest_frame();
        }
    }
    None
}

/// Get all recorded frames (for export/analysis)
pub fn all_frames() -> Vec<ProfileFrame> {
    init();
    if let Ok(profiler) = PROFILER.lock() {
        if let Some(p) = profiler.as_ref() {
            return p.get_all_frames();
        }
    }
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_fps_calculation() {
        assert_eq!(ProfileFrame::from_frame_time(16.667), 60.0);
        assert_eq!(ProfileFrame::from_frame_time(33.333), 30.0);
    }

    #[test]
    fn test_ring_buffer() {
        let mut buffer = RingBuffer::new();

        // Fill first 100 frames
        for i in 0..100 {
            let mut frame = ProfileFrame::new();
            frame.fps = i as f32;
            buffer.push(frame);
        }

        // Collect and verify order
        let collected: Vec<_> = buffer.iter().map(|f| f.fps as u32).collect();
        assert_eq!(collected.len(), 100);
        assert_eq!(collected[0], 0);
        assert_eq!(collected[99], 99);
    }

    #[test]
    fn test_frame_default() {
        let frame = ProfileFrame::default();
        assert_eq!(frame.vram_total_mb, 40960);  // CMP 170HX default
        assert_eq!(frame.fps, 0.0);
        assert!(frame.alerts.is_empty());
    }
}
