//! SnapTo Core Library
//!
//! This crate provides the core functionality for the SnapTo screenshot sharing tool.
//! It includes clipboard management, SSH/SFTP uploading, and various utility functions.

pub mod clipboard;
pub mod config;
pub mod error;
pub mod history;
pub mod keychain;
pub mod naming;
pub mod upload;

// Re-export commonly used types
pub use clipboard::ClipboardManager;
pub use config::{Config, GeneralConfig, HistoryConfig, HistoryMode, NamingConfig, SecurityConfig, UploadConfig, ClipboardCopyMode};
pub use error::{Result, SnaptoError};
pub use history::{HistoryEntry, HistoryManager};
pub use keychain::KeychainManager;
pub use naming::{TemplateParser, generate_filename};
pub use upload::{UploadResult, Uploader, UploaderInfo};
pub use upload::sftp::SftpUploader;
pub use upload::local::LocalUploader;
pub use upload::ssh::SshUploader;
