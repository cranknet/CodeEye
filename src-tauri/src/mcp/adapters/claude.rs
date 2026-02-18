use super::{
    mcp_server_entry, read_json_config, write_json_config, McpConfigAdapter, MCP_SERVER_KEY,
};
use std::path::PathBuf;

pub struct ClaudeAdapter;

impl ClaudeAdapter {
    /// Claude Desktop config path (platform-specific):
    /// - macOS: ~/Library/Application Support/Claude/claude_desktop_config.json
    /// - Linux: ~/.config/Claude/claude_desktop_config.json
    /// - Windows: %APPDATA%/Claude/claude_desktop_config.json
    fn desktop_config_path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_default()
            .join("Claude")
            .join("claude_desktop_config.json")
    }
}

impl McpConfigAdapter for ClaudeAdapter {
    fn name(&self) -> &str {
        "Claude Code"
    }

    fn is_installed(&self) -> bool {
        // Check for Claude Desktop config directory
        let config_dir = dirs::config_dir()
            .unwrap_or_default()
            .join("Claude");
        config_dir.exists()
    }

    fn config_path(&self) -> Option<PathBuf> {
        let path = Self::desktop_config_path();
        Some(path)
    }

    fn is_connected(&self) -> bool {
        let path = Self::desktop_config_path();
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
        let path = Self::desktop_config_path();
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
        let path = Self::desktop_config_path();
        let mut config = read_json_config(&path)?;

        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            servers.remove(MCP_SERVER_KEY);
        }

        write_json_config(&path, &config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_connect_creates_mcp_entry() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("claude_desktop_config.json");

        // Write empty config
        std::fs::write(&config_path, "{}").unwrap();

        // Manually do what connect does, but with our test path
        let mut config = read_json_config(&config_path).unwrap();
        let servers = config
            .as_object_mut()
            .unwrap()
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));
        servers
            .as_object_mut()
            .unwrap()
            .insert(MCP_SERVER_KEY.to_string(), mcp_server_entry("/usr/bin/codeeye"));
        write_json_config(&config_path, &config).unwrap();

        // Verify
        let result = read_json_config(&config_path).unwrap();
        assert!(result["mcpServers"]["codeeye"]["command"]
            .as_str()
            .unwrap()
            .contains("codeeye"));
        assert_eq!(result["mcpServers"]["codeeye"]["args"][0], "--mcp");
    }

    #[test]
    fn test_disconnect_removes_mcp_entry() {
        let tmp = TempDir::new().unwrap();
        let config_path = tmp.path().join("claude_desktop_config.json");

        // Write config with codeeye entry
        let config = serde_json::json!({
            "mcpServers": {
                "codeeye": { "command": "/usr/bin/codeeye", "args": ["--mcp"] },
                "other": { "command": "/usr/bin/other" }
            }
        });
        write_json_config(&config_path, &config).unwrap();

        // Remove codeeye entry
        let mut config = read_json_config(&config_path).unwrap();
        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            servers.remove(MCP_SERVER_KEY);
        }
        write_json_config(&config_path, &config).unwrap();

        // Verify codeeye removed, other preserved
        let result = read_json_config(&config_path).unwrap();
        assert!(result["mcpServers"].get("codeeye").is_none());
        assert!(result["mcpServers"]["other"]["command"].is_string());
    }
}
