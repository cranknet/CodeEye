use super::{
    read_json_config, which_exists, write_json_config, McpConfigAdapter, MCP_SERVER_KEY,
};
use std::path::PathBuf;

pub struct CursorAdapter;

impl CursorAdapter {
    /// Cursor global MCP config: ~/.cursor/mcp.json
    fn config_path_inner() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".cursor")
            .join("mcp.json")
    }

    fn mcp_entry(binary_path: &str) -> serde_json::Value {
        serde_json::json!({
            "command": binary_path,
            "args": ["--mcp"]
        })
    }
}

impl McpConfigAdapter for CursorAdapter {
    fn name(&self) -> &str {
        "Cursor"
    }

    fn is_installed(&self) -> bool {
        which_exists("cursor")
            || dirs::home_dir()
                .map(|h| h.join(".cursor").exists())
                .unwrap_or(false)
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
            .insert(MCP_SERVER_KEY.to_string(), Self::mcp_entry(binary_path));

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

    fn verify(&self) -> Result<String, String> {
        // Cursor verification is done via Cursor Settings > MCP (GUI only).
        Ok("Config written. Open Cursor > Settings > MCP to verify.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_creates_mcp_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        std::fs::write(&config_path, "{}").unwrap();

        let mut config = read_json_config(&config_path).unwrap();
        let servers = config
            .as_object_mut()
            .unwrap()
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));
        servers
            .as_object_mut()
            .unwrap()
            .insert(
                MCP_SERVER_KEY.to_string(),
                CursorAdapter::mcp_entry("/usr/bin/codeeye"),
            );
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        assert!(result["mcpServers"]["codeeye"]["command"]
            .as_str()
            .unwrap()
            .contains("codeeye"));
        assert_eq!(result["mcpServers"]["codeeye"]["args"][0], "--mcp");
    }

    #[test]
    fn test_disconnect_removes_mcp_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let config = serde_json::json!({
            "mcpServers": {
                "codeeye": { "command": "/usr/bin/codeeye", "args": ["--mcp"] },
                "other": { "command": "/usr/bin/other" }
            }
        });
        write_json_config(&config_path, &config).unwrap();

        let mut config = read_json_config(&config_path).unwrap();
        if let Some(servers) = config.get_mut("mcpServers").and_then(|s| s.as_object_mut()) {
            servers.remove(MCP_SERVER_KEY);
        }
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        assert!(result["mcpServers"].get("codeeye").is_none());
        assert!(result["mcpServers"]["other"]["command"].is_string());
    }

    #[test]
    fn test_connect_preserves_existing_servers() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("mcp.json");

        let config = serde_json::json!({
            "mcpServers": {
                "github": { "command": "gh-mcp", "args": ["serve"] }
            }
        });
        write_json_config(&config_path, &config).unwrap();

        let mut config = read_json_config(&config_path).unwrap();
        let servers = config
            .as_object_mut()
            .unwrap()
            .entry("mcpServers")
            .or_insert_with(|| serde_json::json!({}));
        servers
            .as_object_mut()
            .unwrap()
            .insert(
                MCP_SERVER_KEY.to_string(),
                CursorAdapter::mcp_entry("/usr/bin/codeeye"),
            );
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        assert!(result["mcpServers"]["github"]["command"].is_string());
        assert!(result["mcpServers"]["codeeye"]["command"].is_string());
    }
}
