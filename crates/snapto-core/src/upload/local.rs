use crate::config::UploadConfig;
use crate::error::{Result, SnaptoError};
use crate::upload::{UploadResult, Uploader};
use async_trait::async_trait;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

/// Uploader local que guarda archivos en el sistema de archivos
pub struct LocalUploader {
    name: String,
    config: UploadConfig,
}

impl LocalUploader {
    /// Crea un nuevo uploader local
    pub fn new(name: String, config: UploadConfig) -> Self {
        Self { name, config }
    }

    /// Expande la ruta local y crea directorios si es necesario
    fn prepare_path(&self, filename: &str) -> Result<PathBuf> {
        let local_path = self
            .config
            .local_path
            .as_ref()
            .ok_or_else(|| SnaptoError::Config(crate::error::ConfigError::Invalid("Ruta local no configurada".to_string())))?;

        // Expandir ~ y variables de entorno
        let expanded = shellexpand::full(local_path)
            .map_err(|e| SnaptoError::Config(crate::error::ConfigError::Invalid(format!("Error expandiendo ruta: {}", e))))?;

        let base_path = PathBuf::from(expanded.as_ref());

        // Crear directorio si no existe
        if !base_path.exists() {
            fs::create_dir_all(&base_path).map_err(|e| {
                SnaptoError::Config(crate::error::ConfigError::CreateDirectoryFailed(format!(
                    "No se pudo crear directorio {}: {}",
                    base_path.display(),
                    e
                )))
            })?;
        }

        // Validar que sea un directorio
        if !base_path.is_dir() {
            return Err(SnaptoError::InvalidPath(format!(
                "{} no es un directorio",
                base_path.display()
            )));
        }

        Ok(base_path.join(filename))
    }
}

#[async_trait]
impl Uploader for LocalUploader {
    async fn upload(&self, data: &[u8], filename: &str) -> Result<UploadResult> {
        let start = Instant::now();

        // Preparar ruta
        let full_path = self.prepare_path(filename)?;

        // Escribir archivo
        fs::write(&full_path, data).map_err(|e| {
            SnaptoError::Upload(format!(
                "Error al escribir {}: {}",
                full_path.display(),
                e
            ))
        })?;

        let duration_ms = start.elapsed().as_millis() as u64;

        // Construir URL si está configurada
        let url = self.config.base_url.as_ref().map(|base| {
            format!("{}/{}", base.trim_end_matches('/'), filename)
        });

        Ok(UploadResult {
            remote_path: full_path.display().to_string(),
            url,
            size: data.len(),
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
        if self.config.local_path.is_none() {
            return Err(SnaptoError::Config(crate::error::ConfigError::Invalid("Ruta local requerida".to_string())));
        }

        // Intentar expandir la ruta para validar
        if let Some(path) = &self.config.local_path {
            shellexpand::full(path)
                .map_err(|e| SnaptoError::Config(crate::error::ConfigError::Invalid(format!("Ruta local inválida: {}", e))))?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn test_local_uploader_validation() {
        let config = UploadConfig {
            uploader_type: "local".to_string(),
            enabled: true,
            host: None,
            port: None,
            username: None,
            remote_path: None,
            base_url: None,
            local_path: None,
            use_key_auth: None,
            key_path: None,
            timeout: None,
        };

        let uploader = LocalUploader::new("test".to_string(), config);
        assert!(uploader.validate().is_err());
    }

    #[test]
    fn test_local_uploader_with_valid_config() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let config = UploadConfig {
            uploader_type: "local".to_string(),
            enabled: true,
            host: None,
            port: None,
            username: None,
            remote_path: None,
            base_url: Some("file://".to_string()),
            local_path: Some(path.clone()),
            use_key_auth: None,
            key_path: None,
            timeout: None,
        };

        let uploader = LocalUploader::new("test".to_string(), config);
        assert!(uploader.validate().is_ok());
        assert_eq!(uploader.name(), "test");
        assert!(uploader.is_enabled());
    }

    #[tokio::test]
    async fn test_local_uploader_upload() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().to_str().unwrap().to_string();

        let config = UploadConfig {
            uploader_type: "local".to_string(),
            enabled: true,
            host: None,
            port: None,
            username: None,
            remote_path: None,
            base_url: None,
            local_path: Some(path.clone()),
            use_key_auth: None,
            key_path: None,
            timeout: None,
        };

        let uploader = LocalUploader::new("test".to_string(), config);
        let data = b"test data";
        let result = uploader.upload(data, "test.txt").await.unwrap();

        assert_eq!(result.size, 9);
        assert!(result.remote_path.ends_with("test.txt"));

        // Verificar que el archivo existe
        let file_path = temp_dir.path().join("test.txt");
        assert!(file_path.exists());

        let contents = fs::read_to_string(&file_path).unwrap();
        assert_eq!(contents, "test data");
    }

    #[tokio::test]
    async fn test_local_uploader_creates_directory() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("subdir").to_str().unwrap().to_string();

        let config = UploadConfig {
            uploader_type: "local".to_string(),
            enabled: true,
            host: None,
            port: None,
            username: None,
            remote_path: None,
            base_url: None,
            local_path: Some(path.clone()),
            use_key_auth: None,
            key_path: None,
            timeout: None,
        };

        let uploader = LocalUploader::new("test".to_string(), config);
        let data = b"test data";
        let result = uploader.upload(data, "test.txt").await.unwrap();

        assert!(result.remote_path.contains("subdir"));

        // Verificar que el directorio y archivo existen
        let dir_path = temp_dir.path().join("subdir");
        assert!(dir_path.exists());
        assert!(dir_path.is_dir());

        let file_path = dir_path.join("test.txt");
        assert!(file_path.exists());
    }
}
