//! Screenshot capture module for SnapTo
//!
//! Provides functionality to capture screenshots on multiple platforms:
//! - Full screen capture
//! - Region selection capture
//! - Window capture (future)

use image::{DynamicImage, ImageBuffer, Rgba, RgbaImage};
use std::io::Cursor;

#[cfg(target_os = "macos")]
use std::process::Command;

/// Screenshot capture configuration
#[derive(Debug, Clone)]
pub struct ScreenshotConfig {
    /// Output format (png, jpg, webp)
    pub format: ImageFormat,
    /// Quality for lossy formats (1-100)
    pub quality: u8,
    /// Whether to include cursor in capture
    pub include_cursor: bool,
    /// Delay before capture in milliseconds
    pub delay_ms: u64,
}

impl Default for ScreenshotConfig {
    fn default() -> Self {
        Self {
            format: ImageFormat::Png,
            quality: 90,
            include_cursor: false,
            delay_ms: 0,
        }
    }
}

/// Supported image formats
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImageFormat {
    Png,
    Jpeg,
    WebP,
}

impl ImageFormat {
    pub fn extension(&self) -> &'static str {
        match self {
            ImageFormat::Png => "png",
            ImageFormat::Jpeg => "jpg",
            ImageFormat::WebP => "webp",
        }
    }

    pub fn mime_type(&self) -> &'static str {
        match self {
            ImageFormat::Png => "image/png",
            ImageFormat::Jpeg => "image/jpeg",
            ImageFormat::WebP => "image/webp",
        }
    }
}

/// Screen region for capture
#[derive(Debug, Clone, Copy)]
pub struct Region {
    pub x: i32,
    pub y: i32,
    pub width: u32,
    pub height: u32,
}

/// Screenshot capture result
#[derive(Debug)]
pub struct CaptureResult {
    /// Image data as bytes
    pub data: Vec<u8>,
    /// Image width
    pub width: u32,
    /// Image height
    pub height: u32,
    /// Format of the image
    pub format: ImageFormat,
}

/// Screenshot capture manager
pub struct ScreenshotManager {
    config: ScreenshotConfig,
}

impl ScreenshotManager {
    /// Create a new screenshot manager with default config
    pub fn new() -> Self {
        Self {
            config: ScreenshotConfig::default(),
        }
    }

    /// Create a new screenshot manager with custom config
    pub fn with_config(config: ScreenshotConfig) -> Self {
        Self { config }
    }

