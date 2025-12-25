use anyhow::{Context, Result};
use snapto_core::Config;
use std::process::Command;

use crate::output;

/// Show current configuration
pub async fn show() -> Result<()> {
    output::header("SnapTo Configuration");

    let config = Config::load().context("Failed to load configuration")?;

    // General settings
    output::section("General");
    output::kv("Default Uploader", &config.general.default_uploader);
    if !config.general.additional_uploaders.is_empty() {
        output::kv("Additional Uploaders", &config.general.additional_uploaders.join(", "));
    }
    output::kv("Copy to Clipboard", &config.general.copy_url_to_clipboard.to_string());
    output::kv("Clipboard Mode", &format!("{:?}", config.general.clipboard_copy_mode));
    output::kv("Show Notifications", &config.general.show_notifications.to_string());
    if let Some(dir) = &config.general.local_save_dir {
        output::kv("Local Save Dir", dir);
    }

    // Naming settings
    output::section("Naming");
    output::kv("Template", &config.naming.template);
    output::kv("Date Format", &config.naming.date_format);
    output::kv("Time Format", &config.naming.time_format);
    output::kv("Default Extension", &config.naming.default_extension);

    // History settings
    output::section("History");
    output::kv("Enabled", &config.history.enabled.to_string());
    output::kv("Mode", &format!("{:?}", config.history.mode));
    output::kv("Retention Days", &config.history.retention_days.to_string());
    output::kv("Max Entries", &config.history.max_entries.to_string());

    // Security settings
    output::section("Security");
    output::kv("Use System Keychain", &config.security.use_system_keychain.to_string());
    output::kv("Encrypt Credentials", &config.security.encrypt_credentials.to_string());

    // Uploaders
    output::section("Uploaders");
    if config.uploads.is_empty() {
        output::warning("No uploaders configured");
    } else {
        for (name, uploader) in &config.uploads {
            let status = if uploader.enabled { "enabled" } else { "disabled" };
            let default = if name == &config.general.default_uploader { " (default)" } else { "" };
            output::item(&format!("{} [{}] - {}{}", name, uploader.uploader_type, status, default));

            if let Some(host) = &uploader.host {
                output::kv("  Host", host);
            }
            if let Some(port) = uploader.port {
                output::kv("  Port", &port.to_string());
            }
            if let Some(user) = &uploader.username {
                output::kv("  Username", user);
            }
            if let Some(path) = &uploader.remote_path {
                output::kv("  Remote Path", path);
            }
            if let Some(url) = &uploader.base_url {
                output::kv("  Base URL", url);
            }
            if let Some(path) = &uploader.local_path {
                output::kv("  Local Path", path);
            }
        }
    }

    output::separator();
    output::info(&format!("Config file: {:?}", Config::config_path()?));

    Ok(())
}

/// Edit configuration in default editor
pub async fn edit() -> Result<()> {
    let config_path = Config::config_path()?;

    // Ensure config file exists
    if !config_path.exists() {
        output::warning("Configuration file does not exist. Initializing...");
        init().await?;
    }

    // Get editor from environment
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(target_os = "windows") {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    });

    output::info(&format!("Opening {} in {}...", config_path.display(), editor));

    // Open editor
    let status = Command::new(&editor)
        .arg(&config_path)
        .status()
        .context("Failed to open editor")?;

    if !status.success() {
        output::error("Editor exited with error");
        return Ok(());
    }

    // Validate configuration after editing
    output::step("Validating configuration...");
    match Config::load() {
        Ok(_) => {
            output::success("Configuration is valid!");
        }
        Err(e) => {
            output::error(&format!("Configuration validation failed: {}", e));
            output::warning("Please fix the errors and try again");
        }
    }

    Ok(())
}

/// Show configuration file path
pub async fn path() -> Result<()> {
    let config_path = Config::config_path()?;
    println!("{}", config_path.display());
    Ok(())
}

/// Initialize default configuration
pub async fn init() -> Result<()> {
    output::header("Initialize SnapTo Configuration");

    let config_path = Config::config_path()?;

    if config_path.exists() {
        output::warning(&format!(
            "Configuration file already exists at: {}",
            config_path.display()
        ));

        output::info("Use 'snapto config edit' to modify existing configuration");
        return Ok(());
    }

    // Create default configuration
    let default_config = Config::default();

    // Save to file
    default_config
        .save()
        .context("Failed to save default configuration")?;

    output::success(&format!(
        "Configuration initialized at: {}",
        config_path.display()
    ));
    output::separator();
    output::info("Next steps:");
    output::list_item("Edit the configuration: snapto config edit");
    output::list_item("Add your SSH/SFTP server details");
    output::list_item("Set your default destination");
    output::list_item("Test upload: snapto upload");
    output::separator();

    Ok(())
}
