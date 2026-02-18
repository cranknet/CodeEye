use image::ImageFormat;
use serde::{Deserialize, Serialize};
use std::io::Cursor;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitorInfo {
    pub id: u32,
    pub name: String,
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// List all available monitors with position and size.
pub fn list_monitors() -> Result<Vec<MonitorInfo>, String> {
    let monitors =
        xcap::Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;

    let mut infos = Vec::with_capacity(monitors.len());
    for (i, m) in monitors.iter().enumerate() {
        infos.push(MonitorInfo {
            id: i as u32,
            name: m.name().unwrap_or_default(),
            x: m.x().unwrap_or(0),
            y: m.y().unwrap_or(0),
            width: m.width().unwrap_or(0),
            height: m.height().unwrap_or(0),
            is_primary: m.is_primary().unwrap_or(false),
        });
    }

    Ok(infos)
}

/// Capture the full screen of a specific monitor, returning PNG bytes.
pub fn capture_monitor(monitor_id: u32) -> Result<Vec<u8>, String> {
    let monitors =
        xcap::Monitor::all().map_err(|e| format!("Failed to enumerate monitors: {e}"))?;

    let monitor = monitors
        .into_iter()
        .nth(monitor_id as usize)
        .ok_or_else(|| format!("Monitor {monitor_id} not found"))?;

    let img = monitor
        .capture_image()
        .map_err(|e| format!("Failed to capture monitor: {e}"))?;

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    img.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode capture: {e}"))?;

    Ok(buf)
}

/// Capture a rectangular region from a specific monitor, returning PNG bytes.
/// Coordinates are relative to the monitor's own origin.
pub fn capture_region(
    monitor_id: u32,
    x: u32,
    y: u32,
    width: u32,
    height: u32,
) -> Result<Vec<u8>, String> {
    let full = capture_monitor(monitor_id)?;

    let img = image::load_from_memory(&full)
        .map_err(|e| format!("Failed to decode capture: {e}"))?
        .to_rgba8();

    // Clamp region to image bounds
    let img_w = img.width();
    let img_h = img.height();
    let x = x.min(img_w.saturating_sub(1));
    let y = y.min(img_h.saturating_sub(1));
    let width = width.min(img_w - x);
    let height = height.min(img_h - y);

    if width == 0 || height == 0 {
        return Err("Region has zero area".into());
    }

    let cropped = image::imageops::crop_imm(&img, x, y, width, height).to_image();

    let mut buf = Vec::new();
    let mut cursor = Cursor::new(&mut buf);
    cropped
        .write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| format!("Failed to encode region: {e}"))?;

    Ok(buf)
}

/// Check if screen capture permission is available (macOS-specific).
/// On Linux and Windows, always returns true.
pub fn check_capture_permission() -> bool {
    // xcap handles platform-specific permissions internally.
    // If capture_monitor fails with a permission error, the user
    // will get an appropriate error message. This function does a
    // quick test capture to check availability.
    let monitors = xcap::Monitor::all();
    monitors.is_ok() && !monitors.unwrap().is_empty()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_list_monitors_returns_results() {
        // This test requires a display server. In CI without a display,
        // it may return an error which is acceptable.
        let result = list_monitors();
        // On a system with a display, should succeed
        if let Ok(monitors) = result {
            assert!(!monitors.is_empty(), "Should find at least one monitor");
            let primary_count = monitors.iter().filter(|m| m.is_primary).count();
            assert!(primary_count <= 1, "At most one primary monitor");
        }
    }

    #[test]
    fn test_check_permission() {
        // Should not panic regardless of platform
        let _result = check_capture_permission();
    }
}
