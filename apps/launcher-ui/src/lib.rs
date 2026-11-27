//! Launcher UI library: session browsing and analysis

use std::path::{Path, PathBuf};
use victory_core::{session_persistence, session_analysis, ProfileFrame};
use anyhow::Result;

pub struct SessionBrowser;

impl SessionBrowser {
    pub fn list_all() -> Result<Vec<SessionInfo>> {
        let sessions = session_persistence::list_sessions()?;

        let mut infos = Vec::new();
        for path in sessions {
            if let Ok(info) = Self::load_session_info(&path) {
                infos.push(info);
            }
        }

        // Sort by date descending (newest first)
        infos.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        Ok(infos)
    }

    fn load_session_info(path: &Path) -> Result<SessionInfo> {
        let frames = session_persistence::load_session(path)?;

        // Extract game name from filename
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let game_name = filename
            .split('-')
            .next()
            .unwrap_or("unknown")
            .to_string();

        // Parse timestamp from filename (format: GAME-YYYY-MM-DD-HH-MM-SS.json)
        let parts: Vec<&str> = filename.split('-').collect();
        let timestamp = if parts.len() >= 7 {
            format!("{}-{}-{} {}:{}:{}",
                parts[1], parts[2], parts[3], parts[4], parts[5],
                parts[6].split('.').next().unwrap_or("00"))
        } else {
            "unknown".to_string()
        };

        let report = session_analysis::analyze_session(&game_name, &frames);

        Ok(SessionInfo {
            game_name,
            timestamp,
            frame_count: frames.len(),
            duration_secs: report.session_duration_secs,
            fps_avg: report.fps_avg,
            fps_min: report.fps_min,
            fps_max: report.fps_max,
            gpu_load_avg: report.gpu_load_avg,
            gpu_temp_max: report.gpu_temp_max,
            path: path.to_path_buf(),
        })
    }

    pub fn get_session_report(path: &Path) -> Result<String> {
        let frames = session_persistence::load_session(path)?;

        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or("unknown");
        let game_name = filename
            .split('-')
            .next()
            .unwrap_or("unknown")
            .to_string();

        let report = session_analysis::analyze_session(&game_name, &frames);
        Ok(report.pretty_print())
    }
}

#[derive(Clone, Debug)]
pub struct SessionInfo {
    pub game_name: String,
    pub timestamp: String,
    pub frame_count: usize,
    pub duration_secs: f32,
    pub fps_avg: f32,
    pub fps_min: f32,
    pub fps_max: f32,
    pub gpu_load_avg: f32,
    pub gpu_temp_max: f32,
    pub path: PathBuf,
}

impl SessionInfo {
    pub fn summary_line(&self) -> String {
        format!(
            "{:<20} | {:<19} | {:>4}s | {:>5.1} fps avg | GPU {:.0}% | {:.0}°C",
            self.game_name,
            self.timestamp,
            self.frame_count / 60,  // Approximate seconds
            self.fps_avg,
            self.gpu_load_avg,
            self.gpu_temp_max,
        )
    }
}
