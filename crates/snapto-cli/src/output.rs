use colored::*;

/// Print a success message
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an error message
pub fn error(msg: &str) {
    eprintln!("{} {}", "✗".red().bold(), msg.red());
}

/// Print a warning message
pub fn warning(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg.yellow());
}

/// Print an info message
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a step message
pub fn step(msg: &str) {
    println!("{} {}", "→".cyan().bold(), msg);
}

/// Print a header
pub fn header(msg: &str) {
    println!("\n{}", msg.bold().underline());
}

/// Print a key-value pair
pub fn kv(key: &str, value: &str) {
    println!("  {} {}", format!("{}:", key).dimmed(), value);
}

/// Print a list item
pub fn list_item(msg: &str) {
    println!("  • {}", msg);
}

/// Print a separator
pub fn separator() {
    println!("{}", "─".repeat(80).dimmed());
}

/// Print a section header
pub fn section(name: &str) {
    println!("\n{}", format!("[{}]", name).cyan().bold());
}

/// Print an item
pub fn item(msg: &str) {
    println!("  {} {}", "▪".cyan(), msg);
}

/// Format a file size in human-readable format
pub fn format_size(bytes: u64) -> String {
    const KB: u64 = 1024;
    const MB: u64 = KB * 1024;
    const GB: u64 = MB * 1024;

    if bytes >= GB {
        format!("{:.2} GB", bytes as f64 / GB as f64)
    } else if bytes >= MB {
        format!("{:.2} MB", bytes as f64 / MB as f64)
    } else if bytes >= KB {
        format!("{:.2} KB", bytes as f64 / KB as f64)
    } else {
        format!("{} bytes", bytes)
    }
}

/// Format a duration in human-readable format
pub fn format_duration(millis: u64) -> String {
    let seconds = millis / 1000;
    let remaining_millis = millis % 1000;

    if seconds > 0 {
        format!("{}.{:03}s", seconds, remaining_millis)
    } else {
        format!("{}ms", millis)
    }
}
