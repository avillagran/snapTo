use anyhow::{anyhow, Context, Result};
use snapto_core::{
    ClipboardManager,
    ClipboardCopyMode,
    Config,
    HistoryManager,
    SftpUploader,
    LocalUploader,
    SshUploader,
    Uploader,
    TemplateParser,
    UploadConfig,
    UploadResult,
};
use std::time::Instant;

use crate::{output, progress};

/// Create an uploader based on config type
fn create_uploader(name: &str, config: &UploadConfig) -> Result<Box<dyn Uploader>> {
    let uploader: Box<dyn Uploader> = match config.uploader_type.as_str() {
        "sftp" => Box::new(SftpUploader::new(name.to_string(), config.clone())),
        "ssh" => Box::new(SshUploader::new(name.to_string(), config.clone())),
        "local" => Box::new(LocalUploader::new(name.to_string(), config.clone())),
        _ => return Err(anyhow!("Unknown uploader type: {}", config.uploader_type)),
    };
    Ok(uploader)
}

/// Execute the upload command
pub async fn execute(destination: Option<String>, filename: Option<String>) -> Result<()> {
    output::step("Reading image from clipboard...");

    // Initialize clipboard manager
    let mut clipboard = ClipboardManager::new()?;

    // Get image from clipboard
    let image_data = clipboard
        .get_image()
        .context("Failed to read image from clipboard")?;

    if image_data.is_empty() {
        return Err(anyhow!("No image found in clipboard"));
    }

    output::success(&format!(
        "Found image in clipboard ({})",
        output::format_size(image_data.len() as u64)
    ));

    // Load configuration
    output::step("Loading configuration...");
    let config = Config::load().context("Failed to load configuration")?;

    // Generate filename
    let parser = TemplateParser::new(
        config.naming.date_format.clone(),
        config.naming.time_format.clone(),
    );
    let final_filename = if let Some(name) = filename {
        name
    } else {
        parser.generate(&config.naming.template, &config.naming.default_extension)?
    };

    output::info(&format!("Filename: {}", final_filename));

    // Build list of uploaders to use
    let has_specific_dest = destination.is_some();
    let primary_name = destination.unwrap_or_else(|| config.general.default_uploader.clone());
    let mut uploader_names = vec![primary_name.clone()];

    // Add additional uploaders (only if no specific destination was provided)
    if !has_specific_dest {
        for additional in &config.general.additional_uploaders {
            if additional != &primary_name && !uploader_names.contains(additional) {
                uploader_names.push(additional.clone());
            }
        }
    }

    // Show what we're uploading to
    if uploader_names.len() > 1 {
        output::info(&format!("Uploading to {} destinations:", uploader_names.len()));
        for name in &uploader_names {
            output::item(name);
        }
    } else {
        output::info(&format!("Using destination: {}", primary_name));
    }

    // Upload to each destination
    let mut primary_result: Option<UploadResult> = None;
    let start = Instant::now();

    for (i, dest_name) in uploader_names.iter().enumerate() {
        let dest = config
            .uploads
            .get(dest_name)
            .ok_or_else(|| anyhow!("Destination '{}' not found in configuration", dest_name))?;

        if !dest.enabled {
            output::warning(&format!("Destination '{}' is disabled, skipping", dest_name));
            continue;
        }

        let uploader = create_uploader(dest_name, dest)?;
        uploader.validate()?;

        // Show progress bar for primary uploader
        let pb = if i == 0 {
            let pb = progress::upload_progress(image_data.len() as u64);
            pb.set_message(format!("Uploading to {}...", dest_name));
            Some(pb)
        } else {
            output::step(&format!("Uploading to {}...", dest_name));
            None
        };

        match uploader.upload(&image_data, &final_filename).await {
            Ok(result) => {
                if let Some(pb) = pb {
                    pb.finish_and_clear();
                }

                output::success(&format!("✓ {} → {}", dest_name,
                    result.url.as_ref().unwrap_or(&result.remote_path)));

                // Keep first successful result for clipboard
                if primary_result.is_none() {
                    primary_result = Some(result);
                }
            }
            Err(e) => {
                if let Some(pb) = pb {
                    pb.finish_and_clear();
                }
                output::error(&format!("✗ {} failed: {}", dest_name, e));

                // If primary upload failed, it's an error
                if i == 0 {
                    return Err(e.into());
                }
            }
        }
    }

    let duration = start.elapsed();
    let result = primary_result.ok_or_else(|| anyhow!("No successful uploads"))?;

    // Calculate stats
    let size_bytes = image_data.len() as u64;
    let duration_ms = duration.as_millis() as u64;
    let speed = if duration_ms > 0 {
        (size_bytes * 1000) / duration_ms
    } else {
        size_bytes
    };

    output::separator();
    output::kv("Size", &output::format_size(size_bytes));
    output::kv("Duration", &output::format_duration(duration_ms));
    output::kv("Speed", &format!("{}/s", output::format_size(speed)));
    output::separator();

    // Copy to clipboard based on mode (using primary result)
    if config.general.copy_url_to_clipboard {
        let clipboard_text = match config.general.clipboard_copy_mode {
            ClipboardCopyMode::Auto => result.url.as_ref().unwrap_or(&result.remote_path),
            ClipboardCopyMode::Url => {
                if let Some(url) = &result.url {
                    url
                } else {
                    output::warning("No URL available, skipping clipboard copy");
                    &result.remote_path
                }
            }
            ClipboardCopyMode::Path => &result.remote_path,
        };

        if config.general.clipboard_copy_mode != ClipboardCopyMode::Url || result.url.is_some() {
            output::step("Copying to clipboard...");
            clipboard
                .set_text(clipboard_text)
                .context("Failed to copy to clipboard")?;
            output::success(&format!("Copied: {}", clipboard_text));
        }
    }

    // Save to history (using primary result)
    if config.history.enabled {
        if let Ok(history) = HistoryManager::new(config.history.clone()) {
            let entry = snapto_core::HistoryEntry {
                id: 0,
                filename: final_filename.clone(),
                remote_path: result.remote_path.clone(),
                url: result.url.clone(),
                size: result.size,
                destination: primary_name.clone(),
                created_at: chrono::Utc::now(),
                thumbnail_path: None,
                local_copy_path: None,
            };
            if let Err(e) = history.add(&entry, Some(&image_data)) {
                output::warning(&format!("Failed to save to history: {}", e));
            }
        }
    }

    Ok(())
}
