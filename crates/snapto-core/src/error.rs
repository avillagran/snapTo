use thiserror::Error;

#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {0}")]
    FileNotFound(String),

    #[error("Failed to parse configuration: {0}")]
    Parse(String),

    #[error("Invalid configuration: {0}")]
    Invalid(String),

    #[error("Failed to save configuration: {0}")]
    SaveFailed(String),

    #[error("Invalid directory: {0}")]
    InvalidDirectory(String),

    #[error("Failed to create directory: {0}")]
    CreateDirectoryFailed(String),
}

#[derive(Error, Debug)]
pub enum SnaptoError {
    #[error("Clipboard error: {0}")]
    Clipboard(String),

    #[error("No image found in clipboard")]
    NoImageInClipboard,

    #[error("Image processing error: {0}")]
    ImageProcessing(String),

    #[error("SSH connection error: {0}")]
    SshConnection(String),

    #[error("SSH authentication error: {0}")]
    SshAuthentication(String),

    #[error("SFTP error: {0}")]
    Sftp(String),

    #[error("Upload error: {0}")]
    Upload(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Configuration error: {0}")]
    Config(#[from] ConfigError),

    #[error("Invalid path: {0}")]
    InvalidPath(String),

    #[error("Database error: {0}")]
    Database(String),

    #[error("Keychain error: {0}")]
    Keychain(String),

    #[error("Encryption error: {0}")]
    Encryption(String),

    #[error("History error: {0}")]
    History(String),

    #[error("{0}")]
    Other(String),
}

impl From<arboard::Error> for SnaptoError {
    fn from(err: arboard::Error) -> Self {
        SnaptoError::Clipboard(err.to_string())
    }
}

impl From<image::ImageError> for SnaptoError {
    fn from(err: image::ImageError) -> Self {
        SnaptoError::ImageProcessing(err.to_string())
    }
}

impl From<ssh2::Error> for SnaptoError {
    fn from(err: ssh2::Error) -> Self {
        SnaptoError::SshConnection(err.to_string())
    }
}

impl From<rusqlite::Error> for SnaptoError {
    fn from(err: rusqlite::Error) -> Self {
        SnaptoError::Database(err.to_string())
    }
}

impl From<keyring::Error> for SnaptoError {
    fn from(err: keyring::Error) -> Self {
        SnaptoError::Keychain(err.to_string())
    }
}

impl From<aes_gcm::Error> for SnaptoError {
    fn from(err: aes_gcm::Error) -> Self {
        SnaptoError::Encryption(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, SnaptoError>;
