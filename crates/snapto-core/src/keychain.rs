use crate::config::SecurityConfig;
use crate::error::{Result, SnaptoError};
use aes_gcm::{
    aead::{Aead, KeyInit, OsRng},
    Aes256Gcm, Nonce,
};
use argon2::{
    password_hash::{rand_core::RngCore, SaltString},
    Argon2, PasswordHasher,
};
use keyring::Entry;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

const SERVICE_NAME: &str = "snapto";
const NONCE_SIZE: usize = 12;

/// Manages secure credential storage
pub struct KeychainManager {
    use_system_keychain: bool,
    encrypted_file_path: Option<PathBuf>,
}

/// Encrypted credentials store format
#[derive(Debug, Serialize, Deserialize)]
struct EncryptedStore {
    /// Salt for key derivation
    salt: String,
    /// Nonce for AES-GCM
    nonce: Vec<u8>,
    /// Encrypted data
    data: Vec<u8>,
}

impl KeychainManager {
    /// Creates a new KeychainManager with the given configuration
    pub fn new(config: &SecurityConfig) -> Self {
        let encrypted_file_path = if !config.use_system_keychain {
            // Use encrypted file fallback
            let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
            Some(PathBuf::from(home).join(".snapto").join("credentials.enc"))
        } else {
            None
        };

        Self {
            use_system_keychain: config.use_system_keychain,
            encrypted_file_path,
        }
    }

    /// Stores a credential
    pub fn set(&self, key: &str, value: &str) -> Result<()> {
        if self.use_system_keychain {
            self.set_system_keychain(key, value)
        } else {
            self.set_encrypted_file(key, value)
        }
    }

    /// Retrieves a credential
    pub fn get(&self, key: &str) -> Result<Option<String>> {
        if self.use_system_keychain {
            self.get_system_keychain(key)
        } else {
            self.get_encrypted_file(key)
        }
    }

    /// Deletes a credential
    pub fn delete(&self, key: &str) -> Result<()> {
        if self.use_system_keychain {
            self.delete_system_keychain(key)
        } else {
            self.delete_encrypted_file(key)
        }
    }

    /// Lists all stored credential keys
    pub fn list_keys(&self) -> Result<Vec<String>> {
        if self.use_system_keychain {
            // System keychain doesn't support listing, so we maintain a list key
            match self.get_system_keychain("__snapto_keys__")? {
                Some(keys_json) => {
                    let keys: Vec<String> = serde_json::from_str(&keys_json)
                        .map_err(|e| SnaptoError::Keychain(format!("Failed to parse keys list: {}", e)))?;
                    Ok(keys)
                }
                None => Ok(Vec::new()),
            }
        } else {
            let store = self.load_encrypted_store()?;
            Ok(store.keys().cloned().collect())
        }
    }

    // System keychain methods

    fn set_system_keychain(&self, key: &str, value: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to create keychain entry: {}", e)))?;

        entry.set_password(value)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to set password: {}", e)))?;

        // Update keys list
        self.update_keys_list(key, true)?;

        Ok(())
    }

    fn get_system_keychain(&self, key: &str) -> Result<Option<String>> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to create keychain entry: {}", e)))?;

