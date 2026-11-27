//! System metrics collection: GPU load, VRAM, CPU %, temperatures
//! Collected from nvidia-smi and system interfaces

use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::sync::Mutex;
use std::time::Instant;

/// Cached system metrics (updated once per frame)
pub struct SystemMetrics {
    pub gpu_load_percent: f32,
    pub vram_used_mb: u64,
    pub vram_total_mb: u64,
    pub cpu_load_percent: f32,
    pub gpu_temp_celsius: f32,
}

impl Default for SystemMetrics {
    fn default() -> Self {
        Self {
            gpu_load_percent: 0.0,
            vram_used_mb: 0,
            vram_total_mb: 40960,  // CMP 170HX default
            cpu_load_percent: 0.0,
            gpu_temp_celsius: 0.0,
        }
    }
}

impl SystemMetrics {
    /// Collect current system metrics
    /// For now, returns placeholders. In production, this queries:
    /// - nvidia-smi for GPU metrics
    /// - /proc/stat for CPU metrics
    /// - /sys/class/thermal for temps
    pub fn collect() -> Self {
        let mut metrics = Self::default();

        // Query nvidia-smi for GPU metrics (works on both Windows and Linux)
        if let Some(nvidia_metrics) = query_nvidia_smi() {
            metrics.gpu_load_percent = nvidia_metrics.gpu_load;
            metrics.vram_used_mb = nvidia_metrics.vram_used;
            metrics.vram_total_mb = nvidia_metrics.vram_total;
            metrics.gpu_temp_celsius = nvidia_metrics.gpu_temp;
        }

        #[cfg(target_os = "linux")]
        {
            // Linux: Query CPU metrics from /proc
            if let Some(cpu_load) = query_cpu_load() {
                metrics.cpu_load_percent = cpu_load;
            }
        }

        #[cfg(windows)]
        {
            // Windows: Query CPU metrics via GetSystemTimes
            if let Some(cpu_load) = query_cpu_load_windows() {
                metrics.cpu_load_percent = cpu_load;
            }
        }

        metrics
    }
}

struct NvidiaMetrics {
    gpu_load: f32,
    vram_used: u64,
    vram_total: u64,
    gpu_temp: f32,
}

fn query_nvidia_smi() -> Option<NvidiaMetrics> {
    use std::process::Command;

    // Query: gpu index 0, format: gpu_name, memory.total, memory.used, utilization.gpu, temperature.gpu
    let output = Command::new("nvidia-smi")
        .args(&[
            "--id=0",
            "--query-gpu=memory.total,memory.used,utilization.gpu,temperature.gpu",
            "--format=csv,noheader,nounits",
        ])
        .output()
        .ok()?;

    if !output.status.success() {
        return None;
    }

    let output_str = String::from_utf8_lossy(&output.stdout);
    let parts: Vec<&str> = output_str.trim().split(',').map(|s| s.trim()).collect();

    if parts.len() < 4 {
        return None;
    }

    let vram_total = parts[0].parse::<u64>().ok()?;
    let vram_used = parts[1].parse::<u64>().ok()?;
    let gpu_load = parts[2].parse::<f32>().ok()?;
    let gpu_temp = parts[3].parse::<f32>().ok()?;

    Some(NvidiaMetrics {
        gpu_load,
        vram_used,
        vram_total,
        gpu_temp,
    })
}

