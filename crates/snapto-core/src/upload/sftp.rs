use crate::config::UploadConfig;
use crate::error::{ConfigError, Result, SnaptoError};
use crate::upload::{UploadResult, Uploader};
use async_trait::async_trait;
use ssh2::Session;
use std::io::prelude::*;
use std::net::TcpStream;
use std::path::Path;
use std::sync::Arc;
use std::time::Instant;
use tokio::sync::Mutex;

/// Callback type for password prompts
pub type PasswordCallback = Arc<Mutex<Option<Box<dyn FnMut(&str) -> Option<String> + Send>>>>;

/// Uploader SFTP usando SSH2
pub struct SftpUploader {
    name: String,
    config: UploadConfig,
    password: Option<String>,
    password_callback: Option<PasswordCallback>,
}

impl SftpUploader {
    /// Crea un nuevo uploader SFTP
    pub fn new(name: String, config: UploadConfig) -> Self {
        Self {
            name,
            config,
            password: None,
            password_callback: None,
        }
    }

    /// Sets the password for authentication
    pub fn with_password(mut self, password: String) -> Self {
        self.password = Some(password);
        self
    }

    /// Sets a password callback for interactive password prompts
    pub fn with_password_callback(mut self, callback: PasswordCallback) -> Self {
        self.password_callback = Some(callback);
        self
    }

    /// Sets the password directly (mutable version)
    pub fn set_password(&mut self, password: String) {
        self.password = Some(password);
    }

    /// Establece una conexión SSH
    fn connect(&self) -> Result<Session> {
        let host = self
            .config
            .host
            .as_ref()
            .ok_or_else(|| ConfigError::Invalid("Host no configurado".to_string()))?;

        let port = self.config.port.unwrap_or(22);
        let addr = format!("{}:{}", host, port);

        // Conectar con timeout
        let tcp = TcpStream::connect(&addr).map_err(|e| {
            SnaptoError::SshConnection(format!("No se pudo conectar a {}: {}", addr, e))
        })?;

        // Crear sesión SSH
        let mut sess = Session::new().map_err(|e| {
            SnaptoError::SshConnection(format!("No se pudo crear sesión SSH: {}", e))
        })?;

        sess.set_tcp_stream(tcp);
        sess.handshake()
            .map_err(|e| SnaptoError::SshConnection(format!("Handshake falló: {}", e)))?;

        // Autenticar
        self.authenticate(&mut sess)?;

        Ok(sess)
    }

    /// Autentica la sesión SSH
    fn authenticate(&self, sess: &mut Session) -> Result<()> {
        let username = self
            .config
            .username
            .as_ref()
            .ok_or_else(|| ConfigError::Invalid("Usuario no configurado".to_string()))?;

        // Intentar autenticación por clave primero
        if self.config.use_key_auth.unwrap_or(false) {
            let key_path = self
                .config
                .key_path
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("Ruta de clave no configurada".to_string()))?;

            let expanded_path = shellexpand::tilde(key_path);

            // Try with passphrase if we have a password (for encrypted keys)
            let passphrase = self.password.as_deref();

            match sess.userauth_pubkey_file(username, None, Path::new(&*expanded_path), passphrase) {
                Ok(_) => {},
                Err(e) => {
                    // If key auth fails and we have a password, try password auth
                    if self.password.is_some() {
                        return self.authenticate_password(sess, username);
                    }
                    return Err(SnaptoError::SshAuthentication(format!(
                        "Autenticación por clave falló: {}",
                        e
                    )));
                }
            }
        } else {
            // Password authentication
            return self.authenticate_password(sess, username);
        }

        if !sess.authenticated() {
            return Err(SnaptoError::SshAuthentication(
                "Autenticación falló".to_string(),
            ));
        }

