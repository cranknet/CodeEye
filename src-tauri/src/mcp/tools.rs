use crate::storage;
use serde_json::json;
use std::fs;
use std::path::Path;

use super::protocol::{ToolContent, ToolDefinition};

/// Returns all MCP tool definitions.
pub fn list_tools() -> Vec<ToolDefinition> {
    vec![
        ToolDefinition {
            name: "list_ui_feedback".into(),
            description: "List UI feedback sessions. Filter by status (open/resolved) and project name. Returns session summaries with IDs, page names, and annotation counts.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "status": {
                        "type": "string",
                        "enum": ["open", "resolved", "all"],
                        "description": "Filter by session status. Defaults to 'open'."
                    },
                    "project": {
                        "type": "string",
                        "description": "Filter by project name (exact match)."
                    },
                    "limit": {
                        "type": "integer",
                        "description": "Maximum number of sessions to return. Defaults to 20."
                    }
                },
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "get_ui_feedback".into(),
            description: "Get full details of a specific UI feedback session including annotations, git context, notes, and the annotated screenshot as base64 PNG.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "The session ID to retrieve."
                    }
                },
                "required": ["session_id"],
                "additionalProperties": false
            }),
        },
        ToolDefinition {
            name: "resolve_ui_feedback".into(),
            description: "Mark a UI feedback session as resolved. This updates the session status from 'open' to 'resolved'.".into(),
            input_schema: json!({
                "type": "object",
                "properties": {
                    "session_id": {
                        "type": "string",
                        "description": "The session ID to mark as resolved."
                    }
                },
                "required": ["session_id"],
                "additionalProperties": false
            }),
        },
    ]
}

/// Execute a tool by name with the given arguments.
pub fn call_tool(name: &str, args: &serde_json::Value) -> Result<Vec<ToolContent>, String> {
    let base = storage::default_base_path();
    match name {
        "list_ui_feedback" => list_ui_feedback(&base, args),
        "get_ui_feedback" => get_ui_feedback(&base, args),
        "resolve_ui_feedback" => resolve_ui_feedback(&base, args),
        _ => Err(format!("Unknown tool: {name}")),
    }
}

fn list_ui_feedback(
    base: &Path,
    args: &serde_json::Value,
) -> Result<Vec<ToolContent>, String> {
    let status_filter = args
        .get("status")
        .and_then(|v| v.as_str())
        .unwrap_or("open");
    let project_filter = args.get("project").and_then(|v| v.as_str());
    let limit = args
        .get("limit")
        .and_then(|v| v.as_u64())
        .unwrap_or(20) as usize;

    let sessions = storage::list_sessions(base)?;

    let filtered: Vec<_> = sessions
        .into_iter()
        .filter(|s| status_filter == "all" || s.status == status_filter)
        .filter(|s| project_filter.map_or(true, |p| s.project == p))
        .take(limit)
        .collect();

    let output = json!({
        "sessions": filtered.iter().map(|s| json!({
            "id": s.id,
            "page_name": s.page_name,
            "project": s.project,
            "status": s.status,
            "annotation_count": s.annotation_count,
            "created_at": s.created_at,
            "updated_at": s.updated_at,
        })).collect::<Vec<_>>(),
        "total": filtered.len(),
    });

    Ok(vec![ToolContent {
        content_type: "text".into(),
        text: serde_json::to_string_pretty(&output).unwrap(),
    }])
}

fn get_ui_feedback(
    base: &Path,
    args: &serde_json::Value,
) -> Result<Vec<ToolContent>, String> {
    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: session_id")?;

    let session_dir = base.join("sessions").join(session_id);
    let meta_path = session_dir.join("meta.json");

    if !meta_path.exists() {
        return Err(format!("Session not found: {session_id}"));
    }

    let meta_str =
        fs::read_to_string(&meta_path).map_err(|e| format!("Failed to read meta: {e}"))?;
    let meta: serde_json::Value =
        serde_json::from_str(&meta_str).map_err(|e| format!("Failed to parse meta: {e}"))?;

    // Read compressed image if available, fall back to original
    let image_base64 = read_image_base64(&session_dir);

    let output = json!({
        "session_id": session_id,
        "meta": meta,
        "image_base64": image_base64,
    });

    Ok(vec![ToolContent {
        content_type: "text".into(),
        text: serde_json::to_string_pretty(&output).unwrap(),
    }])
}

fn resolve_ui_feedback(
    base: &Path,
    args: &serde_json::Value,
) -> Result<Vec<ToolContent>, String> {
    let session_id = args
        .get("session_id")
        .and_then(|v| v.as_str())
        .ok_or("Missing required parameter: session_id")?;

    let session_dir = base.join("sessions").join(session_id);
    let meta_path = session_dir.join("meta.json");

    if !meta_path.exists() {
        return Err(format!("Session not found: {session_id}"));
    }

    // Update meta.json status
    let meta_str =
        fs::read_to_string(&meta_path).map_err(|e| format!("Failed to read meta: {e}"))?;
    let mut meta: serde_json::Value =
        serde_json::from_str(&meta_str).map_err(|e| format!("Failed to parse meta: {e}"))?;

    meta["status"] = json!("resolved");
    meta["updated_at"] = json!(chrono::Utc::now().timestamp_millis());

    fs::write(
        &meta_path,
        serde_json::to_string_pretty(&meta).unwrap(),
    )
    .map_err(|e| format!("Failed to write meta: {e}"))?;

    // Update the session index
    let mut index = storage::load_index(base)?;
    if let Some(entry) = index.sessions.iter_mut().find(|s| s.id == session_id) {
        entry.status = "resolved".into();
        entry.updated_at = chrono::Utc::now().timestamp_millis();
    }
    storage::save_index(base, &index)?;

    Ok(vec![ToolContent {
        content_type: "text".into(),
        text: json!({
            "session_id": session_id,
            "status": "resolved",
            "message": "Session marked as resolved."
        })
        .to_string(),
    }])
}

/// Read the best available image as base64, preferring compressed over original.
fn read_image_base64(session_dir: &Path) -> Option<String> {
    let candidates = ["compressed.png", "annotated.png", "original.png"];
    for name in &candidates {
        let path = session_dir.join(name);
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                return Some(base64::Engine::encode(
                    &base64::engine::general_purpose::STANDARD,
                    &bytes,
                ));
            }
        }
    }
    None
}
