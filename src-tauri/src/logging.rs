use tauri_plugin_log::{RotationStrategy, Target, TargetKind, TimezoneStrategy};

/// Build the configured tauri-plugin-log instance.
///
/// Configuration:
/// - Stdout + file targets
/// - 5 rotated files, 5 MB each
/// - Local timezone timestamps
/// - Info level by default (debug in dev builds)
pub fn build_plugin() -> tauri_plugin_log::Builder {
    let level = if cfg!(debug_assertions) {
        log::LevelFilter::Debug
    } else {
        log::LevelFilter::Info
    };

    tauri_plugin_log::Builder::new()
        .targets([
            Target::new(TargetKind::Stdout),
            Target::new(TargetKind::LogDir {
                file_name: Some("codeeye".into()),
            }),
        ])
        .rotation_strategy(RotationStrategy::KeepSome(5))
        .max_file_size(5_000_000)
        .timezone_strategy(TimezoneStrategy::UseLocal)
        .level(level)
}
