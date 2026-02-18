use super::{
    mcp_server_entry, read_json_config, write_json_config, McpConfigAdapter, MCP_SERVER_KEY,
};
use std::path::PathBuf;

pub struct GeminiAdapter;

impl GeminiAdapter {
    /// Gemini CLI config: ~/.gemini/config.json
    fn config_path_inner() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".gemini")
            .join("settings.json")
    }
}

impl McpConfigAdapter for GeminiAdapter {
    fn name(&self) -> &str {
        "Gemini CLI"
    }

    fn is_installed(&self) -> bool {
        let config_dir = dirs::home_dir()
            .unwrap_or_default()
            .join(".gemini");
        config_dir.exists()
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(Self::config_path_inner())
    }

    fn is_connected(&self) -> bool {
        let path = Self::config_path_inner();
        if let Ok(config) = read_json_config(&path) {
            config
                .get("mcpServers")
                .and_then(|s| s.get(MCP_SERVER_KEY))
                .is_some()
        } else {
            false
        }
    }

    fn connect(&self, binary_path: &str) -> Result<(), String> {
        let path = Self::config_path_inner();
        let mut config = read_json_config(&path)?;

        let servers = config
            .as_object_mut()
            .ok_or("Config is not an object")?
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));

        servers
            .as_object_mut()
            .ok_or("mcpServers is not an object")?
            .insert(MCP_SERVER_KEY.to_string(), mcp_server_entry(binary_path));

        write_json_config(&path, &config)
    }

    fn disconnect(&self) -> Result<(), String> {
        let path = Self::config_path_inner();
        let mut config = read_json_config(&path)?;

        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            servers.remove(MCP_SERVER_KEY);
        }

        write_json_config(&path, &config)
    }
}
