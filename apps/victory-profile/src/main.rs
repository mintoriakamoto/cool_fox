//! Victory Profile CLI - Working profiling command-line tool
//! Independent of shader translation (no broken dependencies)

use std::path::PathBuf;
use victory_core::{session_persistence, session_analysis};
use anyhow::Result;
use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "victory-profile",
    version,
    about = "Victory game launcher - profiling analysis tool",
    long_about = "Browse, analyze, and export game profiling sessions"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// List all saved profiling sessions
    List,
    /// Analyze a specific profiling session
    Analyze {
        /// Session file path or index
        session: String,
    },
    /// Show the most recent session
    Recent,
    /// Export session data as CSV
    Export {
        /// Input session path
        input: PathBuf,
        /// Output CSV file path
        output: PathBuf,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::List => cmd_list(),
        Commands::Analyze { session } => cmd_analyze(&session),
        Commands::Recent => cmd_recent(),
        Commands::Export { input, output } => cmd_export(&input, &output),
    }
}

fn cmd_list() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║            Victory Profile CLI - Sessions                  ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let sessions = session_persistence::list_sessions()?;

    if sessions.is_empty() {
        println!("No profiling sessions found.");
        println!("Sessions are saved to: ~/.local/share/launcher/profiles/");
        return Ok(());
    }

    println!("📊 Profiling Sessions ({})\n", sessions.len());
    println!("{:<20} | {:<19} | {:<6} | {:<10} | {:<8} | {:<6}",
        "Game", "Timestamp", "Frames", "FPS Avg", "GPU Load", "Temp");
    println!("{}", "─".repeat(90));

    for (i, session_path) in sessions.iter().enumerate() {
        if let Ok(frames) = session_persistence::load_session(session_path) {
            let filename = session_path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("unknown");
            let game_name = filename.split('-').next().unwrap_or("unknown");

            let report = session_analysis::analyze_session(game_name, &frames);
            let timestamp = filename.split('-').skip(1).take(3).collect::<Vec<_>>().join("-");

            println!("{}. {:<20} | {:<19} | {:>4}s | {:>5.1} fps | GPU {:.0}% | {:.0}°C",
                i + 1,
                game_name,
                timestamp,
                frames.len() / 60,
                report.fps_avg,
                report.gpu_load_avg,
                report.gpu_temp_max,
            );
        }
    }

    println!("\nUsage:");
    println!("  victory-profile analyze <index>  - Analyze session by number");
    println!("  victory-profile recent           - Show most recent session");
    println!("  victory-profile export <in> <out> - Export to CSV\n");

    Ok(())
}

fn cmd_analyze(session_ref: &str) -> Result<()> {
    let sessions = session_persistence::list_sessions()?;

    let session_path = if let Ok(idx) = session_ref.parse::<usize>() {
        if idx > 0 && idx <= sessions.len() {
            sessions[idx - 1].clone()
        } else {
            anyhow::bail!("Invalid session index: {}", idx);
        }
    } else {
        PathBuf::from(session_ref)
    };

    let frames = session_persistence::load_session(&session_path)?;
    let filename = session_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let game_name = filename.split('-').next().unwrap_or("unknown");

    let report = session_analysis::analyze_session(game_name, &frames);

    println!("\n{}\n", report.pretty_print());

    Ok(())
}

fn cmd_recent() -> Result<()> {
    let sessions = session_persistence::list_sessions()?;

    if sessions.is_empty() {
        println!("No profiling sessions found.");
        return Ok(());
    }

    cmd_analyze(&"1".to_string())
}

fn cmd_export(input: &PathBuf, output: &PathBuf) -> Result<()> {
    let frames = session_persistence::load_session(input)?;

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
    std::fs::write(output, csv_content)?;

    println!("✓ Exported {} frames to {:?}", frames.len(), output);
    Ok(())
}
