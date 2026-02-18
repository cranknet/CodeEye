use super::{
    read_toml_config, run_verify_command, which_exists, write_toml_config, McpConfigAdapter,
    MCP_SERVER_KEY,
};
use std::path::PathBuf;

pub struct CodexAdapter;

impl CodexAdapter {
    /// Codex CLI config: ~/.codex/config.toml
    fn config_path_inner() -> PathBuf {
        dirs::home_dir()
            .unwrap_or_default()
            .join(".codex")
            .join("config.toml")
    }

    /// Build the TOML table for an MCP server entry:
    /// ```toml
    /// [mcp_servers.codeeye]
    /// command = "/path/to/codeeye"
    /// args = ["--mcp"]
    /// ```
    fn mcp_entry(binary_path: &str) -> toml::Value {
        let mut entry = toml::map::Map::new();
        entry.insert("command".into(), toml::Value::String(binary_path.into()));
        entry.insert(
            "args".into(),
            toml::Value::Array(vec![toml::Value::String("--mcp".into())]),
        );
        toml::Value::Table(entry)
    }
}

impl McpConfigAdapter for CodexAdapter {
    fn name(&self) -> &str {
        "OpenAI Codex"
    }

    fn is_installed(&self) -> bool {
        which_exists("codex")
    }

    fn config_path(&self) -> Option<PathBuf> {
        Some(Self::config_path_inner())
    }

    fn is_connected(&self) -> bool {
        let path = Self::config_path_inner();
        if let Ok(config) = read_toml_config(&path) {
            config
                .get("mcp_servers")
                .and_then(|s| s.get(MCP_SERVER_KEY))
                .is_some()
        } else {
            false
        }
    }

    fn connect(&self, binary_path: &str) -> Result<(), String> {
        let path = Self::config_path_inner();
        let mut config = read_toml_config(&path)?;

        let table = config
            .as_table_mut()
            .ok_or("Config is not a TOML table")?;

        let servers = table
            .entry("mcp_servers")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));

        servers
            .as_table_mut()
            .ok_or("mcp_servers is not a TOML table")?
            .insert(MCP_SERVER_KEY.into(), Self::mcp_entry(binary_path));

        write_toml_config(&path, &config)
    }

    fn disconnect(&self) -> Result<(), String> {
        let path = Self::config_path_inner();
        let mut config = read_toml_config(&path)?;

        if let Some(servers) = config
            .get_mut("mcp_servers")
            .and_then(|s| s.as_table_mut())
        {
            servers.remove(MCP_SERVER_KEY);
        }

        write_toml_config(&path, &config)
    }

    fn verify(&self) -> Result<String, String> {
        run_verify_command("codex", &["mcp", "list"])
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcp::adapters::{read_toml_config, write_toml_config};

    #[test]
    fn test_connect_creates_toml_mcp_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        std::fs::write(&config_path, "").unwrap();

        let mut config = read_toml_config(&config_path).unwrap();
        let table = config.as_table_mut().unwrap();
        let servers = table
            .entry("mcp_servers")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        servers
            .as_table_mut()
            .unwrap()
            .insert(MCP_SERVER_KEY.into(), CodexAdapter::mcp_entry("/usr/bin/codeeye"));
        write_toml_config(&config_path, &config).unwrap();

        let result = read_toml_config(&config_path).unwrap();
        let entry = &result["mcp_servers"]["codeeye"];
        assert_eq!(entry["command"].as_str().unwrap(), "/usr/bin/codeeye");
        assert_eq!(entry["args"].as_array().unwrap()[0].as_str().unwrap(), "--mcp");
    }

    #[test]
    fn test_connect_preserves_existing_toml() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        std::fs::write(
            &config_path,
            "model = \"o3\"\n\n[mcp_servers.existing]\ncommand = \"foo\"\n",
        )
        .unwrap();

        let mut config = read_toml_config(&config_path).unwrap();
        let table = config.as_table_mut().unwrap();
        let servers = table
            .entry("mcp_servers")
            .or_insert_with(|| toml::Value::Table(toml::map::Map::new()));
        servers
            .as_table_mut()
            .unwrap()
            .insert(MCP_SERVER_KEY.into(), CodexAdapter::mcp_entry("/usr/bin/codeeye"));
        write_toml_config(&config_path, &config).unwrap();

        let result = read_toml_config(&config_path).unwrap();
        assert_eq!(result["model"].as_str().unwrap(), "o3");
        assert_eq!(result["mcp_servers"]["existing"]["command"].as_str().unwrap(), "foo");
        assert_eq!(result["mcp_servers"]["codeeye"]["command"].as_str().unwrap(), "/usr/bin/codeeye");
    }

    #[test]
    fn test_disconnect_removes_toml_entry() {
        let tmp = tempfile::TempDir::new().unwrap();
        let config_path = tmp.path().join("config.toml");

        std::fs::write(
            &config_path,
            "[mcp_servers.codeeye]\ncommand = \"/usr/bin/codeeye\"\nargs = [\"--mcp\"]\n\n[mcp_servers.other]\ncommand = \"bar\"\n",
        )
        .unwrap();

        let mut config = read_toml_config(&config_path).unwrap();
        if let Some(servers) = config.get_mut("mcp_servers").and_then(|s| s.as_table_mut()) {
            servers.remove(MCP_SERVER_KEY);
        }
        write_toml_config(&config_path, &config).unwrap();

        let result = read_toml_config(&config_path).unwrap();
        assert!(result.get("mcp_servers").unwrap().get("codeeye").is_none());
        assert_eq!(result["mcp_servers"]["other"]["command"].as_str().unwrap(), "bar");
    }
}
