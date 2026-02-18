use crate::storage;
use std::path::Path;

/// Run cleanup: remove sessions exceeding age limit, then count limit.
/// Returns number of sessions cleaned.
pub fn run_cleanup(base_path: &Path, max_count: usize, max_age_days: u32) -> Result<usize, String> {
    let mut sessions = storage::list_sessions(base_path)?;
    let now = chrono::Utc::now().timestamp_millis();
    let max_age_ms = max_age_days as i64 * 24 * 60 * 60 * 1000;

    // Sort oldest first
    sessions.sort_by_key(|s| s.created_at);

    let mut deleted = 0;

    // Pass 1: age-based — delete anything older than max_age_days
    let (expired, remaining): (Vec<_>, Vec<_>) = sessions
        .into_iter()
        .partition(|s| now - s.created_at > max_age_ms);

    for session in &expired {
        storage::delete_session(base_path, &session.id)?;
        log::info!("Auto-cleaner: deleted expired session {}", session.id);
        deleted += 1;
    }

    // Pass 2: count-based — if still over max_count, delete oldest remaining
    if remaining.len() > max_count {
        let excess = remaining.len() - max_count;
        for session in remaining.iter().take(excess) {
            storage::delete_session(base_path, &session.id)?;
            log::info!("Auto-cleaner: deleted excess session {}", session.id);
            deleted += 1;
        }
    }

    Ok(deleted)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_clean_by_count() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        storage::init_storage(&base).unwrap();

        for i in 0..5 {
            storage::create_session(&base, &format!("Page {}", i), "project").unwrap();
        }

        let cleaned = run_cleanup(&base, 3, 365).unwrap();
        assert_eq!(cleaned, 2);

        let remaining = storage::list_sessions(&base).unwrap();
        assert_eq!(remaining.len(), 3);
    }

    #[test]
    fn test_clean_nothing_when_under_limit() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        storage::init_storage(&base).unwrap();

        storage::create_session(&base, "Page 1", "project").unwrap();

        let cleaned = run_cleanup(&base, 200, 30).unwrap();
        assert_eq!(cleaned, 0);
    }

    #[test]
    fn test_clean_preserves_newest_sessions() {
        let tmp = TempDir::new().unwrap();
        let base = tmp.path().join(".codeeye");
        storage::init_storage(&base).unwrap();

        let mut ids = Vec::new();
        for i in 0..5 {
            let id = storage::create_session(&base, &format!("Page {}", i), "project").unwrap();
            ids.push(id);
        }

        run_cleanup(&base, 3, 365).unwrap();

        let remaining = storage::list_sessions(&base).unwrap();
        let remaining_ids: Vec<&str> = remaining.iter().map(|s| s.id.as_str()).collect();

        // The 3 newest (last created) should remain
        for id in &ids[2..] {
            assert!(remaining_ids.contains(&id.as_str()));
        }
    }
}
