use crate::error::{ConfigError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

/// Configuración principal de SnapTo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub general: GeneralConfig,
    pub naming: NamingConfig,
    pub history: HistoryConfig,
    pub uploads: HashMap<String, UploadConfig>,
    pub security: SecurityConfig,
}

/// Configuración general
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GeneralConfig {
    /// Directorio donde se guardan las capturas localmente (opcional)
    pub local_save_dir: Option<String>,
    /// Copiar URL al portapapeles después de subir
    pub copy_url_to_clipboard: bool,
    /// Qué copiar al portapapeles: auto (URL si existe, sino path), url, path
    #[serde(default)]
    pub clipboard_copy_mode: ClipboardCopyMode,
    /// Mostrar notificaciones
    pub show_notifications: bool,
    /// Uploader principal (para clipboard)
    pub default_uploader: String,
    /// Uploaders adicionales a ejecutar junto con el principal
    #[serde(default)]
    pub additional_uploaders: Vec<String>,
}

/// Modo de copia al portapapeles
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum ClipboardCopyMode {
    /// Copiar URL si existe, sino la ruta del archivo
    #[default]
    Auto,
    /// Solo copiar URL (no copiar nada si no hay URL)
    Url,
    /// Siempre copiar la ruta del archivo remoto/local
    Path,
}

/// Configuración de nombres de archivo
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NamingConfig {
    /// Template para nombres de archivo
    /// Soporta: {date}, {time}, {random:N}, {uuid}, {counter}
    pub template: String,
    /// Formato de fecha para {date}
    pub date_format: String,
    /// Formato de hora para {time}
    pub time_format: String,
    /// Extensión por defecto
    pub default_extension: String,
}

/// Configuración de historial
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HistoryConfig {
    /// Habilitar historial
    pub enabled: bool,
    /// Modo de historial
    pub mode: HistoryMode,
    /// Días de retención (0 = infinito)
    pub retention_days: u32,
    /// Límite máximo de registros
    pub max_entries: usize,
    /// Ruta donde se guarda el historial
    pub path: PathBuf,
}

/// Modo de almacenamiento del historial
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "lowercase")]
pub enum HistoryMode {
    /// Solo metadatos
    Metadata,
    /// Metadatos + thumbnail
    Thumbnails,
    /// Metadatos + copia completa
    Full,
}

/// Configuración de un uploader
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UploadConfig {
    /// Tipo de uploader (sftp, local, etc.)
    #[serde(rename = "type")]
    pub uploader_type: String,
    /// Habilitado
    pub enabled: bool,
    /// Host (para SFTP)
    pub host: Option<String>,
    /// Puerto (para SFTP)
    pub port: Option<u16>,
    /// Usuario (para SFTP)
    pub username: Option<String>,
    /// Ruta remota
    pub remote_path: Option<String>,
    /// URL base para generar enlaces
    pub base_url: Option<String>,
    /// Ruta local (para uploader local)
    pub local_path: Option<String>,
    /// Usar autenticación por clave
    pub use_key_auth: Option<bool>,
    /// Ruta de la clave privada
    pub key_path: Option<String>,
    /// Timeout de conexión en segundos
    pub timeout: Option<u64>,
}

/// Configuración de seguridad
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecurityConfig {
    /// Usar keychain del sistema para credenciales
    pub use_system_keychain: bool,
    /// Encriptar credenciales en configuración
    pub encrypt_credentials: bool,
}

