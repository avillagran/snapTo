# SnapTo Core

Core library for SnapTo - a screenshot sharing tool with clipboard integration, SFTP upload, and history management.

## Features

- **History Management**: SQLite-based history with three modes:
  - Metadata only
  - Metadata + thumbnails (200x200 PNG)
  - Metadata + full image copies

- **Secure Credentials**: Multi-platform keychain integration with encrypted fallback:
  - macOS Keychain
  - Windows Credential Store
  - Linux Secret Service (GNOME Keyring, KWallet)
  - Encrypted file fallback with AES-256-GCM and Argon2

## Modules

### History (`history.rs`)

Manages upload history with SQLite backend.

```rust
use snapto_core::{HistoryConfig, HistoryMode, HistoryManager, HistoryEntry};
use chrono::Utc;
use std::path::PathBuf;

// Configure history
let config = HistoryConfig {
    enabled: true,
    mode: HistoryMode::Thumbnails, // Save metadata + thumbnails
    retention_days: 30,
    max_entries: 1000,
    path: PathBuf::from("~/.snapto"),
};

// Create manager
let manager = HistoryManager::new(config)?;

// Add entry
let entry = HistoryEntry {
    id: 0,
    filename: "screenshot.png".to_string(),
    remote_path: "/screenshots/screenshot.png".to_string(),
    url: Some("https://example.com/screenshot.png".to_string()),
    destination: "my-server".to_string(),
    size: 12345,
    created_at: Utc::now(),
    thumbnail_path: None,
    local_copy_path: None,
};

// Save with image data (will generate thumbnail)
let image_data = std::fs::read("screenshot.png")?;
let id = manager.add(&entry, Some(&image_data))?;

// Get recent entries
let recent = manager.get_recent(10)?;

// Search
let results = manager.search("screenshot")?;

// Delete entry
manager.delete(id)?;
```

### Keychain (`keychain.rs`)

Secure credential management with system keychain integration.

```rust
use snapto_core::{KeychainManager, SecurityConfig};

// Configure to use system keychain
let config = SecurityConfig {
    use_system_keychain: true,
    encrypt_credentials: false,
};

let keychain = KeychainManager::new(&config);

// Store credential
keychain.set("server_password", "my-secure-password")?;

// Retrieve credential
if let Some(password) = keychain.get("server_password")? {
    println!("Password: {}", password);
}

// List all keys
let keys = keychain.list_keys()?;

// Delete credential
keychain.delete("server_password")?;
```

#### Encrypted File Fallback

When system keychain is not available or disabled:

```rust
let config = SecurityConfig {
    use_system_keychain: false,
    encrypt_credentials: true,
};

let keychain = KeychainManager::new(&config);

// Set SNAPTO_MASTER_PASSWORD environment variable
// or it will use default password (not recommended for production)
std::env::set_var("SNAPTO_MASTER_PASSWORD", "my-master-password");

// Same API, but stores encrypted in ~/.snapto/credentials.enc
keychain.set("api_key", "secret-api-key")?;
```

## Database Schema

### History Table

```sql
CREATE TABLE history (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    filename TEXT NOT NULL,
    remote_path TEXT NOT NULL,
    url TEXT,
    destination TEXT NOT NULL,
    size INTEGER NOT NULL,
    created_at TEXT NOT NULL,
    thumbnail_path TEXT,
    local_copy_path TEXT
);

CREATE INDEX idx_created_at ON history(created_at DESC);
CREATE INDEX idx_filename ON history(filename);
```

## File Structure

```
~/.snapto/
├── config.toml           # Configuration file
├── history.db            # SQLite database
├── credentials.enc       # Encrypted credentials (if not using system keychain)
├── thumbnails/           # Thumbnail images (200x200 PNG)
│   └── thumb_*.png
└── images/               # Full image copies (if mode is Full)
    └── *.png
```

## Security

### Keychain Integration

- **macOS**: Uses Keychain Services via keyring crate
- **Windows**: Uses Windows Credential Manager
- **Linux**: Uses Secret Service API (libsecret)

### Encrypted Storage

When system keychain is not available:

- **Encryption**: AES-256-GCM
- **Key Derivation**: Argon2 with random salt
- **Master Password**: From `SNAPTO_MASTER_PASSWORD` environment variable
- **Storage**: `~/.snapto/credentials.enc`

## Error Handling

All operations return `Result<T, SnaptoError>`:

```rust
use snapto_core::{Result, SnaptoError};

fn upload_screenshot() -> Result<()> {
    // Operations that can fail
    let manager = HistoryManager::new(config)?;
    // ...
    Ok(())
}
```

Error types:
- `SnaptoError::Database`: SQLite errors
- `SnaptoError::Keychain`: Keychain access errors
- `SnaptoError::Encryption`: Encryption/decryption errors
- `SnaptoError::ImageProcessing`: Image manipulation errors
- `SnaptoError::Io`: File system errors

## Testing

```bash
# Run all tests
cargo test -p snapto-core

# Run specific module tests
cargo test -p snapto-core history
cargo test -p snapto-core keychain

# Run with output
cargo test -p snapto-core -- --nocapture
```

## License

MIT
