use anyhow::{anyhow, Context, Result};
use snapto_core::{
    ClipboardManager,
    ClipboardCopyMode,
    Config,
    HistoryManager,
    HistoryEntry,
    SftpUploader,
    LocalUploader,
    SshUploader,
    Uploader,
    TemplateParser,
    UploadConfig,
    UploadResult,
};
use std::time::{Duration, Instant};
use tokio::time::sleep;

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

/// Execute the watch command
pub async fn execute(interval_ms: u64, destination: Option<String>) -> Result<()> {
    output::header("SnapTo Watch Mode");
    output::info("Watching clipboard for images...");
    output::info(&format!("Check interval: {}ms", interval_ms));
    output::info("Press Ctrl+C to stop");
    output::separator();

    // Load configuration
    let config = Config::load().context("Failed to load configuration")?;

    // Build list of uploaders to use
    let primary_name = destination.clone().unwrap_or_else(|| config.general.default_uploader.clone());
    let mut uploader_names = vec![primary_name.clone()];

    // Add additional uploaders (only if no specific destination was provided)
    if destination.is_none() {
        for additional in &config.general.additional_uploaders {
            if additional != &primary_name && !uploader_names.contains(additional) {
                uploader_names.push(additional.clone());
            }
        }
    }

    // Create and validate all uploaders
    let mut uploaders: Vec<(String, Box<dyn Uploader>)> = Vec::new();

    for name in &uploader_names {
        let dest = config
            .uploads
            .get(name)
            .ok_or_else(|| anyhow!("Destination '{}' not found in configuration", name))?;

        if !dest.enabled {
            output::warning(&format!("Destination '{}' is disabled, skipping", name));
            continue;
        }

        let uploader = create_uploader(name, dest)?;
        uploader.validate()?;
        uploaders.push((name.clone(), uploader));
    }

    if uploaders.is_empty() {
        return Err(anyhow!("No enabled uploaders configured"));
    }

    // Show what we're uploading to
    if uploaders.len() > 1 {
        output::info(&format!("Uploading to {} destinations:", uploaders.len()));
        for (name, _) in &uploaders {
            output::item(name);
        }
    } else {
        output::info(&format!("Using destination: {}", uploaders[0].0));
    }
    output::separator();

    // Initialize clipboard manager
    let mut clipboard = ClipboardManager::new()?;
    let mut last_hash: Option<u64> = None;
    let mut upload_count = 0u64;

    // Initialize history storage
    let history = if config.history.enabled {
        HistoryManager::new(config.history.clone()).ok()
    } else {
        None
    };

    output::success("Watch mode started");

    loop {
        // Check clipboard for image
        match clipboard.get_image() {
            Ok(image_data) if !image_data.is_empty() => {
                // Calculate hash to detect changes
                let current_hash = calculate_hash(&image_data);

                // Only upload if image has changed
                if last_hash.map_or(true, |h| h != current_hash) {
                    last_hash = Some(current_hash);

                    println!();
                    output::step(&format!(
                        "New image detected ({})",
                        output::format_size(image_data.len() as u64)
                    ));

                    // Generate filename
                    let parser = TemplateParser::new(
                        config.naming.date_format.clone(),
                        config.naming.time_format.clone(),
                    );
                    let filename = parser.generate(&config.naming.template, &config.naming.default_extension)?;

                    // Upload to all destinations
                    let mut primary_result: Option<UploadResult> = None;
                    let start = Instant::now();

                    for (i, (dest_name, uploader)) in uploaders.iter().enumerate() {
                        let pb = if i == 0 {
                            Some(progress::simple_progress(&format!("Uploading to {}...", dest_name)))
                        } else {
                            output::step(&format!("Uploading to {}...", dest_name));
                            None
                        };

                        match uploader.upload(&image_data, &filename).await {
                            Ok(result) => {
                                if let Some(pb) = pb {
                                    pb.finish_and_clear();
                                }

                                output::success(&format!("✓ {} → {}",
                                    dest_name,
                                    result.url.as_ref().unwrap_or(&result.remote_path)));

                                if primary_result.is_none() {
                                    primary_result = Some(result);
                                }
                            }
                            Err(e) => {
                                if let Some(pb) = pb {
                                    pb.finish_and_clear();
                                }
                                output::error(&format!("✗ {} failed: {}", dest_name, e));
                            }
                        }
                    }

                    // Process result
                    if let Some(result) = primary_result {
                        let duration = start.elapsed();
                        upload_count += 1;

                        output::info(&format!(
                            "Upload #{} completed in {}",
                            upload_count,
                            output::format_duration(duration.as_millis() as u64)
                        ));

                        // Copy to clipboard based on mode
                        if config.general.copy_url_to_clipboard {
                            let should_copy = match config.general.clipboard_copy_mode {
                                ClipboardCopyMode::Url if result.url.is_none() => {
                                    output::warning("No URL available, skipping clipboard copy");
                                    false
                                }
                                _ => true,
                            };

                            if should_copy {
                                let clipboard_text = match config.general.clipboard_copy_mode {
                                    ClipboardCopyMode::Auto => result.url.as_ref().unwrap_or(&result.remote_path),
                                    ClipboardCopyMode::Url => result.url.as_ref().unwrap(),
                                    ClipboardCopyMode::Path => &result.remote_path,
                                };

                                if let Err(e) = clipboard.set_text(clipboard_text) {
                                    output::warning(&format!("Failed to copy to clipboard: {}", e));
                                } else {
                                    output::info(&format!("Copied: {}", clipboard_text));
                                }
                            }
                        }

                        // Save to history
                        if let Some(h) = history.as_ref() {
                            let entry = HistoryEntry {
                                id: 0,
                                filename: filename.clone(),
                                remote_path: result.remote_path.clone(),
                                url: result.url.clone(),
                                size: result.size,
                                destination: primary_name.clone(),
                                created_at: chrono::Utc::now(),
                                thumbnail_path: None,
                                local_copy_path: None,
                            };
                            if let Err(e) = h.add(&entry, Some(&image_data)) {
                                output::warning(&format!("Failed to save to history: {}", e));
                            }
                        }
                    } else {
                        output::error("All uploads failed!");
                    }

                    output::separator();
                    output::info("Waiting for next image...");
                }
            }
            Ok(_) => {
                // No image in clipboard, continue watching
            }
            Err(_) => {
                // Error reading clipboard, continue
            }
        }

        // Wait before next check
        sleep(Duration::from_millis(interval_ms)).await;
    }
}

/// Calculate a simple hash of the image data
fn calculate_hash(data: &[u8]) -> u64 {
    use std::collections::hash_map::DefaultHasher;
    use std::hash::{Hash, Hasher};

    let mut hasher = DefaultHasher::new();
    data.hash(&mut hasher);
    hasher.finish()
}
