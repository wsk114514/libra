//! Stats command for reporting file extension distributions.

use std::{
    collections::HashMap,
    fs,
    path::Path,
};

use clap::Parser;
use serde::Serialize;

use crate::utils::{
    error::{CliError, CliResult, StableErrorCode},
    output::{OutputConfig, emit_json_data},
    util::require_repo,
};

#[derive(Parser, Debug)]
#[command(about = "Show file extension distribution in the working directory")]
pub struct StatsArgs {
    /// Directory to scan. Defaults to the current directory.
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
struct StatsOutput {
    total_files: usize,
    extensions: HashMap<String, usize>,
}

/// Core execution logic for the `stats` command.
pub async fn execute_safe(args: StatsArgs, output: &OutputConfig) -> CliResult<()> {
    require_repo().map_err(|_| CliError::repo_not_found())?;

    let target_dir = args.path.unwrap_or_else(|| ".".to_string());
    let path = Path::new(&target_dir);

    if !path.exists() || !path.is_dir() {
        return Err(CliError::fatal(format!(
            "invalid target path: '{}'",
            target_dir
        ))
        .with_stable_code(StableErrorCode::CliInvalidTarget));
    }

    let mut extension_counts: HashMap<String, usize> = HashMap::new();
    let mut total_files = 0;

    scan_directory(path, &mut extension_counts, &mut total_files)?;

    let stats_output = StatsOutput {
        total_files,
        extensions: extension_counts,
    };

    if output.is_json() {
        emit_json_data("stats", &stats_output, output)?;
    } else if !output.quiet {
        print_stats_text(&stats_output);
    }

    Ok(())
}

fn scan_directory(
    dir: &Path,
    counts: &mut HashMap<String, usize>,
    total: &mut usize,
) -> CliResult<()> {
    let entries = fs::read_dir(dir).map_err(|e| {
        CliError::fatal(format!("failed to read directory '{}': {}", dir.display(), e))
            .with_stable_code(StableErrorCode::IoReadFailed)
    })?;

    for entry in entries {
        let entry = entry.map_err(|e| {
            CliError::fatal(format!("failed to read directory entry: {}", e))
                .with_stable_code(StableErrorCode::IoReadFailed)
        })?;

        let path = entry.path();
        let file_name = path.file_name().unwrap_or_default().to_string_lossy();

        // Skip ignored directories as requested in the assignment
        if file_name == ".libra" || file_name == "target" || file_name == ".git" {
            continue;
        }

        if path.is_dir() {
            scan_directory(&path, counts, total)?;
        } else if path.is_file() {
            *total += 1;
            let ext = get_extension(&path);
            *counts.entry(ext).or_insert(0) += 1;
        }
    }

    Ok(())
}

fn get_extension(path: &Path) -> String {
    path.extension()
        .map(|os_str| os_str.to_string_lossy().to_string())
        .unwrap_or_else(|| "no_extension".to_string())
}

fn print_stats_text(stats: &StatsOutput) {
    println!("Total files scanned: {}", stats.total_files);
    if stats.total_files > 0 {
        println!("\nExtension Distribution:");
        
        // Sort extensions by count (descending), then alphabetically for stability
        let mut sorted_exts: Vec<_> = stats.extensions.iter().collect();
        sorted_exts.sort_by(|a, b| b.1.cmp(a.1).then_with(|| a.0.cmp(b.0)));

        for (ext, count) in sorted_exts {
            let percentage = (*count as f64 / stats.total_files as f64) * 100.0;
            println!("  {:<15} {:>5} ({:>5.1}%)", ext, count, percentage);
        }
    }
}