        Ok(())
    }

    /// Authenticate using password
    fn authenticate_password(&self, sess: &mut Session, username: &str) -> Result<()> {
        let password = self.password.as_ref()
            .ok_or_else(|| SnaptoError::SshAuthentication(
                "Se requiere contraseña para autenticación".to_string()
            ))?;

        sess.userauth_password(username, password)
            .map_err(|e| SnaptoError::SshAuthentication(format!(
                "Autenticación por contraseña falló: {}",
                e
            )))?;

        if !sess.authenticated() {
            return Err(SnaptoError::SshAuthentication(
                "Autenticación por contraseña falló".to_string(),
            ));
        }

        Ok(())
    }

    /// Gets the password, either from stored value or keychain
    pub fn get_password_from_keychain(&self, keychain: &crate::KeychainManager) -> Option<String> {
        let key = format!("ssh_password_{}", self.name);
        keychain.get(&key).ok().flatten()
    }

    /// Stores the password in keychain
    pub fn store_password_in_keychain(&self, keychain: &crate::KeychainManager, password: &str) -> Result<()> {
        let key = format!("ssh_password_{}", self.name);
        keychain.set(&key, password)
    }
}

#[async_trait]
impl Uploader for SftpUploader {
    async fn upload(&self, data: &[u8], filename: &str) -> Result<UploadResult> {
        let start = Instant::now();

        // Ejecutar en un thread bloqueante porque ssh2 no es async
        let name = self.name.clone();
        let config = self.config.clone();
        let password = self.password.clone();
        let data = data.to_vec();
        let filename = filename.to_string();

        let result = tokio::task::spawn_blocking(move || {
            let mut uploader = SftpUploader::new(name, config);
            if let Some(pwd) = password {
                uploader.set_password(pwd);
            }

            // Conectar
            let sess = uploader.connect()?;

            // Abrir canal SFTP
            let sftp = sess
                .sftp()
                .map_err(|e| SnaptoError::Sftp(format!("No se pudo abrir canal SFTP: {}", e)))?;

            // Construir ruta remota
            let remote_path = uploader
                .config
                .remote_path
                .as_ref()
                .ok_or_else(|| ConfigError::Invalid("Ruta remota no configurada".to_string()))?;

            let remote_file = format!("{}/{}", remote_path.trim_end_matches('/'), filename);

            // Crear directorios si no existen
            let parent_dir = Path::new(&remote_file)
                .parent()
                .ok_or_else(|| SnaptoError::InvalidPath("Ruta remota inválida".to_string()))?;

            // Intentar crear el directorio (ignorar error si ya existe)
            let _ = sftp.mkdir(parent_dir, 0o755);

            // Subir archivo
            let mut remote = sftp
                .create(Path::new(&remote_file))
                .map_err(|e| SnaptoError::Sftp(format!("No se pudo crear archivo remoto: {}", e)))?;

            remote
                .write_all(&data)
                .map_err(|e| SnaptoError::Sftp(format!("Error al escribir: {}", e)))?;

            remote
                .flush()
                .map_err(|e| SnaptoError::Sftp(format!("Error al hacer flush: {}", e)))?;

            // Construir URL si está configurada
            let url = uploader.config.base_url.as_ref().map(|base| {
                format!("{}/{}", base.trim_end_matches('/'), filename)
            });

            Ok::<_, SnaptoError>((remote_file, url, data.len()))
        })
        .await
        .map_err(|e| SnaptoError::Upload(format!("Error en task: {}", e)))??;

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
            return Err(ConfigError::Invalid("Host requerido".to_string()).into());
        }

        if self.config.username.is_none() {
            return Err(ConfigError::Invalid("Usuario requerido".to_string()).into());
        }

        if self.config.remote_path.is_none() {
            return Err(ConfigError::Invalid("Ruta remota requerida".to_string()).into());
        }

        if self.config.use_key_auth.unwrap_or(false) && self.config.key_path.is_none() {
            return Err(ConfigError::Invalid(
                "Ruta de clave requerida para autenticación por clave".to_string(),
            ).into());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sftp_uploader_validation() {
        let config = UploadConfig {
            uploader_type: "sftp".to_string(),
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

        let uploader = SftpUploader::new("test".to_string(), config);
        assert!(uploader.validate().is_err());
    }

    #[test]
    fn test_sftp_uploader_with_valid_config() {
        let config = UploadConfig {
            uploader_type: "sftp".to_string(),
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

        let uploader = SftpUploader::new("test".to_string(), config);
        assert!(uploader.validate().is_ok());
        assert_eq!(uploader.name(), "test");
        assert!(uploader.is_enabled());
    }
}
