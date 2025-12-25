use arboard::{Clipboard, ImageData};
use image::{ImageBuffer, ImageFormat, Rgba};
use std::io::Cursor;
use tokio::sync::mpsc;
use tracing::{debug, error, info, warn};

use crate::error::{Result, SnaptoError};

/// Manager for clipboard operations
pub struct ClipboardManager {
    clipboard: Clipboard,
}

impl ClipboardManager {
    /// Create a new clipboard manager
    ///
    /// # Errors
    /// Returns an error if the clipboard cannot be accessed
    pub fn new() -> Result<Self> {
        let clipboard = Clipboard::new()?;
        Ok(Self { clipboard })
    }

    /// Read an image from the clipboard and return it as PNG bytes
    ///
    /// # Errors
    /// Returns an error if:
    /// - No image is available in the clipboard
    /// - The image data cannot be converted to PNG format
    pub fn get_image(&mut self) -> Result<Vec<u8>> {
        debug!("Attempting to read image from clipboard");

        let image_data = self
            .clipboard
            .get_image()
            .map_err(|e| {
                debug!("No image in clipboard: {}", e);
                SnaptoError::NoImageInClipboard
            })?;

        debug!(
            "Got image from clipboard: {}x{} pixels",
            image_data.width, image_data.height
        );

        // Convert ImageData to PNG bytes
        let png_bytes = image_data_to_png(&image_data)?;
        
        info!("Successfully converted clipboard image to PNG ({} bytes)", png_bytes.len());
        Ok(png_bytes)
    }

    /// Check if there is an image available in the clipboard
    pub fn has_image(&mut self) -> bool {
        self.clipboard.get_image().is_ok()
    }

    /// Copy text to the clipboard
    ///
    /// # Arguments
    /// * `text` - The text to copy to the clipboard
    ///
    /// # Errors
    /// Returns an error if the clipboard cannot be accessed
    pub fn set_text(&mut self, text: &str) -> Result<()> {
        debug!("Setting clipboard text: {}", text);
        self.clipboard.set_text(text)?;
        info!("Successfully set clipboard text");
        Ok(())
    }

    /// Watch the clipboard for new images
    ///
    /// Returns a receiver that will emit PNG bytes whenever a new image
    /// is detected in the clipboard.
    ///
    /// # Note
    /// This is a blocking operation that polls the clipboard in a separate thread.
    /// The polling interval is currently set to 500ms.
    pub fn watch(&mut self) -> mpsc::Receiver<Vec<u8>> {
        let (tx, rx) = mpsc::channel(10);
        
        info!("Starting clipboard watch mode");

        // We need to create a new clipboard instance for the thread
        // because Clipboard is not Send
        std::thread::spawn(move || {
            let mut clipboard = match Clipboard::new() {
                Ok(cb) => cb,
                Err(e) => {
                    error!("Failed to create clipboard in watch thread: {}", e);
                    return;
                }
            };

            let mut last_image: Option<Vec<u8>> = None;

            loop {
                // Check if the receiver has been dropped
                if tx.is_closed() {
                    info!("Clipboard watch receiver dropped, stopping watch");
                    break;
                }

                // Try to get the current image
                if let Ok(image_data) = clipboard.get_image() {
                    match image_data_to_png(&image_data) {
                        Ok(png_bytes) => {
                            // Check if this is a new image (compare bytes)
                            let is_new = match &last_image {
                                None => true,
                                Some(last) => last != &png_bytes,
                            };

                            if is_new {
                                debug!("New image detected in clipboard");
                                last_image = Some(png_bytes.clone());
                                
                                // Try to send the image
                                if let Err(e) = tx.blocking_send(png_bytes) {
                                    error!("Failed to send clipboard image: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            warn!("Failed to convert clipboard image to PNG: {}", e);
                        }
                    }
                }

                // Sleep for a bit before checking again
                std::thread::sleep(std::time::Duration::from_millis(500));
            }

            info!("Clipboard watch thread exiting");
        });

        rx
    }
}

/// Convert arboard ImageData to PNG bytes
fn image_data_to_png(image_data: &ImageData) -> Result<Vec<u8>> {
    let width = image_data.width;
    let height = image_data.height;
    let bytes = &image_data.bytes;

    debug!("Converting image data to PNG: {}x{}", width, height);

    // Create an ImageBuffer from the raw RGBA bytes
    let img_buffer: ImageBuffer<Rgba<u8>, Vec<u8>> = ImageBuffer::from_raw(
        width as u32,
        height as u32,
        bytes.to_vec(),
    )
    .ok_or_else(|| {
        error!("Failed to create image buffer from clipboard data");
        SnaptoError::ImageProcessing("Failed to create image buffer".to_string())
    })?;

    // Convert to PNG bytes
    let mut png_bytes = Vec::new();
    let mut cursor = Cursor::new(&mut png_bytes);
    
    img_buffer.write_to(&mut cursor, ImageFormat::Png)
        .map_err(|e| {
            error!("Failed to encode image as PNG: {}", e);
            SnaptoError::from(e)
        })?;

    debug!("Successfully converted to PNG ({} bytes)", png_bytes.len());
    Ok(png_bytes)
}

impl Default for ClipboardManager {
    fn default() -> Self {
        Self::new().expect("Failed to create clipboard manager")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clipboard_creation() {
        let result = ClipboardManager::new();
        assert!(result.is_ok());
    }

    #[test]
    fn test_set_and_get_text() {
        let mut manager = ClipboardManager::new().unwrap();
        let test_text = "Hello, clipboard!";
        
        manager.set_text(test_text).unwrap();
        // Note: We can't easily test get_text without platform-specific code
    }
}
