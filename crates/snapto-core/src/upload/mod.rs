use async_trait::async_trait;
use serde::{Deserialize, Serialize};

use crate::error::Result;

pub mod sftp;
pub mod local;
pub mod ssh;

/// Resultado de una operación de subida
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadResult {
    /// Ruta remota donde se guardó el archivo
    pub remote_path: String,
    /// URL pública del archivo (si está disponible)
    pub url: Option<String>,
    /// Tamaño del archivo en bytes
    pub size: usize,
    /// Tiempo que tomó la subida en milisegundos
    pub duration_ms: u64,
}

/// Trait para implementar uploaders personalizados
#[async_trait]
pub trait Uploader: Send + Sync {
    /// Sube datos a un destino remoto
    ///
    /// # Argumentos
    /// * `data` - Datos binarios a subir
    /// * `filename` - Nombre del archivo destino
    ///
    /// # Returns
    /// Resultado de la subida con información sobre la ubicación
    async fn upload(&self, data: &[u8], filename: &str) -> Result<UploadResult>;

    /// Nombre identificador del uploader
    fn name(&self) -> &str;

    /// Indica si el uploader está habilitado
    fn is_enabled(&self) -> bool;

    /// Valida la configuración del uploader
    fn validate(&self) -> Result<()> {
        Ok(())
    }

    /// Obtiene información sobre el uploader
    fn info(&self) -> UploaderInfo {
        UploaderInfo {
            name: self.name().to_string(),
            enabled: self.is_enabled(),
            uploader_type: "generic".to_string(),
        }
    }
}

/// Información sobre un uploader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploaderInfo {
    pub name: String,
    pub enabled: bool,
    pub uploader_type: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUploader {
        name: String,
        enabled: bool,
    }

    #[async_trait]
    impl Uploader for MockUploader {
        async fn upload(&self, data: &[u8], filename: &str) -> Result<UploadResult> {
            Ok(UploadResult {
                remote_path: format!("/uploads/{}", filename),
                url: Some(format!("https://example.com/{}", filename)),
                size: data.len(),
                duration_ms: 100,
            })
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn is_enabled(&self) -> bool {
            self.enabled
        }
    }

    #[tokio::test]
    async fn test_mock_uploader() {
        let uploader = MockUploader {
            name: "test".to_string(),
            enabled: true,
        };

        assert_eq!(uploader.name(), "test");
        assert!(uploader.is_enabled());

        let data = b"test data";
        let result = uploader.upload(data, "test.png").await.unwrap();

        assert_eq!(result.remote_path, "/uploads/test.png");
        assert_eq!(result.url, Some("https://example.com/test.png".to_string()));
        assert_eq!(result.size, 9);
    }

    #[test]
    fn test_uploader_info() {
        let uploader = MockUploader {
            name: "test".to_string(),
            enabled: true,
        };

        let info = uploader.info();
        assert_eq!(info.name, "test");
        assert!(info.enabled);
    }
}
