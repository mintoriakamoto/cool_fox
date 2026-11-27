//! Windows named pipe IPC and Unix socket tests

#[cfg(test)]
mod ipc_protocol_tests {
    #[test]
    fn test_frame_json_protocol() {
        // Test that the JSON protocol is correctly structured for IPC
        let frame_json = r#"{"timestamp_ms":1000,"fps":60.0,"gpu_load_percent":75.0,"vram_used_mb":30000,"vram_total_mb":40960,"cpu_load_percent":50.0,"gpu_temp_celsius":70.0,"frame_time_ms":16.667,"alerts":[]}"#;

        let parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(frame_json);
        assert!(parsed.is_ok(), "Frame JSON must be parseable");

        let obj = parsed.unwrap();
        assert_eq!(obj["timestamp_ms"], 1000);
        assert_eq!(obj["fps"], 60.0);
        assert_eq!(obj["gpu_load_percent"], 75.0);
        assert_eq!(obj["vram_used_mb"], 30000);
    }

    #[test]
    fn test_newline_delimited_frames() {
        // Test that frames can be sent as JSON lines (newline-delimited)
        let frame1 = r#"{"timestamp_ms":1000,"fps":60.0,"gpu_load_percent":75.0,"vram_used_mb":30000,"vram_total_mb":40960,"cpu_load_percent":50.0,"gpu_temp_celsius":70.0,"frame_time_ms":16.667,"alerts":[]}"#;
        let frame2 = r#"{"timestamp_ms":1016,"fps":60.0,"gpu_load_percent":76.0,"vram_used_mb":30100,"vram_total_mb":40960,"cpu_load_percent":51.0,"gpu_temp_celsius":71.0,"frame_time_ms":16.667,"alerts":[]}"#;

        let stream = format!("{}\n{}\n", frame1, frame2);

        // Parse as lines
        let lines: Vec<&str> = stream.lines().collect();
        assert_eq!(lines.len(), 2);

        for line in lines {
            let _parsed: serde_json::Result<serde_json::Value> = serde_json::from_str(line);
            assert!(_parsed.is_ok());
        }
    }
}

#[cfg(unix)]
#[cfg(test)]
mod unix_socket_tests {
    use std::os::unix::net::UnixListener;

    #[test]
    fn test_unix_socket_creation() {
        let socket_path = "/tmp/test-victory-ipc.sock";
        let _ = std::fs::remove_file(socket_path);

        // Test that we can create a Unix socket
        let result = UnixListener::bind(socket_path);
        assert!(result.is_ok(), "Should be able to create Unix socket");

        let _ = std::fs::remove_file(socket_path);
    }
}

#[cfg(windows)]
#[cfg(test)]
mod windows_named_pipe_tests {
    #[test]
    fn test_named_pipe_path_format() {
        // Test that named pipe path format is correct
        let pipe_name = format!("\\\\.\\pipe\\victory-game-{}", std::process::id());

        // Should follow Windows named pipe format
        assert!(pipe_name.starts_with("\\\\.\\pipe\\"));
        assert!(pipe_name.contains("victory-game-"));

        println!("Named pipe path format: {}", pipe_name);
    }
}
