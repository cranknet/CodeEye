use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize)]
pub struct AppConfig {
    pub version: u32,
    pub first_run_complete: bool,
    pub theme: String,
    pub global_shortcut: String,
    pub launch_at_login: bool,
    pub capture_sound: bool,
    pub session_limit: u32,
    pub session_retention_days: u32,
    pub feedback_loop_enabled: bool,
    pub git_diff_in_prompt: bool,
    pub prompt_preview: bool,
    pub compression_max_resolution: u32,
    pub compression_format: String,
    pub compression_quality: u8,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            version: 1,
            first_run_complete: false,
            theme: "dark".into(),
            global_shortcut: "CmdOrCtrl+Shift+E".into(),
            launch_at_login: false,
            capture_sound: false,
            session_limit: 200,
            session_retention_days: 30,
            feedback_loop_enabled: false,
            git_diff_in_prompt: true,
            prompt_preview: true,
            compression_max_resolution: 1920,
            compression_format: "png".into(),
            compression_quality: 85,
            log_level: "info".into(),
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SessionIndex {
    pub version: u32,
    pub sessions: Vec<SessionSummary>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionSummary {
    pub id: String,
    pub page_name: String,
    pub project: String,
    pub status: String,
    pub tags: Vec<String>,
    pub annotation_count: u32,
    pub created_at: i64,
    pub updated_at: i64,
}

/// Initialize the `~/.codeeye/` directory structure.
/// Idempotent — safe to call on every startup.
pub fn init_storage(base_path: &Path) -> Result<(), String> {
    let dirs = [
        base_path.to_path_buf(),
        base_path.join("sessions"),
        base_path.join("logs"),
    ];

    for dir in &dirs {
        fs::create_dir_all(dir).map_err(|e| format!("Failed to create {:?}: {}", dir, e))?;
    }

    // Write default config if it doesn't exist
    let config_path = base_path.join("config.json");
    if !config_path.exists() {
        let config = AppConfig::default();
        let json = serde_json::to_string_pretty(&config)
            .map_err(|e| format!("Failed to serialize config: {}", e))?;
        fs::write(&config_path, json)
            .map_err(|e| format!("Failed to write config: {}", e))?;
    }

    // Write empty session index if it doesn't exist
    let index_path = base_path.join("sessions").join("index.json");
    if !index_path.exists() {
        let index = SessionIndex {
            version: 1,
            sessions: vec![],
        };
        let json = serde_json::to_string_pretty(&index)
            .map_err(|e| format!("Failed to serialize index: {}", e))?;
        fs::write(&index_path, json)
            .map_err(|e| format!("Failed to write index: {}", e))?;
    }

    Ok(())
}

/// Load config from disk, falling back to defaults on any error.
pub fn load_config(base_path: &Path) -> Result<AppConfig, String> {
    let path = base_path.join("config.json");
    let data =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read config: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse config: {}", e))
}

/// Save config to disk.
pub fn save_config(base_path: &Path, config: &AppConfig) -> Result<(), String> {
    let path = base_path.join("config.json");
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| format!("Failed to serialize config: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write config: {}", e))
}

/// Load full session metadata from a session directory.
pub fn load_session_meta(base_path: &Path, session_id: &str) -> Result<serde_json::Value, String> {
    let meta_path = base_path
        .join("sessions")
        .join(session_id)
        .join("meta.json");
    let data =
        fs::read_to_string(&meta_path).map_err(|e| format!("Failed to read meta: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse meta: {}", e))
}

/// Save session metadata to a session directory.
pub fn save_session_meta(
    base_path: &Path,
    session_id: &str,
    meta: &serde_json::Value,
) -> Result<(), String> {
    let session_dir = base_path.join("sessions").join(session_id);
    if !session_dir.exists() {
        return Err(format!("Session not found: {session_id}"));
    }
    let meta_path = session_dir.join("meta.json");
    let json = serde_json::to_string_pretty(meta)
        .map_err(|e| format!("Failed to serialize meta: {}", e))?;
    fs::write(&meta_path, json).map_err(|e| format!("Failed to write meta: {}", e))
}

/// Returns the default storage base path: `~/.codeeye/`
pub fn default_base_path() -> PathBuf {
    dirs::home_dir()
        .expect("Could not find home directory")
        .join(".codeeye")
}

// --- Session Index helpers ---

pub fn load_index(base_path: &Path) -> Result<SessionIndex, String> {
    let path = base_path.join("sessions").join("index.json");
    let data =
        fs::read_to_string(&path).map_err(|e| format!("Failed to read index: {}", e))?;
    serde_json::from_str(&data).map_err(|e| format!("Failed to parse index: {}", e))
}

pub fn save_index(base_path: &Path, index: &SessionIndex) -> Result<(), String> {
    let path = base_path.join("sessions").join("index.json");
    let json = serde_json::to_string_pretty(index)
        .map_err(|e| format!("Failed to serialize index: {}", e))?;
    fs::write(&path, json).map_err(|e| format!("Failed to write index: {}", e))
}

/// Create a new session directory and add to index. Returns session ID.
pub fn create_session(
    base_path: &Path,
    page_name: &str,
    project: &str,
) -> Result<String, String> {
    let id = uuid::Uuid::new_v4().to_string();
    let session_dir = base_path.join("sessions").join(&id);
    fs::create_dir_all(&session_dir)
        .map_err(|e| format!("Failed to create session dir: {}", e))?;

    let now = chrono::Utc::now().timestamp_millis();

    let meta = serde_json::json!({
        "version": 1,
        "status": "open",
        "page_name": page_name,
        "general_notes": "",
        "tags": [],
        "annotations": [],
        "git": null,
        "viewport": null,
        "created_at": now,
        "updated_at": now,
    });
    fs::write(
        session_dir.join("meta.json"),
        serde_json::to_string_pretty(&meta).unwrap(),
    )
    .map_err(|e| format!("Failed to write meta: {}", e))?;

    let mut index = load_index(base_path)?;
    index.sessions.push(SessionSummary {
        id: id.clone(),
        page_name: page_name.to_string(),
        project: project.to_string(),
        status: "open".into(),
        tags: vec![],
        annotation_count: 0,
        created_at: now,
        updated_at: now,
    });
    save_index(base_path, &index)?;

    Ok(id)
}

/// List all sessions from the index.
pub fn list_sessions(base_path: &Path) -> Result<Vec<SessionSummary>, String> {
    let index = load_index(base_path)?;
    Ok(index.sessions)
}

/// Delete a session by ID — removes directory and index entry.
pub fn delete_session(base_path: &Path, session_id: &str) -> Result<(), String> {
    let session_dir = base_path.join("sessions").join(session_id);
    if session_dir.exists() {
        fs::remove_dir_all(&session_dir)
            .map_err(|e| format!("Failed to delete session dir: {}", e))?;
    }

    let mut index = load_index(base_path)?;
    index.sessions.retain(|s| s.id != session_id);
    save_index(base_path, &index)?;

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_init_creates_directory_structure() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        assert!(base.exists());
        assert!(base.join("sessions").exists());
        assert!(base.join("logs").exists());
        assert!(base.join("config.json").exists());
        assert!(base.join("sessions").join("index.json").exists());
    }

