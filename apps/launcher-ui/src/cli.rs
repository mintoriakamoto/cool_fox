//! Launcher UI CLI: Browse and analyze profiling sessions

use std::path::PathBuf;
use launcher_ui::SessionBrowser;
use anyhow::Result;

fn main() -> Result<()> {
    println!("╔════════════════════════════════════════════════════════════╗");
    println!("║            Victory Game Launcher - Session Browser         ║");
    println!("╚════════════════════════════════════════════════════════════╝\n");

    let sessions = SessionBrowser::list_all()?;

    if sessions.is_empty() {
        println!("No profiling sessions found.");
        println!("Sessions are saved to: ~/.local/share/launcher/profiles/");
        return Ok(());
    }

    println!("📊 Profiling Sessions ({})\n", sessions.len());
    println!("{:<20} | {:<19} | {:<6} | {:<10} | {:<8} | {:<6}",
        "Game", "Timestamp", "Frames", "FPS Avg", "GPU Load", "Temp");
    println!("{}", "─".repeat(90));

    for (i, session) in sessions.iter().enumerate() {
        println!("{}. {}", i + 1, session.summary_line());
    }

    println!("\n📌 Usage:");
    println!("  launcher-ui list    - Show all sessions");
    println!("  launcher-ui view <N> - View detailed report for session N");
    println!("  launcher-ui recent   - View most recent session\n");

    // If interactive mode requested
    if std::env::args().any(|arg| arg == "recent") && !sessions.is_empty() {
        println!("\n📈 Most Recent Session Report:\n");
        if let Ok(report) = SessionBrowser::get_session_report(&sessions[0].path) {
            println!("{}", report);
        }
    }

    Ok(())
}
