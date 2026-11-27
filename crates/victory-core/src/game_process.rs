//! Game process manager: launch, monitor, profile, track sessions

use std::process::{Child, Command};
use std::time::Instant;
use crate::profiler_client::ProfilerClient;

/// Metadata about a game being played
#[derive(Clone, Debug)]
pub struct GameMetadata {
    pub id: String,
    pub title: String,
    pub path: String,
    pub executable: String,
    pub playtime_hours: f32,
    pub last_played: Option<u64>,
}

/// Configuration for launching a game
#[derive(Clone, Debug)]
pub struct LaunchConfig {
    pub game: GameMetadata,
    pub graphics: GraphicsSettings,
    pub performance: PerformanceSettings,
}

#[derive(Clone, Debug)]
pub struct GraphicsSettings {
    pub resolution_width: u32,
    pub resolution_height: u32,
    pub texture_detail: TextureDetail,
    pub shader_quality: ShaderQuality,
    pub ray_tracing: bool,
}

#[derive(Clone, Debug, Default)]
pub enum TextureDetail {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

#[derive(Clone, Debug, Default)]
pub enum ShaderQuality {
    Low,
    Medium,
    #[default]
    High,
    Ultra,
}

#[derive(Clone, Debug)]
pub struct PerformanceSettings {
    pub fps_cap: u32,
    pub vram_target_mb: u64,
    pub g_sync_enabled: bool,
}

impl Default for GraphicsSettings {
    fn default() -> Self {
        Self {
            resolution_width: 3840,
            resolution_height: 2160,
            texture_detail: TextureDetail::High,
            shader_quality: ShaderQuality::High,
            ray_tracing: true,
        }
    }
}

impl Default for PerformanceSettings {
    fn default() -> Self {
        Self {
            fps_cap: 60,
            vram_target_mb: 32000,  // 32 GB for 40 GB GPU
            g_sync_enabled: true,
        }
    }
}

/// Active game session
pub struct GameSession {
    pub process: Child,
    pub metadata: GameMetadata,
    pub started_at: Instant,
    pub profiler: ProfilerClient,
}

impl GameSession {
    /// Launch a game with given configuration
    pub fn launch(config: LaunchConfig) -> anyhow::Result<Self> {
        // Build launch environment variables
        let mut env_vars = vec![
            ("VICTORY_GAME_DIR".to_string(), config.game.path.clone()),
            ("VICTORY_TEXTURE_STAGE_MB".to_string(), "128".to_string()),
            ("VICTORY_DSTORAGE_STAGING_MB".to_string(), "128".to_string()),
            ("VICTORY_UPLOAD_AHEAD".to_string(), "1".to_string()),
            ("VICTORY_TEXTURE_NO_WAIT".to_string(), "1".to_string()),
            ("VICTORY_PORT_CONSTANTS".to_string(), "1".to_string()),
            ("VICTORY_PORT_DESCRIPTORS".to_string(), "1".to_string()),
            ("VICTORY_PORT_GUARD".to_string(), "1".to_string()),
            ("VICTORY_PORT_JOURNAL".to_string(), "1".to_string()),
            ("VICTORY_FPS_CAP".to_string(), config.performance.fps_cap.to_string()),
            ("VICTORY_PROFILE".to_string(), "1".to_string()),  // Enable profiling
            ("VICTORY_VRAM_TARGET_MB".to_string(), config.performance.vram_target_mb.to_string()),
            // NVIDIA-specific tuning
            ("VKD3D_DESCRIPTOR_QA".to_string(), "0".to_string()),
            ("VKD3D_FORCE_DISCRETE_GPU".to_string(), "1".to_string()),
            ("VKD3D_MAX_FRAME_LATENCY".to_string(), "1".to_string()),
            ("NV_GPU".to_string(), "0".to_string()),
            ("__NV_PRIME_RENDER_OFFLOAD".to_string(), "1".to_string()),
        ];

        // Add graphics settings as env vars
        env_vars.push(("GAME_RESOLUTION".to_string(),
            format!("{}x{}", config.graphics.resolution_width, config.graphics.resolution_height)));
        env_vars.push(("GAME_TEXTURE_DETAIL".to_string(),
            format!("{:?}", config.graphics.texture_detail)));
        env_vars.push(("GAME_SHADER_QUALITY".to_string(),
            format!("{:?}", config.graphics.shader_quality)));
        env_vars.push(("GAME_RAY_TRACING".to_string(),
            if config.graphics.ray_tracing { "1" } else { "0" }.to_string()));
        env_vars.push(("GAME_G_SYNC".to_string(),
            if config.performance.g_sync_enabled { "1" } else { "0" }.to_string()));

        // Build command to launch game
        let mut cmd = Command::new("wine");
        cmd.arg("/opt/victory/victory.exe")
            .arg("run")
            .arg(format!("{}/{}", config.game.path, config.game.executable));

        // Set environment variables
        for (key, value) in env_vars {
            cmd.env(&key, &value);
        }

        // Spawn process
        let process = cmd.spawn()?;
        let pid = process.id();

        // Create profiler client for this game session
        let profiler = ProfilerClient::new(pid);
        profiler.start_reading();

        Ok(GameSession {
            process,
            metadata: config.game,
            started_at: Instant::now(),
            profiler,
        })
    }