    /// Capture the entire screen
    #[cfg(target_os = "macos")]
    pub fn capture_fullscreen(&self) -> Result<CaptureResult, ScreenshotError> {
        use std::fs;
        use std::path::PathBuf;

        // Apply delay if configured
        if self.config.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.config.delay_ms));
        }

        // Use macOS screencapture command
        let temp_path = std::env::temp_dir().join(format!(
            "snapto_screenshot_{}.png",
            uuid::Uuid::new_v4()
        ));

        let mut cmd = Command::new("screencapture");
        cmd.arg("-x"); // No sound

        if !self.config.include_cursor {
            cmd.arg("-C"); // Capture cursor (inverted logic in screencapture)
        }

        cmd.arg(&temp_path);

        let output = cmd.output().map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to execute screencapture: {}", e),
        })?;

        if !output.status.success() {
            return Err(ScreenshotError::CaptureError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        // Read the captured image
        let data = fs::read(&temp_path).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to read screenshot: {}", e),
        })?;

        // Clean up temp file
        let _ = fs::remove_file(&temp_path);

        // Get image dimensions
        let img = image::load_from_memory(&data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let (width, height) = img.dimensions();

        // Convert to requested format if needed
        let (data, format) = self.convert_format(img)?;

        Ok(CaptureResult {
            data,
            width,
            height,
            format,
        })
    }

    /// Capture a specific region of the screen
    #[cfg(target_os = "macos")]
    pub fn capture_region(&self, region: Region) -> Result<CaptureResult, ScreenshotError> {
        use std::fs;

        // Apply delay if configured
        if self.config.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.config.delay_ms));
        }

        let temp_path = std::env::temp_dir().join(format!(
            "snapto_screenshot_{}.png",
            uuid::Uuid::new_v4()
        ));

        let rect = format!(
            "{},{},{},{}",
            region.x, region.y, region.width, region.height
        );

        let mut cmd = Command::new("screencapture");
        cmd.arg("-x") // No sound
            .arg("-R")
            .arg(&rect)
            .arg(&temp_path);

        let output = cmd.output().map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to execute screencapture: {}", e),
        })?;

        if !output.status.success() {
            return Err(ScreenshotError::CaptureError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let data = fs::read(&temp_path).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to read screenshot: {}", e),
        })?;

        let _ = fs::remove_file(&temp_path);

        let img = image::load_from_memory(&data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let (width, height) = img.dimensions();
        let (data, format) = self.convert_format(img)?;

        Ok(CaptureResult {
            data,
            width,
            height,
            format,
        })
    }

    /// Interactive region selection (opens selection UI)
    #[cfg(target_os = "macos")]
    pub fn capture_interactive(&self) -> Result<CaptureResult, ScreenshotError> {
        use std::fs;

        let temp_path = std::env::temp_dir().join(format!(
            "snapto_screenshot_{}.png",
            uuid::Uuid::new_v4()
        ));

        // -i for interactive mode, -s for selection
        let mut cmd = Command::new("screencapture");
        cmd.arg("-x") // No sound
            .arg("-i") // Interactive mode
            .arg("-s") // Selection mode
            .arg(&temp_path);

        let output = cmd.output().map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to execute screencapture: {}", e),
        })?;

        // Check if user cancelled (file doesn't exist)
        if !temp_path.exists() {
            return Err(ScreenshotError::Cancelled);
        }

        if !output.status.success() {
            return Err(ScreenshotError::CaptureError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let data = fs::read(&temp_path).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to read screenshot: {}", e),
        })?;

        let _ = fs::remove_file(&temp_path);

        let img = image::load_from_memory(&data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let (width, height) = img.dimensions();
        let (data, format) = self.convert_format(img)?;

        Ok(CaptureResult {
            data,
            width,
            height,
            format,
        })
    }

    /// Capture a specific window (by window ID)
    #[cfg(target_os = "macos")]
    pub fn capture_window(&self, window_id: u32) -> Result<CaptureResult, ScreenshotError> {
        use std::fs;

        let temp_path = std::env::temp_dir().join(format!(
            "snapto_screenshot_{}.png",
            uuid::Uuid::new_v4()
        ));

        let mut cmd = Command::new("screencapture");
        cmd.arg("-x")
            .arg("-l")
            .arg(window_id.to_string())
            .arg(&temp_path);

        let output = cmd.output().map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to execute screencapture: {}", e),
        })?;

        if !output.status.success() {
            return Err(ScreenshotError::CaptureError {
                message: String::from_utf8_lossy(&output.stderr).to_string(),
            });
        }

        let data = fs::read(&temp_path).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to read screenshot: {}", e),
        })?;

        let _ = fs::remove_file(&temp_path);

        let img = image::load_from_memory(&data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let (width, height) = img.dimensions();
        let (data, format) = self.convert_format(img)?;

        Ok(CaptureResult {
            data,
            width,
            height,
            format,
        })
    }

    /// Convert image to the configured format
    fn convert_format(
        &self,
        img: DynamicImage,
    ) -> Result<(Vec<u8>, ImageFormat), ScreenshotError> {
        let mut buffer = Cursor::new(Vec::new());

        match self.config.format {
            ImageFormat::Png => {
                img.write_to(&mut buffer, image::ImageFormat::Png)
                    .map_err(|e| ScreenshotError::ConversionError {
                        message: format!("Failed to encode PNG: {}", e),
                    })?;
            }
            ImageFormat::Jpeg => {
                img.write_to(&mut buffer, image::ImageFormat::Jpeg)
                    .map_err(|e| ScreenshotError::ConversionError {
                        message: format!("Failed to encode JPEG: {}", e),
                    })?;
            }
            ImageFormat::WebP => {
                img.write_to(&mut buffer, image::ImageFormat::WebP)
                    .map_err(|e| ScreenshotError::ConversionError {
                        message: format!("Failed to encode WebP: {}", e),
                    })?;
            }
        }

        Ok((buffer.into_inner(), self.config.format))
    }

    /// List available displays/monitors
    #[cfg(target_os = "macos")]
    pub fn list_displays() -> Result<Vec<DisplayInfo>, ScreenshotError> {
        // On macOS, we can use system_profiler or CGGetActiveDisplayList
        // For now, return a simple implementation
        Ok(vec![DisplayInfo {
            id: 0,
            name: "Main Display".to_string(),
            width: 0,  // Would need Core Graphics to get actual values
            height: 0,
            is_primary: true,
        }])
    }
}

impl Default for ScreenshotManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Display/monitor information
#[derive(Debug, Clone)]
pub struct DisplayInfo {
    pub id: u32,
    pub name: String,
    pub width: u32,
    pub height: u32,
    pub is_primary: bool,
}

/// Screenshot errors
#[derive(Debug, thiserror::Error)]
pub enum ScreenshotError {
    #[error("Screenshot capture failed: {message}")]
    CaptureError { message: String },

    #[error("Image format conversion failed: {message}")]
    ConversionError { message: String },

    #[error("Screenshot cancelled by user")]
    Cancelled,

    #[error("Display not found: {id}")]
    DisplayNotFound { id: u32 },

    #[error("Screenshot not supported on this platform")]
    NotSupported,
}