    #[test]
    fn test_init_creates_default_config() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        let config: serde_json::Value =
            serde_json::from_str(&fs::read_to_string(base.join("config.json")).unwrap()).unwrap();
        assert_eq!(config["version"], 1);
        assert_eq!(config["first_run_complete"], false);
    }

    #[test]
    fn test_init_creates_empty_session_index() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        let index: serde_json::Value = serde_json::from_str(
            &fs::read_to_string(base.join("sessions").join("index.json")).unwrap(),
        )
        .unwrap();
        assert_eq!(index["version"], 1);
        assert!(index["sessions"].as_array().unwrap().is_empty());
    }

    #[test]
    fn test_init_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();
        init_storage(&base).unwrap(); // Second call should not fail or overwrite
        assert!(base.join("config.json").exists());
    }

    #[test]
    fn test_create_session() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        let session_id = create_session(&base, "Test Page", "myproject").unwrap();
        assert!(!session_id.is_empty());
        assert!(base.join("sessions").join(&session_id).exists());
        assert!(base
            .join("sessions")
            .join(&session_id)
            .join("meta.json")
            .exists());
    }

    #[test]
    fn test_list_sessions_returns_created() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        create_session(&base, "Page A", "project1").unwrap();
        create_session(&base, "Page B", "project2").unwrap();

        let sessions = list_sessions(&base).unwrap();
        assert_eq!(sessions.len(), 2);
    }

    #[test]
    fn test_delete_session() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        let id = create_session(&base, "To Delete", "project").unwrap();
        delete_session(&base, &id).unwrap();

        let sessions = list_sessions(&base).unwrap();
        assert!(sessions.is_empty());
        assert!(!base.join("sessions").join(&id).exists());
    }

    #[test]
    fn test_load_config() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        init_storage(&base).unwrap();

        let config = load_config(&base).unwrap();
        assert_eq!(config.version, 1);
        assert_eq!(config.session_limit, 200);
        assert_eq!(config.theme, "dark");
    }
}
