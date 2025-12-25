use crate::config::{HistoryConfig, HistoryMode};
use crate::error::{Result, SnaptoError};
use chrono::{DateTime, Utc};
use image::{imageops::FilterType, ImageFormat};
use rusqlite::{params, Connection};
use std::fs;
use std::io::Cursor;
use std::path::PathBuf;

/// Entry in the upload history
#[derive(Debug, Clone)]
pub struct HistoryEntry {
    pub id: i64,
    pub filename: String,
    pub remote_path: String,
    pub url: Option<String>,
    pub destination: String,
    pub size: usize,
    pub created_at: DateTime<Utc>,
    pub thumbnail_path: Option<String>,
    pub local_copy_path: Option<String>,
}

/// Manages the upload history using SQLite
pub struct HistoryManager {
    conn: Connection,
    config: HistoryConfig,
}

impl HistoryManager {
    /// Creates a new HistoryManager with the given configuration
    pub fn new(config: HistoryConfig) -> Result<Self> {
        // Expand path
        let path = shellexpand::tilde(&config.path.to_string_lossy()).to_string();
        let db_path = PathBuf::from(path).join("history.db");

        // Create parent directory if it doesn't exist
        if let Some(parent) = db_path.parent() {
            if !parent.exists() {
                fs::create_dir_all(parent)?;
            }
        }

        // Open database connection
        let conn = Connection::open(&db_path)
            .map_err(|e| SnaptoError::Database(format!("Failed to open database: {}", e)))?;

        let mut manager = Self { conn, config };
        manager.init_db()?;

        Ok(manager)
    }

    /// Initializes the database schema
    fn init_db(&mut self) -> Result<()> {
        self.conn.execute(
            "CREATE TABLE IF NOT EXISTS history (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                filename TEXT NOT NULL,
                remote_path TEXT NOT NULL,
                url TEXT,
                destination TEXT NOT NULL,
                size INTEGER NOT NULL,
                created_at TEXT NOT NULL,
                thumbnail_path TEXT,
                local_copy_path TEXT
            )",
            [],
        )?;

