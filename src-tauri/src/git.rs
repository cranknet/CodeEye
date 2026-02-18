use serde::{Deserialize, Serialize};
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitContext {
    pub project: String,
    pub branch: String,
    pub working_directory: String,
    pub suggested_file: Option<String>,
    pub recent_diff: Option<String>,
}

/// Detect git context from a directory.
/// Returns project name (from remote), branch, and working directory.
pub fn detect_git_context(dir: &Path) -> Result<GitContext, String> {
    let root_output = Command::new("git")
        .args(["rev-parse", "--show-toplevel"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("git not found: {}", e))?;

    if !root_output.status.success() {
        return Err("Not a git repository".into());
    }

    let working_directory = String::from_utf8_lossy(&root_output.stdout)
        .trim()
        .to_string();

    let project = get_project_name(dir).unwrap_or_else(|_| "unknown".into());
    let branch = get_branch(dir).unwrap_or_else(|_| "unknown".into());

    Ok(GitContext {
        project,
        branch,
        working_directory,
        suggested_file: None,
        recent_diff: None,
    })
}

fn get_project_name(dir: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["remote", "get-url", "origin"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to get remote: {}", e))?;

    if !output.status.success() {
        return Err("No remote origin".into());
    }

    let url = String::from_utf8_lossy(&output.stdout).trim().to_string();
    // Extract project name: https://github.com/user/project.git -> project
    // Also handles: git@github.com:user/project.git
    let name = url
        .rsplit('/')
        .next()
        .or_else(|| url.rsplit(':').next())
        .unwrap_or("unknown")
        .trim_end_matches(".git")
        .to_string();
    Ok(name)
}

fn get_branch(dir: &Path) -> Result<String, String> {
    let output = Command::new("git")
        .args(["rev-parse", "--abbrev-ref", "HEAD"])
        .current_dir(dir)
        .output()
        .map_err(|e| format!("Failed to get branch: {}", e))?;

    if !output.status.success() {
        return Err("Failed to get branch".into());
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}

/// Get recent git diff for frontend files only, truncated to max_lines.
pub fn get_recent_diff(dir: &Path, max_lines: usize) -> Option<String> {
    let output = Command::new("git")
        .args([
            "diff", "HEAD", "--", "*.ts", "*.tsx", "*.js", "*.jsx", "*.svelte", "*.vue", "*.css",
            "*.html",
        ])
        .current_dir(dir)
        .output()
        .ok()?;

    if !output.status.success() || output.stdout.is_empty() {
        return None;
    }

    let diff = String::from_utf8_lossy(&output.stdout);
    let truncated: String = diff.lines().take(max_lines).collect::<Vec<_>>().join("\n");
    Some(truncated)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_git_repo(dir: &Path) {
        Command::new("git")
            .args(["init"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.email", "test@test.com"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["config", "user.name", "Test"])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args([
                "remote",
                "add",
                "origin",
                "https://github.com/user/myproject.git",
            ])
            .current_dir(dir)
            .output()
            .unwrap();
        Command::new("git")
            .args(["commit", "--allow-empty", "-m", "init"])
            .current_dir(dir)
            .output()
            .unwrap();
    }

    #[test]
    fn test_detect_project_name() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo(tmp.path());
        let ctx = detect_git_context(tmp.path()).unwrap();
        assert_eq!(ctx.project, "myproject");
    }

    #[test]
    fn test_detect_branch() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo(tmp.path());
        let ctx = detect_git_context(tmp.path()).unwrap();
        assert!(!ctx.branch.is_empty());
    }

    #[test]
    fn test_no_git_repo_returns_error() {
        let tmp = TempDir::new().unwrap();
        let result = detect_git_context(tmp.path());
        assert!(result.is_err());
    }

    #[test]
    fn test_working_directory_is_absolute() {
        let tmp = TempDir::new().unwrap();
        setup_git_repo(tmp.path());
        let ctx = detect_git_context(tmp.path()).unwrap();
        assert!(
            Path::new(&ctx.working_directory).is_absolute(),
            "working_directory should be absolute: {}",
            ctx.working_directory
        );
    }
}
