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
    #[cfg(target_os = "linux")]
    socket_listener: Mutex<SocketListener>,
    #[cfg(target_os = "linux")]
    connected_streams: Mutex<Vec<std::os::unix::net::UnixStream>>,
    enabled: AtomicBool,
    frame_count: AtomicU64,
    last_frame_time: Mutex<Option<Duration>>,
}

impl Profiler {
    fn new() -> Self {
        #[cfg(target_os = "linux")]
        let socket_listener = SocketListener::new().unwrap_or_else(|_| SocketListener {
            listener: None,
            socket_path: format!("/tmp/victory-game-{}.sock", std::process::id()),
        });

        Self {
            ring_buffer: Mutex::new(RingBuffer::new()),
            named_pipe: Mutex::new(None),
            #[cfg(target_os = "linux")]
            socket_listener: Mutex::new(socket_listener),
            #[cfg(target_os = "linux")]
            connected_streams: Mutex::new(Vec::new()),
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

        // Windows: try to export to named pipe
        #[cfg(windows)]
        {
            if let Ok(mut pipe) = self.named_pipe.lock() {
                if pipe.is_none() {
                    *pipe = NamedPipeWriter::new().ok();
                }
                if let Some(writer) = pipe.as_mut() {
                    let _ = writer.write_frame(&frame);
                }
            }
        }

        // Linux: accept new connections and broadcast to all
        #[cfg(target_os = "linux")]
        {
            if let Ok(listener) = self.socket_listener.lock() {
                let new_streams = listener.accept_connections();
                if !new_streams.is_empty() {
                    if let Ok(mut streams) = self.connected_streams.lock() {
                        streams.extend(new_streams);
                    }
                }
            }

            if let Ok(mut streams) = self.connected_streams.lock() {
                use std::io::Write;

                let json = match serde_json::to_string(&frame) {
                    Ok(j) => j,
                    Err(_) => return,
                };
                let mut line = json;
                line.push('\n');

                // Write to all connected streams, remove dead ones
                streams.retain_mut(|stream| {
                    stream.write_all(line.as_bytes()).is_ok()
                });
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

/// Unix socket listener for accepting launcher connections (Linux only)
#[cfg(target_os = "linux")]
struct SocketListener {
    listener: Option<std::os::unix::net::UnixListener>,
    socket_path: String,
}

#[cfg(target_os = "linux")]
impl SocketListener {
    fn new() -> std::io::Result<Self> {
        let pid = std::process::id();
        let socket_path = format!("/tmp/victory-game-{}.sock", pid);

        // Remove stale socket
        let _ = std::fs::remove_file(&socket_path);

        match std::os::unix::net::UnixListener::bind(&socket_path) {
            Ok(listener) => {
                listener.set_nonblocking(true)?;
                tracing::info!(socket = %socket_path, "profiler unix socket listener started");
                Ok(Self {
                    listener: Some(listener),
                    socket_path,
                })
            }
            Err(e) => {
                tracing::warn!(socket = %socket_path, error = %e, "failed to bind unix socket");
                Ok(Self {
                    listener: None,
                    socket_path,
                })
            }
        }
    }

    fn accept_connections(&self) -> Vec<std::os::unix::net::UnixStream> {
        let mut streams = Vec::new();
        if let Some(ref listener) = self.listener {
            while let Ok((stream, _)) = listener.accept() {
                streams.push(stream);
            }
        }
        streams
    }
}

/// Named pipe writer for exporting profiling data to launcher
#[cfg(windows)]
struct NamedPipeWriter {
    pipe: Option<std::fs::File>,
    pipe_name: String,
}

#[cfg(windows)]
impl NamedPipeWriter {
    fn new() -> std::io::Result<Self> {
        let pid = std::process::id();
        let pipe_name = format!("\\\\.\\pipe\\victory-game-{}", pid);

        tracing::info!(pipe = %pipe_name, "profiler named pipe ready");
        Ok(Self {
            pipe: None,
            pipe_name,
        })
    }

    fn write_frame(&mut self, frame: &ProfileFrame) -> std::io::Result<()> {
        use std::io::Write;

        // Serialize frame to JSON
        let json = serde_json::to_string(frame)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Try to connect if not already connected
        if self.pipe.is_none() {
            if let Ok(file) = std::fs::OpenOptions::new()
                .write(true)
                .open(&self.pipe_name)
            {
                self.pipe = Some(file);
            }
        }

        // Write frame to pipe if connected
        if let Some(ref mut pipe) = self.pipe {
            let mut line = json;
            line.push('\n');

            if pipe.write_all(line.as_bytes()).is_err() {
                // Connection lost, reconnect on next frame
                self.pipe = None;
            }
        }

        Ok(())
    }
}

#[cfg(target_os = "linux")]
struct NamedPipeWriter {
    stream: Option<std::os::unix::net::UnixStream>,
    socket_path: String,
}

#[cfg(target_os = "linux")]
impl NamedPipeWriter {
    fn new() -> std::io::Result<Self> {
        let pid = std::process::id();
        let socket_path = format!("/tmp/victory-game-{}.sock", pid);

        tracing::info!(socket = %socket_path, "profiler unix socket ready");
        Ok(Self {
            stream: None,
            socket_path,
        })
    }

    fn write_frame(&mut self, frame: &ProfileFrame) -> std::io::Result<()> {
        use std::io::Write;
        use std::os::unix::net::UnixStream;

        // Serialize frame to JSON
        let json = serde_json::to_string(frame)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

        // Try to connect if not already connected
        if self.stream.is_none() {
            if let Ok(stream) = UnixStream::connect(&self.socket_path) {
                self.stream = Some(stream);
            }
        }

        // Write frame to socket if connected
        if let Some(ref mut stream) = self.stream {
            let mut line = json;
            line.push('\n');

            if stream.write_all(line.as_bytes()).is_err() {
                // Connection lost, reconnect on next frame
                self.stream = None;
            }
        }

        Ok(())
    }
}

#[cfg(not(any(windows, target_os = "linux")))]
struct NamedPipeWriter;

#[cfg(not(any(windows, target_os = "linux")))]
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