        // Create indexes for better query performance
        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON history(created_at DESC)",
            [],
        )?;

        self.conn.execute(
            "CREATE INDEX IF NOT EXISTS idx_filename ON history(filename)",
            [],
        )?;

        Ok(())
    }

    /// Adds a new entry to the history
    pub fn add(&self, entry: &HistoryEntry, image_data: Option<&[u8]>) -> Result<i64> {
        if !self.config.enabled {
            return Ok(0);
        }

        let mut thumbnail_path = None;
        let mut local_copy_path = None;

        // Process image based on history mode
        if let Some(data) = image_data {
            match self.config.mode {
                HistoryMode::Thumbnails => {
                    thumbnail_path = Some(self.save_thumbnail(data, &entry.filename)?);
                }
                HistoryMode::Full => {
                    thumbnail_path = Some(self.save_thumbnail(data, &entry.filename)?);
                    local_copy_path = Some(self.save_full_image(data, &entry.filename)?);
                }
                HistoryMode::Metadata => {
                    // Only metadata, no files saved
                }
            }
        }

        // Insert into database
        self.conn.execute(
            "INSERT INTO history (filename, remote_path, url, destination, size, created_at, thumbnail_path, local_copy_path)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                entry.filename,
                entry.remote_path,
                entry.url,
                entry.destination,
                entry.size as i64,
                entry.created_at.to_rfc3339(),
                thumbnail_path,
                local_copy_path,
            ],
        )?;

        let id = self.conn.last_insert_rowid();

        // Cleanup old entries if needed
        self.cleanup()?;

        Ok(id)
    }

    /// Gets the most recent N entries
    pub fn get_recent(&self, limit: usize) -> Result<Vec<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, remote_path, url, destination, size, created_at, thumbnail_path, local_copy_path
             FROM history
             ORDER BY created_at DESC
             LIMIT ?1"
        )?;

        let entries = stmt.query_map(params![limit as i64], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                filename: row.get(1)?,
                remote_path: row.get(2)?,
                url: row.get(3)?,
                destination: row.get(4)?,
                size: row.get::<_, i64>(5)? as usize,
                created_at: {
                    let date_str: String = row.get(6)?;
                    DateTime::parse_from_rfc3339(&date_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())
                },
                thumbnail_path: row.get(7)?,
                local_copy_path: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Searches entries by filename or URL
    pub fn search(&self, query: &str) -> Result<Vec<HistoryEntry>> {
        let search_pattern = format!("%{}%", query);

        let mut stmt = self.conn.prepare(
            "SELECT id, filename, remote_path, url, destination, size, created_at, thumbnail_path, local_copy_path
             FROM history
             WHERE filename LIKE ?1 OR url LIKE ?1
             ORDER BY created_at DESC
             LIMIT 100"
        )?;

        let entries = stmt.query_map(params![search_pattern], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                filename: row.get(1)?,
                remote_path: row.get(2)?,
                url: row.get(3)?,
                destination: row.get(4)?,
                size: row.get::<_, i64>(5)? as usize,
                created_at: {
                    let date_str: String = row.get(6)?;
                    DateTime::parse_from_rfc3339(&date_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())
                },
                thumbnail_path: row.get(7)?,
                local_copy_path: row.get(8)?,
            })
        })?
        .collect::<std::result::Result<Vec<_>, _>>()?;

        Ok(entries)
    }

    /// Deletes an entry from the history
    pub fn delete(&self, id: i64) -> Result<()> {
        // Get the entry first to delete associated files
        let mut stmt = self.conn.prepare(
            "SELECT thumbnail_path, local_copy_path FROM history WHERE id = ?1"
        )?;

        let result: rusqlite::Result<(Option<String>, Option<String>)> = stmt.query_row(
            params![id],
            |row| Ok((row.get(0)?, row.get(1)?))
        );

        if let Ok((thumb, local)) = result {
            // Delete thumbnail file
            if let Some(thumb_path) = thumb {
                let _ = fs::remove_file(&thumb_path);
            }

            // Delete local copy file
            if let Some(local_path) = local {
                let _ = fs::remove_file(&local_path);
            }
        }

        // Delete database entry
        self.conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;

        Ok(())
    }

    /// Cleans up old entries according to max_items configuration
    pub fn cleanup(&self) -> Result<usize> {
        if self.config.max_entries == 0 {
            return Ok(0);
        }

        // Get IDs of entries to delete (everything beyond max_entries)
        let mut stmt = self.conn.prepare(
            "SELECT id, thumbnail_path, local_copy_path
             FROM history
             ORDER BY created_at DESC
             LIMIT -1 OFFSET ?1"
        )?;

        let to_delete: Vec<(i64, Option<String>, Option<String>)> = stmt
            .query_map(params![self.config.max_entries as i64], |row| {
                Ok((row.get(0)?, row.get(1)?, row.get(2)?))
            })?
            .collect::<std::result::Result<Vec<_>, _>>()?;

        let count = to_delete.len();

        for (id, thumb, local) in to_delete {
            // Delete files
            if let Some(thumb_path) = thumb {
                let _ = fs::remove_file(&thumb_path);
            }
            if let Some(local_path) = local {
                let _ = fs::remove_file(&local_path);
            }

            // Delete from database
            self.conn.execute("DELETE FROM history WHERE id = ?1", params![id])?;
        }

        Ok(count)
    }

    /// Generates and saves a thumbnail from image data
    fn generate_thumbnail(&self, image_data: &[u8]) -> Result<Vec<u8>> {
        // Load image
        let img = image::load_from_memory(image_data)
            .map_err(|e| SnaptoError::ImageProcessing(format!("Failed to load image: {}", e)))?;

        // Resize to thumbnail (max 200x200, maintaining aspect ratio)
        let thumbnail = img.resize(200, 200, FilterType::Lanczos3);

        // Encode as PNG
        let mut buffer = Vec::new();
        let mut cursor = Cursor::new(&mut buffer);

        thumbnail.write_to(&mut cursor, ImageFormat::Png)
            .map_err(|e| SnaptoError::ImageProcessing(format!("Failed to encode thumbnail: {}", e)))?;

        Ok(buffer)
    }

    /// Saves a thumbnail to disk
    fn save_thumbnail(&self, image_data: &[u8], filename: &str) -> Result<String> {
        let path = shellexpand::tilde(&self.config.path.to_string_lossy()).to_string();
        let thumbnails_dir = PathBuf::from(path).join("thumbnails");

        if !thumbnails_dir.exists() {
            fs::create_dir_all(&thumbnails_dir)?;
        }

        let thumbnail_data = self.generate_thumbnail(image_data)?;
        let thumbnail_filename = format!("thumb_{}.png", Self::sanitize_filename(filename));
        let thumbnail_path = thumbnails_dir.join(&thumbnail_filename);

        fs::write(&thumbnail_path, thumbnail_data)?;

        Ok(thumbnail_path.to_string_lossy().to_string())
    }

    /// Saves the full image to disk
    fn save_full_image(&self, image_data: &[u8], filename: &str) -> Result<String> {
        let path = shellexpand::tilde(&self.config.path.to_string_lossy()).to_string();
        let images_dir = PathBuf::from(path).join("images");

        if !images_dir.exists() {
            fs::create_dir_all(&images_dir)?;
        }

        let image_path = images_dir.join(filename);
        fs::write(&image_path, image_data)?;

        Ok(image_path.to_string_lossy().to_string())
    }

    /// Sanitizes a filename by removing/replacing invalid characters
    fn sanitize_filename(filename: &str) -> String {
        filename
            .chars()
            .map(|c| match c {
                '/' | '\\' | ':' | '*' | '?' | '"' | '<' | '>' | '|' => '_',
                _ => c,
            })
            .collect()
    }

    /// Gets total count of entries
    pub fn count(&self) -> Result<usize> {
        let count: i64 = self.conn.query_row(
            "SELECT COUNT(*) FROM history",
            [],
            |row| row.get(0)
        )?;

        Ok(count as usize)
    }

    /// Gets an entry by ID
    pub fn get_by_id(&self, id: i64) -> Result<Option<HistoryEntry>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, filename, remote_path, url, destination, size, created_at, thumbnail_path, local_copy_path
             FROM history
             WHERE id = ?1"
        )?;

        let result = stmt.query_row(params![id], |row| {
            Ok(HistoryEntry {
                id: row.get(0)?,
                filename: row.get(1)?,
                remote_path: row.get(2)?,
                url: row.get(3)?,
                destination: row.get(4)?,
                size: row.get::<_, i64>(5)? as usize,
                created_at: {
                    let date_str: String = row.get(6)?;
                    DateTime::parse_from_rfc3339(&date_str)
                        .map(|dt| dt.with_timezone(&Utc))
                        .unwrap_or_else(|_| Utc::now())
                },
                thumbnail_path: row.get(7)?,
                local_copy_path: row.get(8)?,
            })
        });

        match result {
            Ok(entry) => Ok(Some(entry)),
            Err(rusqlite::Error::QueryReturnedNoRows) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    /// Clears all history
    pub fn clear_all(&self) -> Result<()> {
        // Get all entries to delete files
        let entries = self.get_recent(usize::MAX)?;

        for entry in entries {
            // Delete thumbnail file
            if let Some(thumb_path) = entry.thumbnail_path {
                let _ = fs::remove_file(&thumb_path);
            }

            // Delete local copy file
            if let Some(local_path) = entry.local_copy_path {
                let _ = fs::remove_file(&local_path);
            }
        }

        // Delete all database entries
        self.conn.execute("DELETE FROM history", [])?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use uuid::Uuid;

    fn test_config() -> HistoryConfig {
        // Use unique directory for each test to avoid conflicts
        let temp_dir = env::temp_dir().join(format!("snapto_test_{}", Uuid::new_v4()));
        HistoryConfig {
            enabled: true,
            mode: HistoryMode::Metadata,
            retention_days: 30,
            max_entries: 100,
            path: temp_dir,
        }
    }

    #[test]
    fn test_history_manager_creation() {
        let config = test_config();
        let manager = HistoryManager::new(config);
        assert!(manager.is_ok());
    }

    #[test]
    fn test_add_and_get_entry() {
        let config = test_config();
        let manager = HistoryManager::new(config).unwrap();

        let entry = HistoryEntry {
            id: 0,
            filename: "test.png".to_string(),
            remote_path: "/screenshots/test.png".to_string(),
            url: Some("https://example.com/test.png".to_string()),
            destination: "my-server".to_string(),
            size: 12345,
            created_at: Utc::now(),
            thumbnail_path: None,
            local_copy_path: None,
        };

        let id = manager.add(&entry, None).unwrap();
        assert!(id > 0);

        let entries = manager.get_recent(10).unwrap();
        assert_eq!(entries.len(), 1);
        assert_eq!(entries[0].filename, "test.png");
    }

    #[test]
    fn test_search() {
        let config = test_config();
        let manager = HistoryManager::new(config).unwrap();

        let entry = HistoryEntry {
            id: 0,
            filename: "screenshot_test.png".to_string(),
            remote_path: "/screenshots/screenshot_test.png".to_string(),
            url: Some("https://example.com/screenshot_test.png".to_string()),
            destination: "my-server".to_string(),
            size: 12345,
            created_at: Utc::now(),
            thumbnail_path: None,
            local_copy_path: None,
        };

        manager.add(&entry, None).unwrap();

        let results = manager.search("screenshot").unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].filename, "screenshot_test.png");
    }

    #[test]
    fn test_delete() {
        let config = test_config();
        let manager = HistoryManager::new(config).unwrap();

        let entry = HistoryEntry {
            id: 0,
            filename: "to_delete.png".to_string(),
            remote_path: "/screenshots/to_delete.png".to_string(),
            url: None,
            destination: "my-server".to_string(),
            size: 12345,
            created_at: Utc::now(),
            thumbnail_path: None,
            local_copy_path: None,
        };

        let id = manager.add(&entry, None).unwrap();
        assert!(manager.delete(id).is_ok());

        let entry = manager.get_by_id(id).unwrap();
        assert!(entry.is_none());
    }

    #[test]
    fn test_cleanup() {
        let mut config = test_config();
        config.max_entries = 5;
        let manager = HistoryManager::new(config).unwrap();

        // Add 10 entries
        for i in 0..10 {
            let entry = HistoryEntry {
                id: 0,
                filename: format!("test_{}.png", i),
                remote_path: format!("/screenshots/test_{}.png", i),
                url: None,
                destination: "my-server".to_string(),
                size: 12345,
                created_at: Utc::now(),
                thumbnail_path: None,
                local_copy_path: None,
            };
            manager.add(&entry, None).unwrap();
        }

        let count = manager.count().unwrap();
        assert_eq!(count, 5); // Should only have 5 entries due to cleanup
    }
}