// Linux implementation using gnome-screenshot or scrot
#[cfg(target_os = "linux")]
impl ScreenshotManager {
    pub fn capture_fullscreen(&self) -> Result<CaptureResult, ScreenshotError> {
        use std::fs;

        if self.config.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.config.delay_ms));
        }

        let temp_path = std::env::temp_dir().join(format!(
            "snapto_screenshot_{}.png",
            uuid::Uuid::new_v4()
        ));

        // Try gnome-screenshot first, fallback to scrot
        let result = Command::new("gnome-screenshot")
            .arg("-f")
            .arg(&temp_path)
            .output();

        let output = match result {
            Ok(o) if o.status.success() => o,
            _ => {
                // Fallback to scrot
                Command::new("scrot")
                    .arg(&temp_path)
                    .output()
                    .map_err(|e| ScreenshotError::CaptureError {
                        message: format!("Failed to capture screenshot: {}", e),
                    })?
            }
        };

        if !temp_path.exists() {
            return Err(ScreenshotError::CaptureError {
                message: "Screenshot file not created".to_string(),
            });
        }

        let data = fs::read(&temp_path).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to read screenshot: {}", e),
        })?;

        let _ = fs::remove_file(&temp_path);

        let img = image::load_from_memory(&data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let (width, height) = img.dimensions();
        let (data, format) = self.convert_format(img)?;

        Ok(CaptureResult {
            data,
            width,
            height,
            format,
        })
    }

    pub fn capture_region(&self, region: Region) -> Result<CaptureResult, ScreenshotError> {
        // Capture full screen and crop
        let full = self.capture_fullscreen()?;
        let img = image::load_from_memory(&full.data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let cropped = img.crop_imm(
            region.x as u32,
            region.y as u32,
            region.width,
            region.height,
        );

        let (data, format) = self.convert_format(cropped)?;

        Ok(CaptureResult {
            data,
            width: region.width,
            height: region.height,
            format,
        })
    }

    pub fn capture_interactive(&self) -> Result<CaptureResult, ScreenshotError> {
        use std::fs;

        let temp_path = std::env::temp_dir().join(format!(
            "snapto_screenshot_{}.png",
            uuid::Uuid::new_v4()
        ));

        // Try gnome-screenshot with area selection
        let result = Command::new("gnome-screenshot")
            .arg("-a") // Area selection
            .arg("-f")
            .arg(&temp_path)
            .output();

        let success = match result {
            Ok(o) => o.status.success() && temp_path.exists(),
            Err(_) => false,
        };

        if !success {
            // Fallback to scrot with selection
            Command::new("scrot")
                .arg("-s") // Selection mode
                .arg(&temp_path)
                .output()
                .map_err(|e| ScreenshotError::CaptureError {
                    message: format!("Failed to capture screenshot: {}", e),
                })?;
        }

        if !temp_path.exists() {
            return Err(ScreenshotError::Cancelled);
        }

        let data = fs::read(&temp_path).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to read screenshot: {}", e),
        })?;

        let _ = fs::remove_file(&temp_path);

        let img = image::load_from_memory(&data).map_err(|e| ScreenshotError::CaptureError {
            message: format!("Failed to load image: {}", e),
        })?;

        let (width, height) = img.dimensions();
        let (data, format) = self.convert_format(img)?;

        Ok(CaptureResult {
            data,
            width,
            height,
            format,
        })
    }

    pub fn capture_window(&self, _window_id: u32) -> Result<CaptureResult, ScreenshotError> {
        Err(ScreenshotError::NotSupported)
    }

    pub fn list_displays() -> Result<Vec<DisplayInfo>, ScreenshotError> {
        Ok(vec![DisplayInfo {
            id: 0,
            name: "Main Display".to_string(),
            width: 0,
            height: 0,
            is_primary: true,
        }])
    }
}

// Windows implementation placeholder
#[cfg(target_os = "windows")]
impl ScreenshotManager {
    pub fn capture_fullscreen(&self) -> Result<CaptureResult, ScreenshotError> {
        Err(ScreenshotError::NotSupported)
    }

    pub fn capture_region(&self, _region: Region) -> Result<CaptureResult, ScreenshotError> {
        Err(ScreenshotError::NotSupported)
    }

    pub fn capture_interactive(&self) -> Result<CaptureResult, ScreenshotError> {
        Err(ScreenshotError::NotSupported)
    }

    pub fn capture_window(&self, _window_id: u32) -> Result<CaptureResult, ScreenshotError> {
        Err(ScreenshotError::NotSupported)
    }

    pub fn list_displays() -> Result<Vec<DisplayInfo>, ScreenshotError> {
        Ok(vec![])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_image_format_extension() {
        assert_eq!(ImageFormat::Png.extension(), "png");
        assert_eq!(ImageFormat::Jpeg.extension(), "jpg");
        assert_eq!(ImageFormat::WebP.extension(), "webp");
    }

    #[test]
    fn test_default_config() {
        let config = ScreenshotConfig::default();
        assert_eq!(config.format, ImageFormat::Png);
        assert_eq!(config.quality, 90);
        assert!(!config.include_cursor);
        assert_eq!(config.delay_ms, 0);
    }
}
