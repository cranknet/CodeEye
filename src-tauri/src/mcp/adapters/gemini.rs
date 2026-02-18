use super::{
    read_json_config, which_exists, write_json_config, McpConfigAdapter, MCP_SERVER_KEY,
};
use std::path::PathBuf;

pub struct GeminiAdapter;

impl GeminiAdapter {
    /// Gemini CLI config: ~/.gemini/settings.json
    fn config_path_inner() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".gemini")
            .join("settings.json")
    }

    fn mcp_entry(binary_path: &str) -> serde_json::Value {
        serde_json::json!({
            "command": binary_path,
            "args": ["--mcp"]
        })
    }
}

impl McpConfigAdapter for GeminiAdapter {
    fn name(&self) -> &str {
        "Gemini CLI"
    }

    fn is_installed(&self) -> bool {
        which_exists("gemini")
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
        // Gemini CLI uses the interactive `/mcp` slash command, not a CLI flag.
        // We can't automate it, so return guidance instead.
        Ok("Config written. Run `gemini` then type `/mcp` to verify.".into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_connect_creates_mcp_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("settings.json");

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
                GeminiAdapter::mcp_entry("/usr/bin/codeeye"),
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
    fn test_connect_preserves_existing_settings() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("settings.json");

        let config = serde_json::json!({
            "theme": "dark",
            "mcpServers": {
                "existing": { "command": "foo" }
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
                GeminiAdapter::mcp_entry("/usr/bin/codeeye"),
            );
        write_json_config(&config_path, &config).unwrap();

        let result = read_json_config(&config_path).unwrap();
        assert_eq!(result["theme"], "dark");
        assert!(result["mcpServers"]["existing"]["command"].is_string());
        assert!(result["mcpServers"]["codeeye"]["command"].is_string());
    }
}
