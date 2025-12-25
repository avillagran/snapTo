use crate::error::{ConfigError, Result};
use chrono::Local;
use rand::Rng;
use uuid::Uuid;
use std::sync::atomic::{AtomicU64, Ordering};

/// Contador global para {counter}
static COUNTER: AtomicU64 = AtomicU64::new(1);

/// Parser de templates para nombres de archivo
pub struct TemplateParser {
    date_format: String,
    time_format: String,
}

impl TemplateParser {
    /// Crea un nuevo parser con formatos personalizados
    pub fn new(date_format: String, time_format: String) -> Self {
        Self {
            date_format,
            time_format,
        }
    }

    /// Genera un nombre de archivo basado en un template
    ///
    /// Soporta los siguientes placeholders:
    /// - {date}: Fecha actual con formato configurable
    /// - {time}: Hora actual con formato configurable
    /// - {random:N}: N caracteres aleatorios (alfanuméricos)
    /// - {uuid}: UUID v4
    /// - {counter}: Contador incremental
    ///
    /// # Ejemplos
    /// ```
    /// let parser = TemplateParser::new("%Y%m%d".to_string(), "%H%M%S".to_string());
    /// let filename = parser.generate("screenshot_{date}_{time}", "png");
    /// // Resultado: screenshot_20231225_143022.png
    /// ```
    pub fn generate(&self, template: &str, extension: &str) -> Result<String> {
        let mut result = template.to_string();
        let now = Local::now();

        // Reemplazar {date}
        if result.contains("{date}") {
            let date_str = now.format(&self.date_format).to_string();
            result = result.replace("{date}", &date_str);
        }

        // Reemplazar {time}
        if result.contains("{time}") {
            let time_str = now.format(&self.time_format).to_string();
            result = result.replace("{time}", &time_str);
        }

        // Reemplazar {uuid}
        if result.contains("{uuid}") {
            let uuid = Uuid::new_v4().to_string();
            result = result.replace("{uuid}", &uuid);
        }

        // Reemplazar {counter}
        if result.contains("{counter}") {
            let counter = COUNTER.fetch_add(1, Ordering::SeqCst);
            result = result.replace("{counter}", &counter.to_string());
        }

        // Reemplazar {random:N}
        while let Some(start) = result.find("{random:") {
            if let Some(end) = result[start..].find('}') {
                let end = start + end;
                let num_str = &result[start + 8..end];

                match num_str.parse::<usize>() {
                    Ok(n) if n > 0 && n <= 32 => {
                        let random = generate_random_string(n);
                        result.replace_range(start..=end, &random);
                    }
                    _ => {
                        return Err(ConfigError::Invalid(format!(
                            "Invalid random length in template: {}. Must be between 1 and 32",
                            num_str
                        )).into());
                    }
                }
            } else {
                return Err(ConfigError::Invalid(
                    "Malformed {random:N} placeholder".to_string()
                ).into());
            }
        }

        // Agregar extensión
        let filename = if extension.is_empty() {
            result
        } else {
            format!("{}.{}", result, extension.trim_start_matches('.'))
        };

        Ok(filename)
    }

    /// Resetea el contador (útil para tests)
    #[cfg(test)]
    pub fn reset_counter() {
        COUNTER.store(1, Ordering::SeqCst);
    }
}

impl Default for TemplateParser {
    fn default() -> Self {
        Self {
            date_format: "%Y%m%d".to_string(),
            time_format: "%H%M%S".to_string(),
        }
    }
}

/// Genera una cadena aleatoria de caracteres alfanuméricos
fn generate_random_string(length: usize) -> String {
    const CHARSET: &[u8] = b"abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789";
    let mut rng = rand::thread_rng();

    (0..length)
        .map(|_| {
            let idx = rng.gen_range(0..CHARSET.len());
            CHARSET[idx] as char
        })
        .collect()
}

/// Función de conveniencia para generar nombres de archivo
/// usando la configuración por defecto
pub fn generate_filename(template: &str, extension: &str) -> Result<String> {
    let parser = TemplateParser::default();
    parser.generate(template, extension)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_template() {
        let parser = TemplateParser::default();
        let result = parser.generate("screenshot", "png").unwrap();
        assert_eq!(result, "screenshot.png");
    }

    #[test]
    fn test_date_template() {
        let parser = TemplateParser::new("%Y".to_string(), "%H".to_string());
        let result = parser.generate("file_{date}", "png").unwrap();
        assert!(result.starts_with("file_"));
        assert!(result.ends_with(".png"));
        assert!(result.len() > 10);
    }

    #[test]
    fn test_uuid_template() {
        let parser = TemplateParser::default();
        let result = parser.generate("file_{uuid}", "png").unwrap();
        assert!(result.starts_with("file_"));
        assert!(result.ends_with(".png"));
        // UUID tiene 36 caracteres + "file_" (5) + ".png" (4) = 45
        assert_eq!(result.len(), 45);
    }

    #[test]
    fn test_random_template() {
        let parser = TemplateParser::default();
        let result = parser.generate("file_{random:8}", "png").unwrap();
        assert!(result.starts_with("file_"));
        assert!(result.ends_with(".png"));
        // "file_" (5) + random (8) + ".png" (4) = 17
        assert_eq!(result.len(), 17);
    }

    #[test]
    fn test_counter_template() {
        TemplateParser::reset_counter();
        let parser = TemplateParser::default();

        let result1 = parser.generate("file_{counter}", "png").unwrap();
        let result2 = parser.generate("file_{counter}", "png").unwrap();

        assert_eq!(result1, "file_1.png");
        assert_eq!(result2, "file_2.png");
    }

    #[test]
    fn test_complex_template() {
        let parser = TemplateParser::new("%Y%m%d".to_string(), "%H%M%S".to_string());
        let result = parser.generate("screenshot_{date}_{time}_{random:4}", "png").unwrap();

        assert!(result.starts_with("screenshot_"));
        assert!(result.ends_with(".png"));
        assert!(result.contains("_"));
    }

    #[test]
    fn test_invalid_random_length() {
        let parser = TemplateParser::default();
        let result = parser.generate("file_{random:0}", "png");
        assert!(result.is_err());

        let result = parser.generate("file_{random:100}", "png");
        assert!(result.is_err());
    }

    #[test]
    fn test_malformed_random() {
        let parser = TemplateParser::default();
        let result = parser.generate("file_{random:abc}", "png");
        assert!(result.is_err());
    }

    #[test]
    fn test_extension_with_dot() {
        let parser = TemplateParser::default();
        let result = parser.generate("file", ".png").unwrap();
        assert_eq!(result, "file.png");
    }

    #[test]
    fn test_empty_extension() {
        let parser = TemplateParser::default();
        let result = parser.generate("file", "").unwrap();
        assert_eq!(result, "file");
    }

    #[test]
    fn test_generate_random_string() {
        let s1 = generate_random_string(10);
        let s2 = generate_random_string(10);

        assert_eq!(s1.len(), 10);
        assert_eq!(s2.len(), 10);
        assert_ne!(s1, s2); // Muy improbable que sean iguales
        assert!(s1.chars().all(|c| c.is_alphanumeric()));
    }
}
