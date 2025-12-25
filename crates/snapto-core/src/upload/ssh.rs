use async_trait::async_trait;
use ssh2::Session;
use std::io::Write;
use std::net::TcpStream;
use std::path::{Path, PathBuf};
use std::time::Instant;
use tracing::{debug, error, info};

use crate::config::UploadConfig;
use crate::error::{Result, SnaptoError};
use crate::upload::{UploadResult, Uploader};

/// SSH/SFTP uploader
/// This is an alternative implementation to SftpUploader with extended authentication options
pub struct SshUploader {
    name: String,
    config: UploadConfig,
    password: Option<String>,
}

impl SshUploader {
    /// Create a new SSH uploader
    pub fn new(name: String, config: UploadConfig) -> Self {
        Self { name, config, password: None }
    }

    /// Sets the password for authentication
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Sets the password directly (mutable version)
    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }

    /// Gets the password from keychain
    pub fn get_password_from_keychain(&self, keychain: &crate::KeychainManager) -> Option<String> {
        let key = format!("ssh_password_{}", self.name);
        keychain.get(&key).ok().flatten()
    }

    /// Stores the password in keychain
    pub fn store_password_in_keychain(&self, keychain: &crate::KeychainManager, password: &str) -> crate::error::Result<()> {
        let key = format!("ssh_password_{}", self.name);
        keychain.set(&key, password)
    }

    /// Establish an SSH connection
    fn connect(&self) -> Result<Session> {
        let host = self
            .config
            .host
            .as_ref()
            .ok_or_else(|| SnaptoError::Config(crate::error::ConfigError::Invalid("Host not configured".to_string())))?;

        let username = self
            .config
            .username
            .as_ref()
            .ok_or_else(|| SnaptoError::Config(crate::error::ConfigError::Invalid("Username not configured".to_string())))?;

        let port = self.config.port.unwrap_or(22);

        debug!("Connecting to {}@{}:{}", username, host, port);

        // Create TCP connection
        let tcp = TcpStream::connect(format!("{}:{}", host, port))
            .map_err(|e| {
                error!("Failed to connect to SSH host: {}", e);
                SnaptoError::SshConnection(format!("Connection failed: {}", e))
            })?;

        // Create SSH session
        let mut session = Session::new()
            .map_err(|e| {
                error!("Failed to create SSH session: {}", e);
                SnaptoError::SshConnection(format!("Session creation failed: {}", e))
            })?;

        session.set_tcp_stream(tcp);
        session.handshake()
            .map_err(|e| {
                error!("SSH handshake failed: {}", e);
                SnaptoError::SshConnection(format!("Handshake failed: {}", e))
            })?;

        debug!("SSH connection established, authenticating...");

        // Authenticate based on the configured method
        if self.config.use_key_auth.unwrap_or(false) {
            let key_path = self
                .config
                .key_path
                .as_ref()
                .ok_or_else(|| SnaptoError::Config(crate::error::ConfigError::Invalid("Key path not configured".to_string())))?;

            let expanded_path = shellexpand::tilde(key_path).to_string();
            debug!("Authenticating with SSH key: {}", expanded_path);

            // Try with passphrase if we have a password (for encrypted keys)
            let passphrase = self.password.as_deref();

            match session.userauth_pubkey_file(
                username,
                None,
                Path::new(&expanded_path),
                passphrase,
            ) {
                Ok(_) => {},
                Err(e) => {
                    // If key auth fails and we have a password, try password auth
                    if self.password.is_some() {
                        debug!("Key auth failed, trying password auth");
                        self.authenticate_password(&mut session, username)?;
                    } else {
                        error!("SSH key authentication failed: {}", e);
                        return Err(SnaptoError::SshAuthentication(format!("Key authentication failed: {}", e)));
                    }
                }
            }
        } else if let Some(ref password) = self.password {
            // Password authentication
            debug!("Authenticating with password");
            session.userauth_password(username, password)
                .map_err(|e| {
                    error!("Password authentication failed: {}", e);
                    SnaptoError::SshAuthentication(format!("Password authentication failed: {}", e))
                })?;
        } else {
            // Try SSH agent as fallback
            debug!("Authenticating with SSH agent");
            session.userauth_agent(username)
                .map_err(|e| {
                    error!("SSH agent authentication failed: {}", e);
                    SnaptoError::SshAuthentication(format!("Agent authentication failed: {}", e))
                })?;
        }

        if !session.authenticated() {
            error!("SSH authentication failed: session not authenticated");
            return Err(SnaptoError::SshAuthentication(
                "Authentication failed".to_string(),
            ));
        }

        info!("SSH authentication successful");
        Ok(session)
    }

    /// Authenticate using password
    fn authenticate_password(&self, session: &mut Session, username: &str) -> Result<()> {
        let password = self.password.as_ref()
            .ok_or_else(|| SnaptoError::SshAuthentication(
                "Se requiere contraseña para autenticación".to_string()
            ))?;

        session.userauth_password(username, password)
            .map_err(|e| {
                error!("Password authentication failed: {}", e);
                SnaptoError::SshAuthentication(format!("Password authentication failed: {}", e))
            })?;

        if !session.authenticated() {
            return Err(SnaptoError::SshAuthentication(
                "Password authentication failed".to_string(),
            ));
        }

        Ok(())
    }

    /// Generate the public URL for a file based on base_url
    fn generate_url(&self, filename: &str) -> Option<String> {
        self.config.base_url.as_ref().map(|base| {
            format!("{}/{}", base.trim_end_matches('/'), filename)
        })
    }

    /// Ensure the remote directory exists
    fn ensure_remote_dir(&self, sftp: &ssh2::Sftp, remote_path: &str) -> Result<()> {
        debug!("Ensuring remote directory exists: {}", remote_path);

        // Try to stat the directory
        match sftp.stat(Path::new(remote_path)) {
            Ok(stat) => {
                if !stat.is_dir() {
                    return Err(SnaptoError::Sftp(format!(
                        "Path exists but is not a directory: {}",
                        remote_path
                    )));
                }
                debug!("Remote directory already exists");
                Ok(())
            }
            Err(_) => {
                // Directory doesn't exist, try to create it
                debug!("Creating remote directory: {}", remote_path);
                sftp.mkdir(Path::new(remote_path), 0o755)
                    .map_err(|e| {
                        error!("Failed to create remote directory: {}", e);
                        SnaptoError::Sftp(format!("Failed to create directory: {}", e))
                    })?;
                info!("Created remote directory: {}", remote_path);
                Ok(())
            }
        }
    }
}

