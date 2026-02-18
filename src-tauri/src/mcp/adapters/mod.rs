pub mod claude;
pub mod codex;
pub mod cursor;
pub mod gemini;
pub mod opencode;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Status of an AI tool integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterStatus {
    pub tool_name: String,
    pub installed: bool,
    pub connected: bool,
    pub config_path: Option<String>,
    pub verification: Option<String>,
}

/// Trait for MCP config adapters — each AI tool implements this.
pub trait McpConfigAdapter {
    /// Human-readable tool name (e.g. "Claude Code").
    fn name(&self) -> &str;

    /// Check if the tool is installed on this system.
    fn is_installed(&self) -> bool;

    /// Path to the tool's MCP config file.
    fn config_path(&self) -> Option<PathBuf>;

    /// Whether CodeEye is already registered in the tool's config.
    fn is_connected(&self) -> bool;

    /// Add CodeEye to the tool's MCP config. Backs up config first.
    fn connect(&self, binary_path: &str) -> Result<(), String>;

    /// Remove CodeEye from the tool's MCP config.
    fn disconnect(&self) -> Result<(), String>;

    /// Run the tool's verify command and return its output.
    fn verify(&self) -> Result<String, String> {
        Err("Verification not available for this tool".into())
    }

    /// Path to the config backup file (e.g. `.json.bak` or `.toml.bak`).
    /// Returns `None` if no backup exists. Uses `derive_backup_path` for consistency
    /// with the write helpers that create backups.
    fn backup_path(&self) -> Option<PathBuf> {
        let config = self.config_path()?;
        let backup = derive_backup_path(&config);
        if backup.exists() {
            Some(backup)
        } else {
            None
        }
    }

    /// Restore the config backup, overwriting the current config.
    fn restore_backup(&self) -> Result<(), String> {
        let config = self
            .config_path()
            .ok_or("No config path for this tool")?;
        let backup = self
            .backup_path()
            .ok_or("No backup file found")?;
        std::fs::copy(&backup, &config)
            .map_err(|e| format!("Failed to restore backup: {e}"))?;
        Ok(())
    }

    /// Full uninstall: disconnect CodeEye + remove the backup file.
    fn uninstall(&self) -> Result<(), String> {
        if self.is_connected() {
            self.disconnect()?;
        }
        if let Some(backup) = self.backup_path() {
            std::fs::remove_file(&backup)
                .map_err(|e| format!("Failed to remove backup: {e}"))?;
        }
        Ok(())
    }

    /// Read the current CodeEye MCP entry as a formatted string.
    fn read_entry(&self) -> Result<String, String> {
        Err("Not available for this tool".into())
    }

    /// Return full status (fast — no shell commands).
    fn status(&self) -> AdapterStatus {
        AdapterStatus {
            tool_name: self.name().to_string(),
            installed: self.is_installed(),
            connected: self.is_connected(),
            config_path: self.config_path().map(|p| p.to_string_lossy().to_string()),
            verification: None,
        }
    }
}

/// Returns all available adapters.
pub fn all_adapters() -> Vec<Box<dyn McpConfigAdapter>> {
    vec![
        Box::new(claude::ClaudeAdapter),
        Box::new(gemini::GeminiAdapter),
        Box::new(codex::CodexAdapter),
        Box::new(opencode::OpenCodeAdapter),
        Box::new(cursor::CursorAdapter),
    ]
}

/// Scan all adapters and return their statuses (fast, no shell commands).
pub fn scan_all() -> Vec<AdapterStatus> {
    all_adapters().iter().map(|a| a.status()).collect()
}

// ── Shared constants ────────────────────────────────────────────────

pub const MCP_SERVER_KEY: &str = "codeeye";

// ── Detection helper ────────────────────────────────────────────────

/// Check if a binary exists on `$PATH` via `which`.
pub fn which_exists(binary: &str) -> bool {
    std::process::Command::new("which")
        .arg(binary)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Run a shell command and return its stdout.
pub fn run_verify_command(cmd: &str, args: &[&str]) -> Result<String, String> {
    let output = std::process::Command::new(cmd)
        .args(args)
        .output()
        .map_err(|e| format!("Failed to run `{cmd}`: {e}"))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(format!("{stdout}{stderr}").trim().to_string())
    }
}

// ── JSON config helpers ─────────────────────────────────────────────

/// Read a JSON config file, returning an empty object if missing.
pub fn read_json_config(path: &std::path::Path) -> Result<serde_json::Value, String> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let data =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;
    if data.trim().is_empty() {
        return Ok(serde_json::json!({}));
    }
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse config: {e}"))
}

/// Derive the backup path for a config file (e.g. `config.json` → `config.json.bak`).
/// Used by both the `McpConfigAdapter` trait and the write helpers to ensure consistency.
pub fn derive_backup_path(config_path: &std::path::Path) -> PathBuf {
    let ext = config_path
        .extension()
        .unwrap_or_default()
        .to_string_lossy();
    config_path.with_extension(format!("{ext}.bak"))
}

/// Write JSON config with backup.
pub fn write_json_config(
    path: &std::path::Path,
    value: &serde_json::Value,
) -> Result<(), String> {
    if path.exists() {
        let backup = derive_backup_path(path);
        std::fs::copy(path, &backup)
            .map_err(|e| format!("Failed to backup config: {e}"))?;
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write config: {e}"))
}

// ── TOML config helpers (for Codex) ─────────────────────────────────

/// Read a TOML config file, returning an empty table if missing.
pub fn read_toml_config(path: &std::path::Path) -> Result<toml::Value, String> {
    if !path.exists() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    let data =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;
    if data.trim().is_empty() {
        return Ok(toml::Value::Table(toml::map::Map::new()));
    }
    data.parse::<toml::Value>()
        .map_err(|e| format!("Failed to parse TOML config: {e}"))
}

/// Write TOML config with backup.
pub fn write_toml_config(
    path: &std::path::Path,
    value: &toml::Value,
) -> Result<(), String> {
    if path.exists() {
        let backup = derive_backup_path(path);
        std::fs::copy(path, &backup)
            .map_err(|e| format!("Failed to backup config: {e}"))?;
    }

    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    let toml_str = toml::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize TOML config: {e}"))?;
    std::fs::write(path, toml_str).map_err(|e| format!("Failed to write config: {e}"))
}
