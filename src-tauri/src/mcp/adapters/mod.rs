pub mod claude;
pub mod codex;
pub mod gemini;

use serde::{Deserialize, Serialize};
use std::path::PathBuf;

/// Status of an AI tool integration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AdapterStatus {
    pub tool_name: String,
    pub installed: bool,
    pub connected: bool,
    pub config_path: Option<String>,
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

    /// Return full status.
    fn status(&self, binary_path: &str) -> AdapterStatus {
        let _ = binary_path;
        AdapterStatus {
            tool_name: self.name().to_string(),
            installed: self.is_installed(),
            connected: self.is_connected(),
            config_path: self.config_path().map(|p| p.to_string_lossy().to_string()),
        }
    }
}

/// Returns all available adapters.
pub fn all_adapters() -> Vec<Box<dyn McpConfigAdapter>> {
    vec![
        Box::new(claude::ClaudeAdapter),
        Box::new(codex::CodexAdapter),
        Box::new(gemini::GeminiAdapter),
    ]
}

/// Scan all adapters and return their statuses.
pub fn scan_all(binary_path: &str) -> Vec<AdapterStatus> {
    all_adapters()
        .iter()
        .map(|a| a.status(binary_path))
        .collect()
}

/// Helper: read a JSON config file, returning an empty object if missing.
pub fn read_json_config(path: &std::path::Path) -> Result<serde_json::Value, String> {
    if !path.exists() {
        return Ok(serde_json::json!({}));
    }
    let data =
        std::fs::read_to_string(path).map_err(|e| format!("Failed to read config: {e}"))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse config: {e}"))
}

/// Helper: write JSON config with backup.
pub fn write_json_config(
    path: &std::path::Path,
    value: &serde_json::Value,
) -> Result<(), String> {
    // Backup existing file
    if path.exists() {
        let backup = path.with_extension("json.bak");
        std::fs::copy(path, &backup)
            .map_err(|e| format!("Failed to backup config: {e}"))?;
    }

    // Ensure parent directory exists
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)
            .map_err(|e| format!("Failed to create config dir: {e}"))?;
    }

    let json = serde_json::to_string_pretty(value)
        .map_err(|e| format!("Failed to serialize config: {e}"))?;
    std::fs::write(path, json).map_err(|e| format!("Failed to write config: {e}"))
}

const MCP_SERVER_KEY: &str = "codeeye";

/// Build the MCP server entry for config files.
pub fn mcp_server_entry(binary_path: &str) -> serde_json::Value {
    serde_json::json!({
        "command": binary_path,
        "args": ["--mcp"]
    })
}