#[async_trait]
impl Uploader for SshUploader {
    async fn upload(&self, data: &[u8], filename: &str) -> Result<UploadResult> {
        let start = Instant::now();
        info!("Starting SSH upload: {} ({} bytes)", filename, data.len());

        // Run the blocking SSH operations in a blocking task
        let name = self.name.clone();
        let config = self.config.clone();
        let password = self.password.clone();
        let data = data.to_vec();
        let filename = filename.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut uploader = SshUploader::new(name, config.clone());
            if let Some(pwd) = password {
                uploader.set_password(pwd);
            }

            // 1. Connect via SSH
            let session = uploader.connect()?;

            // 2. Open SFTP session
            debug!("Opening SFTP session");
            let sftp = session.sftp()
                .map_err(|e| {
                    error!("Failed to open SFTP session: {}", e);
                    SnaptoError::Sftp(format!("Failed to open SFTP: {}", e))
                })?;

            // 3. Ensure remote directory exists
            let remote_path = config
                .remote_path
                .as_ref()
                .ok_or_else(|| SnaptoError::Config(crate::error::ConfigError::Invalid("Remote path not configured".to_string())))?;

            let expanded_path = shellexpand::tilde(remote_path).to_string();
            uploader.ensure_remote_dir(&sftp, &expanded_path)?;

            // 4. Create the full remote file path
            let remote_file_path = PathBuf::from(&expanded_path).join(&filename);
            let remote_file_path_str = remote_file_path.to_string_lossy().to_string();

            debug!("Creating remote file: {}", remote_file_path_str);

            // 5. Create and write to the remote file
            let mut remote_file = sftp.create(&remote_file_path)
                .map_err(|e| {
                    error!("Failed to create remote file: {}", e);
                    SnaptoError::Sftp(format!("Failed to create file: {}", e))
                })?;

            remote_file.write_all(&data)
                .map_err(|e| {
                    error!("Failed to write data to remote file: {}", e);
                    SnaptoError::Sftp(format!("Failed to write file: {}", e))
                })?;

            // Ensure the file is flushed
            remote_file.flush()
                .map_err(|e| {
                    error!("Failed to flush remote file: {}", e);
                    SnaptoError::Sftp(format!("Failed to flush file: {}", e))
                })?;

            info!("Successfully uploaded {} to {}", filename, remote_file_path_str);

            // 6. Generate URL and return result
            let url = uploader.generate_url(&filename);

            if let Some(ref url) = url {
                info!("Generated URL: {}", url);
            }

            Ok::<(String, Option<String>, usize), SnaptoError>((remote_file_path_str, url, data.len()))
        })
        .await
        .map_err(|e| {
            error!("SSH upload task failed: {}", e);
            SnaptoError::Upload(format!("Upload task failed: {}", e))
        })??;

        let duration_ms = start.elapsed().as_millis() as u64;

        Ok(UploadResult {
            remote_path: result.0,
            url: result.1,
            size: result.2,
            duration_ms,
        })
    }

    fn name(&self) -> &str {
        &self.name
    }

    fn is_enabled(&self) -> bool {
        self.config.enabled
    }

    fn validate(&self) -> Result<()> {
        if self.config.host.is_none() {
            return Err(SnaptoError::Config(crate::error::ConfigError::Invalid("Host required".to_string())));
        }

        if self.config.username.is_none() {
            return Err(SnaptoError::Config(crate::error::ConfigError::Invalid("Username required".to_string())));
        }

        if self.config.remote_path.is_none() {
            return Err(SnaptoError::Config(crate::error::ConfigError::Invalid("Remote path required".to_string())));
        }

        if self.config.use_key_auth.unwrap_or(false) && self.config.key_path.is_none() {
            return Err(SnaptoError::Config(crate::error::ConfigError::Invalid(
                "Key path required for key authentication".to_string(),
            )));
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::UploadConfig;

    #[test]
    fn test_ssh_uploader_validation() {
        let config = UploadConfig {
            uploader_type: "ssh".to_string(),
            enabled: true,
            host: None,
            port: Some(22),
            username: None,
            remote_path: None,
            base_url: None,
            local_path: None,
            use_key_auth: None,
            key_path: None,
            timeout: None,
        };

        let uploader = SshUploader::new("test".to_string(), config);
        assert!(uploader.validate().is_err());
    }

    #[test]
    fn test_ssh_uploader_with_valid_config() {
        let config = UploadConfig {
            uploader_type: "ssh".to_string(),
            enabled: true,
            host: Some("example.com".to_string()),
            port: Some(22),
            username: Some("user".to_string()),
            remote_path: Some("/uploads".to_string()),
            base_url: Some("https://example.com/uploads".to_string()),
            local_path: None,
            use_key_auth: Some(true),
            key_path: Some("~/.ssh/id_rsa".to_string()),
            timeout: Some(30),
        };

        let uploader = SshUploader::new("test".to_string(), config);
        assert!(uploader.validate().is_ok());
        assert_eq!(uploader.name(), "test");
        assert!(uploader.is_enabled());
    }

    #[test]
    fn test_generate_url() {
        let config = UploadConfig {
            uploader_type: "ssh".to_string(),
            enabled: true,
            host: Some("example.com".to_string()),
            port: Some(22),
            username: Some("user".to_string()),
            remote_path: Some("/uploads".to_string()),
            base_url: Some("https://example.com/files".to_string()),
            local_path: None,
            use_key_auth: Some(true),
            key_path: Some("~/.ssh/id_rsa".to_string()),
            timeout: Some(30),
        };

        let uploader = SshUploader::new("test".to_string(), config);
        let url = uploader.generate_url("test.png");

        assert_eq!(url, Some("https://example.com/files/test.png".to_string()));
    }

    #[test]
    fn test_generate_url_no_template() {
        let config = UploadConfig {
            uploader_type: "ssh".to_string(),
            enabled: true,
            host: Some("example.com".to_string()),
            port: Some(22),
            username: Some("user".to_string()),
            remote_path: Some("/uploads".to_string()),
            base_url: None,
            local_path: None,
            use_key_auth: Some(true),
            key_path: Some("~/.ssh/id_rsa".to_string()),
            timeout: Some(30),
        };

        let uploader = SshUploader::new("test".to_string(), config);
        let url = uploader.generate_url("test.png");

        assert_eq!(url, None);
    }
}
