//! End-to-end integration test: game profiler to launcher client

use crate::{ProfileFrame, SessionStats, ProfilerClient};
use std::io::{Write, BufRead, BufReader};

#[cfg(target_os = "linux")]
#[test]
fn test_game_to_launcher_pipeline() {
    use std::os::unix::net::UnixListener;
    use std::thread;
    use std::time::Duration;

    let socket_path = format!("/tmp/victory-integration-{}.sock", std::process::id());
    let _ = std::fs::remove_file(&socket_path);

    // Server: simulate game writing frames
    let socket_path_clone = socket_path.clone();
    let server = thread::spawn(move || {
        if let Ok(listener) = UnixListener::bind(&socket_path_clone) {
            if let Ok((mut stream, _)) = listener.accept() {
                for i in 0..3 {
                    let frame = ProfileFrame {
                        timestamp_ms: 1700000000000 + (i * 16) as u64,
                        fps: 60.0 - (i as f32 * 0.5),
                        gpu_load_percent: 70.0 + (i as f32 * 2.0),
                        vram_used_mb: 30000 + (i as u64 * 500),
                        vram_total_mb: 40960,
                        cpu_load_percent: 50.0 + (i as f32 * 1.0),
                        gpu_temp_celsius: 70.0 + (i as f32),
                        frame_time_ms: 16.667,
                        alerts: if i == 2 { vec!["high_temp".to_string()] } else { vec![] },
                    };

                    if let Ok(json) = serde_json::to_string(&frame) {
                        let _ = stream.write_all(format!("{}\n", json).as_bytes());
                    }

                    thread::sleep(Duration::from_millis(10));
                }
            }
        }
    });

    thread::sleep(Duration::from_millis(50));

    // Client: connect and read frames
    if let Ok(stream) = std::os::unix::net::UnixStream::connect(&socket_path) {
        let reader = BufReader::new(stream);
        let mut stats = SessionStats::default();
        let mut frame_count = 0;

        for line in reader.lines().take(3) {
            if let Ok(line_str) = line {
                if let Ok(frame) = serde_json::from_str::<ProfileFrame>(&line_str) {
                    assert!(frame.fps > 0.0);
                    assert!(frame.gpu_load_percent >= 0.0 && frame.gpu_load_percent <= 100.0);
                    assert!(frame.vram_used_mb <= frame.vram_total_mb);

                    ProfilerClient::update_stats(&mut stats, &frame);
                    frame_count += 1;
                }
            }
        }

        assert_eq!(frame_count, 3);
        assert_eq!(stats.frame_count, 3);
        assert!(stats.fps_avg > 0.0);
        assert!(stats.gpu_load_peak >= 70.0);
        assert!(stats.alerts_total >= 1);
        assert!(!stats.bottleneck.is_empty());
    }

    let _ = std::fs::remove_file(&socket_path);
    let _ = server.join();
}

#[test]
fn test_frame_serialization_format() {
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

    let json = serde_json::to_string(&frame).expect("serialization failed");
    let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse failed");
    assert_eq!(parsed["fps"], 58.3);
    assert_eq!(parsed["vram_used_mb"], 32456);
    assert_eq!(parsed["alerts"][0], "vram_at_80_percent");

    let mut line = json.clone();
    line.push('\n');
    assert!(line.ends_with('\n'));

    let trimmed = line.trim_end();
    let frame2: ProfileFrame = serde_json::from_str(trimmed).expect("deserialize failed");
    assert_eq!(frame2.fps, frame.fps);
    assert_eq!(frame2.alerts.len(), frame.alerts.len());
}
