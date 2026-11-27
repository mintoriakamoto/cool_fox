//! Victory Launcher Web Server - Serves session data to dashboard

use victory_core::{session_persistence, session_analysis};
use serde_json::{json, Value};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Victory Launcher Web Server starting...");
    println!("📊 Available at: http://localhost:8080");
    println!("📁 Sessions from: ~/.local/share/launcher/profiles/\n");

    let app = axum::Router::new()
        .route("/api/sessions", axum::routing::get(list_sessions))
        .route("/api/sessions/:id/data", axum::routing::get(session_data))
        .route("/api/sessions/:id/metrics", axum::routing::get(session_metrics))
        .route("/", axum::routing::get(root))
        .into_make_service();

    let listener = tokio::net::TcpListener::bind("127.0.0.1:8080").await?;

    println!("✓ Server ready. Press Ctrl+C to stop.\n");
    axum::serve(listener, app).await?;

    Ok(())
}

async fn root() -> &'static str {
    r#"
    <!DOCTYPE html>
    <html>
    <head>
        <title>Victory Launcher</title>
        <style>
            body { font-family: sans-serif; margin: 40px; }
            h1 { color: #667eea; }
            a { color: #667eea; text-decoration: none; margin-right: 20px; }
            a:hover { text-decoration: underline; }
        </style>
    </head>
    <body>
        <h1>🎮 Victory Launcher</h1>
        <p>Web API endpoints:</p>
        <ul>
            <li><a href="/api/sessions">/api/sessions</a> - List all sessions</li>
            <li><a href="/dashboard.html">/dashboard.html</a> - Web dashboard</li>
        </ul>
        <p>Or use: <code>victory-profile list</code> for CLI</p>
    </body>
    </html>
    "#
}

async fn list_sessions() -> axum::Json<Value> {
    match session_persistence::list_sessions() {
        Ok(paths) => {
            let sessions: Vec<Value> = paths
                .iter()
                .enumerate()
                .filter_map(|(idx, path)| {
                    let filename = path.file_name()?.to_str()?.to_string();
                    let game_name = filename.split('-').next().unwrap_or("Unknown").to_string();

                    if let Ok(frames) = session_persistence::load_session(path) {
                        let report = session_analysis::analyze_session(&game_name, &frames);

                        Some(json!({
                            "id": idx,
                            "name": game_name,
                            "filename": filename,
                            "frames": report.total_frames,
                            "duration": report.session_duration_secs,
                            "fps_avg": report.fps_avg,
                            "fps_min": report.fps_min,
                            "fps_max": report.fps_max,
                            "gpu_load": report.gpu_load_avg,
                            "gpu_temp": report.gpu_temp_max,
                            "cpu_load": report.cpu_load_avg,
                            "bottleneck": &report.bottleneck_analysis,
                        }))
                    } else {
                        None
                    }
                })
                .collect();

            axum::Json(json!({
                "sessions": sessions,
                "count": sessions.len()
            }))
        }
        Err(_) => {
            axum::Json(json!({
                "sessions": [],
                "count": 0,
                "error": "No sessions found"
            }))
        }
    }
}


async fn session_data(
    axum::extract::Path(id): axum::extract::Path<usize>,
) -> Result<axum::Json<Value>, String> {
    let sessions = session_persistence::list_sessions()
        .map_err(|e| format!("Failed to list sessions: {}", e))?;

    if id >= sessions.len() {
        return Err("Session not found".to_string());
    }

    let frames = session_persistence::load_session(&sessions[id])
        .map_err(|e| format!("Failed to load session: {}", e))?;

    let filename = sessions[id]
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("unknown");
    let game_name = filename.split('-').next().unwrap_or("Unknown");

    let report = session_analysis::analyze_session(game_name, &frames);

    // Build trend arrays (sample every 10th frame for performance)
    let sample_rate = (frames.len() / 100).max(1);
    let fps_trend: Vec<f32> = frames.iter().step_by(sample_rate).map(|f| f.fps).collect();
    let gpu_trend: Vec<f32> = frames.iter().step_by(sample_rate).map(|f| f.gpu_load_percent).collect();
    let cpu_trend: Vec<f32> = frames.iter().step_by(sample_rate).map(|f| f.cpu_load_percent).collect();

    Ok(axum::Json(json!({
        "game": game_name,
        "duration": report.session_duration_secs,
        "frames": report.total_frames,
        "fps": {
            "avg": report.fps_avg,
            "min": report.fps_min,
            "max": report.fps_max,
            "trend": fps_trend,
        },
        "gpu": {
            "load": report.gpu_load_avg,
            "peak": report.gpu_load_peak,
            "temp": report.gpu_temp_max,
            "trend": gpu_trend,
        },
        "cpu": {
            "load": report.cpu_load_avg,
            "peak": report.cpu_load_peak,
            "trend": cpu_trend,
        },
        "vram": {
            "peak": report.vram_used_max_mb,
        },
        "bottleneck": &report.bottleneck_analysis,
        "alerts": report.alerts_count,
        "recommendations": report.recommendations,
    })))
}

async fn session_metrics(
    axum::extract::Path(id): axum::extract::Path<usize>,
) -> Result<axum::Json<Vec<Value>>, String> {
    let sessions = session_persistence::list_sessions()
        .map_err(|e| format!("Failed to list sessions: {}", e))?;

    if id >= sessions.len() {
        return Err("Session not found".to_string());
    }

    let frames = session_persistence::load_session(&sessions[id])
        .map_err(|e| format!("Failed to load session: {}", e))?;

    // Return frame-by-frame metrics (sampled for performance)
    let sample_rate = (frames.len() / 500).max(1);
    let metrics: Vec<Value> = frames
        .iter()
        .step_by(sample_rate)
        .map(|f| {
            json!({
                "timestamp": f.timestamp_ms,
                "fps": f.fps,
                "gpu_load": f.gpu_load_percent,
                "gpu_temp": f.gpu_temp_celsius,
                "cpu_load": f.cpu_load_percent,
                "vram": f.vram_used_mb,
                "alerts": f.alerts.len(),
            })
        })
        .collect();

    Ok(axum::Json(metrics))
}