impl Config {
    /// Obtiene la ruta del archivo de configuración
    pub fn config_path() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| ConfigError::InvalidDirectory("No se pudo obtener HOME".to_string()))?;
        Ok(PathBuf::from(home).join(".snapto").join("config.toml"))
    }

    /// Obtiene el directorio de configuración
    pub fn config_dir() -> Result<PathBuf> {
        let home = std::env::var("HOME")
            .map_err(|_| ConfigError::InvalidDirectory("No se pudo obtener HOME".to_string()))?;
        Ok(PathBuf::from(home).join(".snapto"))
    }

    /// Carga la configuración desde el archivo
    pub fn load() -> Result<Self> {
        let config_path = Self::config_path()?;

        if !config_path.exists() {
            // Si no existe, crear configuración por defecto
            let config = Self::default();
            config.save()?;
            return Ok(config);
        }

        let content = fs::read_to_string(&config_path).map_err(|e| {
            ConfigError::FileNotFound(format!("{}: {}", config_path.display(), e))
        })?;

        let config: Config = toml::from_str(&content)
            .map_err(|e| ConfigError::Parse(format!("Error al parsear TOML: {}", e)))?;

        Ok(config)
    }

    /// Guarda la configuración en el archivo
    pub fn save(&self) -> Result<()> {
        let config_dir = Self::config_dir()?;

        // Crear directorio si no existe
        if !config_dir.exists() {
            fs::create_dir_all(&config_dir).map_err(|e| {
                ConfigError::CreateDirectoryFailed(format!("{}: {}", config_dir.display(), e))
            })?;
        }

        let config_path = Self::config_path()?;
        let content = toml::to_string_pretty(self)
            .map_err(|e| ConfigError::SaveFailed(format!("Error al serializar: {}", e)))?;

        fs::write(&config_path, content).map_err(|e| {
            ConfigError::SaveFailed(format!("{}: {}", config_path.display(), e))
        })?;

        Ok(())
    }

    /// Obtiene la ruta de la base de datos
    pub fn database_path() -> Result<PathBuf> {
        let config_dir = Self::config_dir()?;
        Ok(config_dir.join("history.db"))
    }

    /// Valida la configuración
    pub fn validate(&self) -> Result<()> {
        // Validar que existe al menos un uploader habilitado
        if !self.uploads.values().any(|u| u.enabled) {
            return Err(ConfigError::Invalid(
                "No hay uploaders habilitados".to_string(),
            ).into());
        }

        // Validar que el uploader por defecto existe y está habilitado
        if let Some(default) = self.uploads.get(&self.general.default_uploader) {
            if !default.enabled {
                return Err(ConfigError::Invalid(format!(
                    "El uploader por defecto '{}' está deshabilitado",
                    self.general.default_uploader
                )).into());
            }
        } else {
            return Err(ConfigError::Invalid(format!(
                "El uploader por defecto '{}' no existe",
                self.general.default_uploader
            )).into());
        }

        // Validar configuraciones de uploaders
        for (name, uploader) in &self.uploads {
            if !uploader.enabled {
                continue;
            }

            match uploader.uploader_type.as_str() {
                "sftp" => {
                    if uploader.host.is_none() {
                        return Err(ConfigError::Invalid(format!(
                            "Uploader '{}': host requerido para SFTP",
                            name
                        )).into());
                    }
                    if uploader.username.is_none() {
                        return Err(ConfigError::Invalid(format!(
                            "Uploader '{}': username requerido para SFTP",
                            name
                        )).into());
                    }
                    if uploader.remote_path.is_none() {
                        return Err(ConfigError::Invalid(format!(
                            "Uploader '{}': remote_path requerido para SFTP",
                            name
                        )).into());
                    }
                }
                "local" => {
                    if uploader.local_path.is_none() {
                        return Err(ConfigError::Invalid(format!(
                            "Uploader '{}': local_path requerido para local",
                            name
                        )).into());
                    }
                }
                _ => {
                    return Err(ConfigError::Invalid(format!(
                        "Uploader '{}': tipo '{}' no soportado",
                        name, uploader.uploader_type
                    )).into());
                }
            }
        }

        Ok(())
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut uploads = HashMap::new();

        // Ejemplo de SFTP
        uploads.insert(
            "my-server".to_string(),
            UploadConfig {
                uploader_type: "sftp".to_string(),
                enabled: false,
                host: Some("example.com".to_string()),
                port: Some(22),
                username: Some("user".to_string()),
                remote_path: Some("/var/www/screenshots".to_string()),
                base_url: Some("https://example.com/screenshots".to_string()),
                local_path: None,
                use_key_auth: Some(true),
                key_path: Some("~/.ssh/id_rsa".to_string()),
                timeout: Some(30),
            },
        );

        // Ejemplo de local
        uploads.insert(
            "local".to_string(),
            UploadConfig {
                uploader_type: "local".to_string(),
                enabled: true,
                host: None,
                port: None,
                username: None,
                remote_path: None,
                base_url: None,
                local_path: Some("~/Pictures/Screenshots".to_string()),
                use_key_auth: None,
                key_path: None,
                timeout: None,
            },
        );

        Self {
            general: GeneralConfig {
                local_save_dir: Some("~/Pictures/SnapTo".to_string()),
                copy_url_to_clipboard: true,
                clipboard_copy_mode: ClipboardCopyMode::Auto,
                show_notifications: true,
                default_uploader: "local".to_string(),
                additional_uploaders: vec![],
            },
            naming: NamingConfig {
                template: "screenshot_{date}_{time}".to_string(),
                date_format: "%Y%m%d".to_string(),
                time_format: "%H%M%S".to_string(),
                default_extension: "png".to_string(),
            },
            history: HistoryConfig {
                enabled: true,
                mode: HistoryMode::Thumbnails,
                retention_days: 30,
                max_entries: 1000,
                path: PathBuf::from("~/.snapto"),
            },
            uploads,
            security: SecurityConfig {
                use_system_keychain: true,
                encrypt_credentials: false,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.general.default_uploader, "local");
        assert!(config.uploads.contains_key("local"));
        assert!(config.uploads.contains_key("my-server"));
    }

    #[test]
    fn test_config_serialization() {
        let config = Config::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();
        assert!(toml_str.contains("[general]"));
        assert!(toml_str.contains("[naming]"));
        assert!(toml_str.contains("[history]"));
    }

    #[test]
    fn test_config_validation() {
        let config = Config::default();
        assert!(config.validate().is_ok());
    }
}