    /// Get session duration in seconds
    pub fn duration_seconds(&self) -> u64 {
        self.started_at.elapsed().as_secs()
    }

    /// Get playtime from start plus previous playtime
    pub fn total_playtime_hours(&self) -> f32 {
        self.metadata.playtime_hours + (self.duration_seconds() as f32 / 3600.0)
    }

    /// Wait for game to exit, return exit code
    pub fn wait_for_exit(&mut self) -> anyhow::Result<i32> {
        let status = self.process.wait()?;

        // Stop profiler reading
        self.profiler.stop_reading();

        // Return exit code (0 = normal, non-zero = crash)
        Ok(status.code().unwrap_or(1))
    }

    /// Get latest profiling data
    pub fn latest_profile(&self) -> Option<crate::profiler_client::ProfileFrame> {
        self.profiler.latest_frame()
    }

    /// Get session statistics
    pub fn session_stats(&self) -> Option<crate::profiler_client::SessionStats> {
        self.profiler.session_stats()
    }

    /// Terminate game process (force stop)
    pub fn terminate(&mut self) -> anyhow::Result<()> {
        self.profiler.stop_reading();
        self.process.kill()?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_launch_config_defaults() {
        let game = GameMetadata {
            id: "gta6".to_string(),
            title: "GTA VI".to_string(),
            path: "/home/user/.local/share/launcher/games/gta6".to_string(),
            executable: "game_scarlett_master.exe".to_string(),
            playtime_hours: 42.5,
            last_played: Some(1700000000),
        };

        let config = LaunchConfig {
            game: game.clone(),
            graphics: GraphicsSettings::default(),
            performance: PerformanceSettings::default(),
        };

        assert_eq!(config.graphics.resolution_width, 3840);
        assert_eq!(config.performance.fps_cap, 60);
        assert_eq!(config.performance.vram_target_mb, 32000);
    }

    #[test]
    fn test_session_duration() {
        let game = GameMetadata {
            id: "test".to_string(),
            title: "Test".to_string(),
            path: "/test".to_string(),
            executable: "test.exe".to_string(),
            playtime_hours: 10.0,
            last_played: None,
        };

        let session = GameSession {
            process: unsafe { std::mem::zeroed() },  // Unsafe for testing only
            metadata: game,
            started_at: Instant::now(),
            profiler: ProfilerClient::new(0),
        };

        let duration = session.duration_seconds();
        assert!(duration >= 0);
    }
}
