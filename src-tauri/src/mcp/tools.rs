use crate::storage;
use image::GenericImageView;
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
                    },
                    "resolution_note": {
                        "type": "string",
                        "description": "Optional note describing what was fixed."
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
        .filter(|s| project_filter.is_none_or(|p| s.project == p))
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

    Ok(vec![ToolContent::Text {
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

    let config = storage::load_config(base).unwrap_or_default();
    let mut content = Vec::new();

    // Prompt block: read prompt.md if available
    if config.mcp_include_prompt {
        let prompt_path = session_dir.join("prompt.md");
        if prompt_path.exists() {
            let prompt = fs::read_to_string(&prompt_path)
                .map_err(|e| format!("Failed to read prompt: {e}"))?;
            content.push(ToolContent::Text { text: prompt });
        }
    }

    // Metadata block: full session metadata JSON
    if config.mcp_include_metadata {
        let meta_str = fs::read_to_string(&meta_path)
            .map_err(|e| format!("Failed to read meta: {e}"))?;
        content.push(ToolContent::Text { text: meta_str });
    }

    // Image block: annotated screenshot (respects MCP resolution limit)
    if config.mcp_include_image {
        if let Some((image_b64, mime_type)) =
            read_image_base64(&session_dir, config.mcp_image_max_resolution)
        {
            content.push(ToolContent::Image {
                data: image_b64,
                mime_type,
            });
        }
    }

    // Fallback: if all toggles are off, return meta so the response isn't empty
    if content.is_empty() {
        let meta_str = fs::read_to_string(&meta_path)
            .map_err(|e| format!("Failed to read meta: {e}"))?;
        content.push(ToolContent::Text { text: meta_str });
    }

    Ok(content)
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

    if let Some(note) = args.get("resolution_note").and_then(|v| v.as_str()) {
        meta["resolution_note"] = json!(note);
    }

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

    Ok(vec![ToolContent::Text {
        text: json!({
            "session_id": session_id,
            "status": "resolved",
            "message": "Session marked as resolved."
        })
        .to_string(),
    }])
}

/// Read the best available image as base64, preferring compressed over original.
/// If `max_resolution > 0`, re-compresses images exceeding that dimension.
/// Returns (base64_data, mime_type).
fn read_image_base64(session_dir: &Path, max_resolution: u32) -> Option<(String, String)> {
    let candidates = [
        ("compressed.webp", "image/webp", "webp"),
        ("compressed.png", "image/png", "png"),
        ("annotated.png", "image/png", "png"),
        ("original.png", "image/png", "png"),
    ];
    for (name, mime, format) in &candidates {
        let path = session_dir.join(name);
        if path.exists() {
            if let Ok(bytes) = fs::read(&path) {
                let final_bytes = if max_resolution > 0 {
                    // Check if resize is needed before re-encoding
                    let needs_resize = image::load_from_memory(&bytes)
                        .map(|img| {
                            let (w, h) = img.dimensions();
                            w > max_resolution || h > max_resolution
                        })
                        .unwrap_or(false);

                    if needs_resize {
                        crate::compression::compress_image(
                            &bytes,
                            max_resolution,
                            85,
                            format,
                        )
                        .unwrap_or(bytes)
                    } else {
                        bytes
                    }
                } else {
                    bytes
                };

                return Some((
                    base64::Engine::encode(
                        &base64::engine::general_purpose::STANDARD,
                        &final_bytes,
                    ),
                    (*mime).to_string(),
                ));
            }
        }
    }
    None
}
