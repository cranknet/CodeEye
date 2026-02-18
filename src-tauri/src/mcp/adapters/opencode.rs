use super::{
    read_json_config, run_verify_command, which_exists, write_json_config, McpConfigAdapter,
    MCP_SERVER_KEY,
};
use std::path::PathBuf;

pub struct OpenCodeAdapter;

impl OpenCodeAdapter {
    /// OpenCode config: ~/.config/opencode/opencode.json
    fn config_path_inner() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_default()
            .join("opencode")
            .join("opencode.json")
    }

    /// OpenCode uses "command" as an array and "type": "local".
    fn mcp_entry(binary_path: &str) -> serde_json::Value {
        serde_json::json!({
            "type": "local",
            "command": [binary_path, "--mcp"],
            "enabled": true
        })
    }
}

impl McpConfigAdapter for OpenCodeAdapter {
    fn name(&self) -> &str {
        "OpenCode"
    }

    fn is_installed(&self) -> bool {
        which_exists("opencode")
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(Self::config_path_inner())
    }

    fn is_connected(&self) -> bool {
        let path = Self::config_path_inner();
        if let Ok(config) = read_json_config(&path) {
            config
                .get("mcp")
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
            .entry("mcp")
            .or_insert_with(|| serde_json::json!({}));

        servers
            .as_object_mut()
            .ok_or("mcp is not an object")?
            .insert(MCP_SERVER_KEY.to_string(), Self::mcp_entry(binary_path));

        write_json_config(&path, &config)
    }

    fn disconnect(&self) -> Result<(), String> {
        let path = Self::config_path_inner();
        let mut config = read_json_config(&path)?;

        if let Some(servers) = config.get_mut("mcp").and_then(|s| s.as_object_mut()) {
            servers.remove(MCP_SERVER_KEY);
        }

        write_json_config(&path, &config)
    }

    fn verify(&self) -> Result<String, String> {
        run_verify_command("opencode", &["mcp", "list"])
    }

    fn read_entry(&self) -> Result<String, String> {
        let config = read_json_config(&Self::config_path_inner())?;
        let entry = config
            .get("mcp")
            .and_then(|s| s.get(MCP_SERVER_KEY))
            .ok_or("CodeEye entry not found in config")?;
        serde_json::to_string_pretty(entry).map_err(|e| format!("Failed to format entry: {e}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_creates_mcp_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("opencode.json");

        std::fs::write(&config_path, "{}").unwrap();

        let mut config = read_json_config(&config_path).unwrap();
        let servers = config
            .as_object_mut()
            .unwrap()
            .entry("mcp")
            .or_insert_with(|| serde_json::json!({}));
        servers
            .as_object_mut()
            .unwrap()
            .insert(
                MCP_SERVER_KEY.to_string(),
                OpenCodeAdapter::mcp_entry("/usr/bin/codeeye"),
            );
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        assert_eq!(result["mcp"]["codeeye"]["type"], "local");
        assert_eq!(result["mcp"]["codeeye"]["enabled"], true);
        let cmd = result["mcp"]["codeeye"]["command"].as_array().unwrap();
        assert_eq!(cmd[0].as_str().unwrap(), "/usr/bin/codeeye");
        assert_eq!(cmd[1].as_str().unwrap(), "--mcp");
    }

    #[test]
    fn test_uses_mcp_key_not_mcp_servers() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("opencode.json");

        std::fs::write(&config_path, "{}").unwrap();

        let mut config = read_json_config(&config_path).unwrap();
        let servers = config
            .as_object_mut()
            .unwrap()
            .entry("mcp")
            .or_insert_with(|| serde_json::json!({}));
        servers
            .as_object_mut()
            .unwrap()
            .insert(
                MCP_SERVER_KEY.to_string(),
                OpenCodeAdapter::mcp_entry("/usr/bin/codeeye"),
            );
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        // Must use "mcp", NOT "mcpServers"
        assert!(result.get("mcp").is_some());
        assert!(result.get("mcpServers").is_none());
    }

    #[test]
    fn test_connect_preserves_existing_config() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("opencode.json");

        let config = serde_json::json!({
            "$schema": "https://opencode.ai/config.json",
            "mcp": {
                "existing": { "type": "local", "command": ["foo"] }
            }
        });
        write_json_config(&config_path, &config).unwrap();

        let mut config = read_json_config(&config_path).unwrap();
        let servers = config
            .as_object_mut()
            .unwrap()
            .entry("mcp")
            .or_insert_with(|| serde_json::json!({}));
        servers
            .as_object_mut()
            .unwrap()
            .insert(
                MCP_SERVER_KEY.to_string(),
                OpenCodeAdapter::mcp_entry("/usr/bin/codeeye"),
            );
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        assert_eq!(result["$schema"], "https://opencode.ai/config.json");
        assert!(result["mcp"]["existing"]["command"].is_array());
        assert!(result["mcp"]["codeeye"]["command"].is_array());
    }
}
