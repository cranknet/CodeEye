use crate::mcp::adapters;
use crate::storage;
use std::fs;
use std::path::Path;

/// Performs a clean uninstall of CodeEye.
/// 1. Disconnects all AI tool integrations
/// 2. Optionally backs up data
/// 3. Deletes ~/.codeeye/
pub fn run_uninstall(backup: bool) -> Result<String, String> {
    let base = storage::default_base_path();

    // Step 1: Disconnect all integrations
    let adapters = adapters::all_adapters();
    for adapter in &adapters {
        if adapter.is_connected() {
            if let Err(e) = adapter.disconnect() {
                log::warn!("Failed to disconnect {}: {}", adapter.name(), e);
            } else {
                log::info!("Disconnected {}", adapter.name());
            }
        }
    }

    // Step 2: Optional backup
    if backup {
        let backup_path = create_backup(&base)?;
        log::info!("Backup created at {:?}", backup_path);
    }

    // Step 3: Delete ~/.codeeye/
    if base.exists() {
        fs::remove_dir_all(&base)
            .map_err(|e| format!("Failed to delete {}: {e}", base.display()))?;
        log::info!("Deleted {}", base.display());
    }

    Ok("Uninstall complete".into())
}

/// Create a zip backup of ~/.codeeye/ → ~/.codeeye-backup-<timestamp>/
fn create_backup(base: &Path) -> Result<std::path::PathBuf, String> {
    let timestamp = chrono::Utc::now().format("%Y%m%d_%H%M%S");
    let backup_dir = base.with_file_name(format!(".codeeye-backup-{timestamp}"));

    copy_dir_recursive(base, &backup_dir)?;
    Ok(backup_dir)
}

fn copy_dir_recursive(src: &Path, dst: &Path) -> Result<(), String> {
    fs::create_dir_all(dst).map_err(|e| format!("Failed to create {}: {e}", dst.display()))?;

    let entries =
        fs::read_dir(src).map_err(|e| format!("Failed to read {}: {e}", src.display()))?;

    for entry in entries {
        let entry = entry.map_err(|e| format!("Failed to read entry: {e}"))?;
        let path = entry.path();
        let dest = dst.join(entry.file_name());

        if path.is_dir() {
            copy_dir_recursive(&path, &dest)?;
        } else {
            fs::copy(&path, &dest)
                .map_err(|e| format!("Failed to copy {}: {e}", path.display()))?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_copy_dir_recursive() {
        let tmp = TempDir::new().unwrap();
        let src = tmp.path().join("src");
        let dst = tmp.path().join("dst");

        fs::create_dir_all(src.join("sub")).unwrap();
        fs::write(src.join("file.txt"), "hello").unwrap();
        fs::write(src.join("sub").join("nested.txt"), "world").unwrap();

        copy_dir_recursive(&src, &dst).unwrap();

        assert!(dst.join("file.txt").exists());
        assert!(dst.join("sub").join("nested.txt").exists());
        assert_eq!(fs::read_to_string(dst.join("file.txt")).unwrap(), "hello");
    }
}
