//! Game profiling session analysis

use std::path::Path;
use anyhow::Result;
use victory_core::session_persistence;
use victory_core::session_analysis;

pub fn list_sessions() -> Result<()> {
    let sessions = session_persistence::list_sessions()?;

    if sessions.is_empty() {
        println!("No profiling sessions found in ~/.local/share/launcher/profiles/");
        return Ok(());
    }

    println!("┌─ Profiling Sessions ({}) ─────────────────────────────┐", sessions.len());
    for (i, session_path) in sessions.iter().enumerate() {
        if let Some(filename) = session_path.file_name() {
            println!("  {}. {:?}", i + 1, filename);
        }
    }
    println!("└────────────────────────────────────────────────────┘");
    Ok(())
}

pub fn analyze_session(session_path: &Path) -> Result<()> {
    let frames = session_persistence::load_session(session_path)?;

    // Extract game name from filename (format: GAME-YYYY-MM-DD-HH-MM-SS.json)
    let filename = session_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let game_name = filename
        .split('-')
        .next()
        .unwrap_or("unknown")
        .to_string();

    let report = session_analysis::analyze_session(&game_name, &frames);

    println!();
    println!("{}", report.pretty_print());
    println!();

    Ok(())
}

pub fn analyze_latest() -> Result<()> {
    let sessions = session_persistence::list_sessions()?;

    if sessions.is_empty() {
        println!("No profiling sessions found.");
        return Ok(());
    }

    // Most recently modified is last in the list (sorted by filesystem order)
    if let Some(latest) = sessions.last() {
        analyze_session(latest)?;
    }

    Ok(())
}

pub fn export_csv(session_path: &Path, output_path: &Path) -> Result<()> {
    let frames = session_persistence::load_session(session_path)?;

    let csv_header = "timestamp_ms,fps,gpu_load_percent,vram_used_mb,vram_total_mb,cpu_load_percent,gpu_temp_celsius,frame_time_ms\n";

    let csv_rows: Vec<String> = frames
        .iter()
        .map(|f| {
            format!(
                "{},{:.1},{:.1},{},{},{:.1},{:.1},{:.3}",
                f.timestamp_ms,
                f.fps,
                f.gpu_load_percent,
                f.vram_used_mb,
                f.vram_total_mb,
                f.cpu_load_percent,
                f.gpu_temp_celsius,
                f.frame_time_ms
            )
        })
        .collect();

    let csv_content = format!("{}{}", csv_header, csv_rows.join("\n"));

    std::fs::write(output_path, csv_content)?;
    println!("Exported {} frames to {:?}", frames.len(), output_path);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_game_name_extraction() {
        let filename = "GTA6-2024-01-15-14-30-45.json";
        let game_name = filename.split('-').next().unwrap_or("unknown");
        assert_eq!(game_name, "GTA6");
    }
}
