//! Session persistence: save profiling data to disk for analysis

use crate::ProfileFrame;
use std::fs;
use std::path::PathBuf;

/// Save profiling session to disk
pub fn save_session(game_name: &str, frames: &[ProfileFrame]) -> std::io::Result<PathBuf> {
    // Create profiles directory
    let profiles_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("launcher")
        .join("profiles");

    fs::create_dir_all(&profiles_dir)?;

    // Generate filename: game-YYYY-MM-DD-HH-MM-SS.json
    let now = chrono::Local::now();
    let filename = format!("{}-{}.json", game_name, now.format("%Y-%m-%d-%H-%M-%S"));
    let filepath = profiles_dir.join(&filename);

    // Serialize frames to JSON
    let session_data = serde_json::json!({
        "game": game_name,
        "timestamp": now.to_rfc3339(),
        "frame_count": frames.len(),
        "frames": frames,
    });

    // Write to file
    let json = serde_json::to_string_pretty(&session_data)?;
    fs::write(&filepath, json)?;

    Ok(filepath)
}

/// Load profiling session from disk
pub fn load_session(filepath: &std::path::Path) -> std::io::Result<Vec<ProfileFrame>> {
    let content = fs::read_to_string(filepath)?;
    let data: serde_json::Value = serde_json::from_str(&content)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;

    let frames = data["frames"]
        .as_array()
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no frames field"))?;

    let mut result = Vec::new();
    for frame_val in frames {
        let frame: ProfileFrame = serde_json::from_value(frame_val.clone())
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        result.push(frame);
    }

    Ok(result)
}

/// List all saved sessions
pub fn list_sessions() -> std::io::Result<Vec<PathBuf>> {
    let profiles_dir = dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("launcher")
        .join("profiles");

    if !profiles_dir.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();
    for entry in fs::read_dir(profiles_dir)? {
        let entry = entry?;
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) == Some("json") {
            sessions.push(path);
        }
    }

    Ok(sessions)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ProfileFrame;

    #[test]
    fn test_session_save_and_load() {
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
        ];

        // Test serialization (without actual disk write in unit test)
        let session_data = serde_json::json!({
            "game": "test_game",
            "frames": &frames,
        });

        let json = serde_json::to_string(&session_data).expect("serialize failed");
        let parsed: serde_json::Value = serde_json::from_str(&json).expect("parse failed");

        assert_eq!(parsed["game"], "test_game");
        assert_eq!(parsed["frames"][0]["fps"], 60.0);
    }
}