        match entry.get_password() {
            Ok(password) => Ok(Some(password)),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(SnaptoError::Keychain(format!("Failed to get password: {}", e))),
        }
    }

    fn delete_system_keychain(&self, key: &str) -> Result<()> {
        let entry = Entry::new(SERVICE_NAME, key)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to create keychain entry: {}", e)))?;

        match entry.delete_credential() {
            Ok(_) => {
                // Update keys list
                self.update_keys_list(key, false)?;
                Ok(())
            }
            Err(keyring::Error::NoEntry) => Ok(()), // Already deleted
            Err(e) => Err(SnaptoError::Keychain(format!("Failed to delete password: {}", e))),
        }
    }

    fn update_keys_list(&self, key: &str, add: bool) -> Result<()> {
        if key == "__snapto_keys__" {
            return Ok(()); // Don't track the keys list itself
        }

        let mut keys = match self.get_system_keychain("__snapto_keys__")? {
            Some(keys_json) => {
                serde_json::from_str::<Vec<String>>(&keys_json)
                    .unwrap_or_else(|_| Vec::new())
            }
            None => Vec::new(),
        };

        if add {
            if !keys.contains(&key.to_string()) {
                keys.push(key.to_string());
            }
        } else {
            keys.retain(|k| k != key);
        }

        let keys_json = serde_json::to_string(&keys)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to serialize keys list: {}", e)))?;

        self.set_system_keychain("__snapto_keys__", &keys_json)?;

        Ok(())
    }

    // Encrypted file methods

    fn set_encrypted_file(&self, key: &str, value: &str) -> Result<()> {
        let mut store = self.load_encrypted_store()?;
        store.insert(key.to_string(), value.to_string());
        self.save_encrypted_store(&store)?;
        Ok(())
    }

    fn get_encrypted_file(&self, key: &str) -> Result<Option<String>> {
        let store = self.load_encrypted_store()?;
        Ok(store.get(key).cloned())
    }

    fn delete_encrypted_file(&self, key: &str) -> Result<()> {
        let mut store = self.load_encrypted_store()?;
        store.remove(key);
        self.save_encrypted_store(&store)?;
        Ok(())
    }

    /// Loads the encrypted credentials store
    fn load_encrypted_store(&self) -> Result<HashMap<String, String>> {
        let file_path = self.encrypted_file_path.as_ref()
            .ok_or_else(|| SnaptoError::Keychain("No encrypted file path configured".to_string()))?;

        if !file_path.exists() {
            return Ok(HashMap::new());
        }

        // Read encrypted file
        let encrypted_content = fs::read_to_string(file_path)?;
        let encrypted_store: EncryptedStore = serde_json::from_str(&encrypted_content)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to parse encrypted store: {}", e)))?;

        // Get master password from environment or prompt
        let master_password = self.get_master_password()?;

        // Decrypt data
        let decrypted_json = self.decrypt(
            &encrypted_store.data,
            &master_password,
            &encrypted_store.salt,
            &encrypted_store.nonce,
        )?;

        // Parse JSON
        let store: HashMap<String, String> = serde_json::from_str(&decrypted_json)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to parse store: {}", e)))?;

        Ok(store)
    }

    /// Saves the encrypted credentials store
    fn save_encrypted_store(&self, store: &HashMap<String, String>) -> Result<()> {
        let file_path = self.encrypted_file_path.as_ref()
            .ok_or_else(|| SnaptoError::Keychain("No encrypted file path configured".to_string()))?;

        // Create parent directory if it doesn't exist
        if let Some(parent) = file_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Serialize store to JSON
        let store_json = serde_json::to_string(store)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to serialize store: {}", e)))?;

        // Get master password
        let master_password = self.get_master_password()?;

        // Generate salt and nonce
        let salt = SaltString::generate(&mut OsRng);
        let mut nonce_bytes = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce_bytes);

        // Encrypt data
        let encrypted_data = self.encrypt(&store_json, &master_password, salt.as_str(), &nonce_bytes)?;

        // Create encrypted store
        let encrypted_store = EncryptedStore {
            salt: salt.as_str().to_string(),
            nonce: nonce_bytes.to_vec(),
            data: encrypted_data,
        };

        // Write to file
        let encrypted_json = serde_json::to_string_pretty(&encrypted_store)
            .map_err(|e| SnaptoError::Keychain(format!("Failed to serialize encrypted store: {}", e)))?;

        fs::write(file_path, encrypted_json)?;

        Ok(())
    }

    /// Encrypts data using AES-256-GCM
    fn encrypt(&self, data: &str, master_password: &str, salt: &str, nonce: &[u8]) -> Result<Vec<u8>> {
        // Derive key from password using Argon2
        let key = self.derive_key(master_password, salt)?;

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| SnaptoError::Encryption(format!("Failed to create cipher: {}", e)))?;

        // Create nonce
        let nonce = Nonce::from_slice(nonce);

        // Encrypt
        let ciphertext = cipher.encrypt(nonce, data.as_bytes())
            .map_err(|e| SnaptoError::Encryption(format!("Encryption failed: {}", e)))?;

        Ok(ciphertext)
    }

    /// Decrypts data using AES-256-GCM
    fn decrypt(&self, data: &[u8], master_password: &str, salt: &str, nonce: &[u8]) -> Result<String> {
        // Derive key from password using Argon2
        let key = self.derive_key(master_password, salt)?;

        // Create cipher
        let cipher = Aes256Gcm::new_from_slice(&key)
            .map_err(|e| SnaptoError::Encryption(format!("Failed to create cipher: {}", e)))?;

        // Create nonce
        let nonce = Nonce::from_slice(nonce);

        // Decrypt
        let plaintext = cipher.decrypt(nonce, data)
            .map_err(|e| SnaptoError::Encryption(format!("Decryption failed: {}", e)))?;

        // Convert to string
        String::from_utf8(plaintext)
            .map_err(|e| SnaptoError::Encryption(format!("Invalid UTF-8: {}", e)))
    }

    /// Derives a 256-bit key from password using Argon2
    fn derive_key(&self, password: &str, salt: &str) -> Result<Vec<u8>> {
        let argon2 = Argon2::default();

        let salt_string = SaltString::from_b64(salt)
            .map_err(|e| SnaptoError::Encryption(format!("Invalid salt: {}", e)))?;

        let password_hash = argon2.hash_password(password.as_bytes(), &salt_string)
            .map_err(|e| SnaptoError::Encryption(format!("Failed to hash password: {}", e)))?;

        // Extract the hash bytes (32 bytes for Argon2)
        let hash_bytes = password_hash.hash
            .ok_or_else(|| SnaptoError::Encryption("No hash produced".to_string()))?;

        Ok(hash_bytes.as_bytes().to_vec())
    }

    /// Gets the master password from environment or prompts user
    fn get_master_password(&self) -> Result<String> {
        // Try to get from environment variable first
        if let Ok(password) = std::env::var("SNAPTO_MASTER_PASSWORD") {
            return Ok(password);
        }

        // For now, return a default password
        // In a real implementation, this would prompt the user
        Ok("snapto-default-password".to_string())
    }

    /// Clears all credentials
    pub fn clear_all(&self) -> Result<()> {
        if self.use_system_keychain {
            let keys = self.list_keys()?;
            for key in keys {
                self.delete_system_keychain(&key)?;
            }
            // Also delete the keys list
            let _ = self.delete_system_keychain("__snapto_keys__");
        } else {
            let file_path = self.encrypted_file_path.as_ref()
                .ok_or_else(|| SnaptoError::Keychain("No encrypted file path configured".to_string()))?;

            if file_path.exists() {
                fs::remove_file(file_path)?;
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_config() -> SecurityConfig {
        SecurityConfig {
            use_system_keychain: false, // Use encrypted file for tests
            encrypt_credentials: true,
        }
    }

    #[test]
    fn test_keychain_manager_creation() {
        let config = test_config();
        let manager = KeychainManager::new(&config);
        assert!(!manager.use_system_keychain);
        assert!(manager.encrypted_file_path.is_some());
    }

    #[test]
    fn test_set_and_get() {
        let config = test_config();
        let manager = KeychainManager::new(&config);

        // Set a credential
        manager.set("test_key", "test_value").unwrap();

        // Get the credential
        let value = manager.get("test_key").unwrap();
        assert_eq!(value, Some("test_value".to_string()));

        // Clean up
        let _ = manager.clear_all();
    }

    #[test]
    fn test_delete() {
        let config = test_config();
        let manager = KeychainManager::new(&config);

        // Set a credential
        manager.set("test_delete", "value").unwrap();

        // Verify it exists
        assert!(manager.get("test_delete").unwrap().is_some());

        // Delete it
        manager.delete("test_delete").unwrap();

        // Verify it's gone
        assert!(manager.get("test_delete").unwrap().is_none());

        // Clean up
        let _ = manager.clear_all();
    }

    #[test]
    fn test_list_keys() {
        let config = test_config();
        let manager = KeychainManager::new(&config);

        // Set multiple credentials
        manager.set("key1", "value1").unwrap();
        manager.set("key2", "value2").unwrap();
        manager.set("key3", "value3").unwrap();

        // List keys
        let keys = manager.list_keys().unwrap();
        assert_eq!(keys.len(), 3);
        assert!(keys.contains(&"key1".to_string()));
        assert!(keys.contains(&"key2".to_string()));
        assert!(keys.contains(&"key3".to_string()));

        // Clean up
        let _ = manager.clear_all();
    }

    #[test]
    fn test_encryption_decryption() {
        let config = test_config();
        let manager = KeychainManager::new(&config);

        let original = "sensitive data";
        let password = "test_password";
        let salt = SaltString::generate(&mut OsRng);
        let mut nonce = [0u8; NONCE_SIZE];
        OsRng.fill_bytes(&mut nonce);

        // Encrypt
        let encrypted = manager.encrypt(original, password, salt.as_str(), &nonce).unwrap();

        // Decrypt
        let decrypted = manager.decrypt(&encrypted, password, salt.as_str(), &nonce).unwrap();

        assert_eq!(original, decrypted);
    }

    #[test]
    fn test_clear_all() {
        let config = test_config();
        let manager = KeychainManager::new(&config);

        // Set multiple credentials
        manager.set("key1", "value1").unwrap();
        manager.set("key2", "value2").unwrap();

        // Verify they exist
        assert_eq!(manager.list_keys().unwrap().len(), 2);

        // Clear all
        manager.clear_all().unwrap();

        // Verify they're gone
        assert_eq!(manager.list_keys().unwrap().len(), 0);
    }
}
