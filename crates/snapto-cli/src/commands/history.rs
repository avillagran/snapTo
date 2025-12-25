use anyhow::{Context, Result};
use chrono::Local;
use colored::*;
use snapto_core::{Config, HistoryManager};

use crate::output;

/// Execute the history command
pub async fn execute(limit: usize, full: bool) -> Result<()> {
    output::header("Upload History");

    // Load config to get history settings
    let config = Config::load().context("Failed to load configuration")?;
    let history = HistoryManager::new(config.history).context("Failed to open history database")?;
    let entries = history
        .get_recent(limit)
        .context("Failed to retrieve history")?;

    if entries.is_empty() {
        output::warning("No upload history found");
        return Ok(());
    }

    output::info(&format!("Showing last {} upload(s)", entries.len()));
    output::separator();

    for (i, entry) in entries.iter().enumerate() {
        let index = entries.len() - i;

        // Format timestamp
        let timestamp = entry.created_at.with_timezone(&Local);
        let time_str = timestamp.format("%Y-%m-%d %H:%M:%S").to_string();

        if full {
            // Full detailed view
            println!();
            output::info(&format!("Upload #{}", index.to_string().bold()));
            if let Some(url) = &entry.url {
                output::kv("  URL", url);
            }
            output::kv("  Path", &entry.remote_path);
            output::kv("  Filename", &entry.filename);
            output::kv("  Size", &output::format_size(entry.size as u64));
            output::kv("  Destination", &entry.destination);
            output::kv("  Uploaded", &time_str);
        } else {
            // Compact view
            let url_or_path = entry.url.as_ref().unwrap_or(&entry.remote_path);
            println!(
                "{} {} {} {} {}",
                format!("#{}", index).dimmed(),
                time_str.cyan(),
                entry.filename.bright_white(),
                format!("({})", output::format_size(entry.size as u64)).dimmed(),
                url_or_path.blue().underline()
            );
        }
    }

    if !full {
        println!();
        output::info("Use --full flag for detailed view");
    }

    output::separator();

    Ok(())
}