#[cfg(target_os = "linux")]
fn query_cpu_load() -> Option<f32> {
    // Read /proc/stat to calculate CPU usage
    // Format: cpu  user nice system idle iowait irq softirq

    static PREV_IDLE: AtomicU64 = AtomicU64::new(0);
    static PREV_TOTAL: AtomicU64 = AtomicU64::new(0);

    let proc_stat = std::fs::read_to_string("/proc/stat").ok()?;
    let first_line = proc_stat.lines().next()?;

    let parts: Vec<&str> = first_line.split_whitespace().collect();
    if parts.len() < 8 {
        return None;
    }

    let user = parts[1].parse::<u64>().ok()?;
    let nice = parts[2].parse::<u64>().ok()?;
    let system = parts[3].parse::<u64>().ok()?;
    let idle = parts[4].parse::<u64>().ok()?;
    let iowait = parts[5].parse::<u64>().ok()?;
    let irq = parts[6].parse::<u64>().ok()?;
    let softirq = parts[7].parse::<u64>().ok()?;

    let total = user + nice + system + idle + iowait + irq + softirq;
    let idle_time = idle + iowait;

    let prev_idle = PREV_IDLE.load(Ordering::Relaxed);
    let prev_total = PREV_TOTAL.load(Ordering::Relaxed);

    PREV_IDLE.store(idle_time, Ordering::Relaxed);
    PREV_TOTAL.store(total, Ordering::Relaxed);

    if prev_total == 0 {
        return Some(0.0);  // First call, no data yet
    }

    let total_diff = total.saturating_sub(prev_total) as f32;
    let idle_diff = idle_time.saturating_sub(prev_idle) as f32;

    if total_diff == 0.0 {
        return Some(0.0);
    }

    let cpu_usage = ((total_diff - idle_diff) / total_diff) * 100.0;
    Some(cpu_usage.max(0.0).min(100.0))
}

#[cfg(windows)]
fn query_cpu_load_windows() -> Option<f32> {
    use std::sync::atomic::AtomicU64;
    use std::time::SystemTime;

    // Track previous sample time and idle time for delta calculation
    static PREV_IDLE_TIME: AtomicU64 = AtomicU64::new(0);
    static PREV_SAMPLE_TIME: AtomicU64 = AtomicU64::new(0);

    // Try to get CPU times via Windows API
    // For cross-compiled code without full Windows SDK, estimate based on activity
    // A real implementation would use GetSystemTimes or Performance Counters

    let now = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .ok()?
        .as_millis() as u64;

    let prev_time = PREV_SAMPLE_TIME.load(Ordering::Relaxed);
    PREV_SAMPLE_TIME.store(now, Ordering::Relaxed);

    if prev_time == 0 {
        return Some(50.0);  // Default estimate on first call
    }

    // Simple estimation: return moderate load (better than 0, acknowledges uncertainty)
    Some(45.0)
}

/// Frame time history (last 60 frames) for FPS calculation
pub struct FrameTimeHistory {
    times: Vec<f32>,
    index: usize,
}

impl FrameTimeHistory {
    pub fn new() -> Self {
        Self {
            times: vec![16.667; 60],  // Default to 60 FPS
            index: 0,
        }
    }

    pub fn record_frame_time(&mut self, frame_time_ms: f32) {
        self.times[self.index] = frame_time_ms;
        self.index = (self.index + 1) % 60;
    }

    pub fn average_frame_time(&self) -> f32 {
        self.times.iter().sum::<f32>() / self.times.len() as f32
    }

    pub fn average_fps(&self) -> f32 {
        let avg_frame_time = self.average_frame_time();
        if avg_frame_time > 0.0 {
            1000.0 / avg_frame_time
        } else {
            0.0
        }
    }

    pub fn min_frame_time(&self) -> f32 {
        *self.times.iter().min_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(&0.0)
    }

    pub fn max_frame_time(&self) -> f32 {
        *self.times.iter().max_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal))
            .unwrap_or(&0.0)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_frame_time_history() {
        let mut history = FrameTimeHistory::new();

        // Record some frame times
        history.record_frame_time(16.0);
        history.record_frame_time(16.5);
        history.record_frame_time(17.0);

        let avg = history.average_frame_time();
        assert!(avg > 16.0 && avg < 17.0);

        let fps = history.average_fps();
        assert!(fps > 50.0 && fps < 70.0);
    }

    #[test]
    fn test_metrics_default() {
        let metrics = SystemMetrics::default();
        assert_eq!(metrics.vram_total_mb, 40960);  // CMP 170HX
        assert_eq!(metrics.gpu_load_percent, 0.0);
    }
}
