use anyhow::Result;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use snapto_core::{
    ClipboardManager, ClipboardCopyMode, Config, HistoryEntry, HistoryManager, HistoryMode,
    KeychainManager, LocalUploader, SftpUploader, SshUploader, UploadConfig, Uploader,
};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Home,
    History,
    Settings,
    Upload,
}

impl Screen {
    pub fn next(&self) -> Self {
        match self {
            Screen::Home => Screen::History,
            Screen::History => Screen::Settings,
            Screen::Settings => Screen::Upload,
            Screen::Upload => Screen::Home,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            Screen::Home => Screen::Upload,
            Screen::History => Screen::Home,
            Screen::Settings => Screen::History,
            Screen::Upload => Screen::Settings,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            Screen::Home => "Home",
            Screen::History => "History",
            Screen::Settings => "Settings",
            Screen::Upload => "Upload",
        }
    }
}

pub struct App {
    pub screen: Screen,
    pub config: Config,
    pub history: Vec<HistoryEntry>,
    pub should_quit: bool,
    pub status_message: Option<String>,
    pub clipboard_has_image: bool,
    pub history_manager: Option<HistoryManager>,
    pub clipboard_manager: Option<ClipboardManager>,
    // History screen state
    pub history_selected: usize,
    pub show_reupload_menu: bool,
    pub reupload_selected: usize,
    pub available_uploaders: Vec<(String, UploadConfig)>,
    // Settings screen state
    pub settings_section: SettingsSection,
    pub settings_selected: usize,
    pub settings_editing: bool,
    pub edit_buffer: String,
    pub edit_cursor: usize,
    // Uploader editing state
    pub uploader_names: Vec<String>,
    pub uploader_selected: usize,
    pub uploader_field_selected: usize,
    pub uploader_editing: bool,
    pub show_add_uploader: bool,
    pub new_uploader_name: String,
    pub new_uploader_type: usize, // 0=local, 1=sftp, 2=ssh
    // Password prompt state
    pub show_password_prompt: bool,
    pub password_buffer: String,
    pub pending_reupload: Option<PendingReupload>,
    pub keychain_manager: Option<KeychainManager>,
    // Upload screen state
    pub upload_progress: Option<f64>,
    pub upload_result: Option<UploadStatus>,
}

#[derive(Debug, Clone)]
pub struct PendingReupload {
    pub entry: HistoryEntry,
    pub uploader_name: String,
    pub uploader_config: UploadConfig,
    pub file_data: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SettingsSection {
    General,
    Naming,
    History,
    Uploads,
    Security,
}

impl SettingsSection {
    pub fn next(&self) -> Self {
        match self {
            SettingsSection::General => SettingsSection::Naming,
            SettingsSection::Naming => SettingsSection::History,
            SettingsSection::History => SettingsSection::Uploads,
            SettingsSection::Uploads => SettingsSection::Security,
            SettingsSection::Security => SettingsSection::General,
        }
    }

    pub fn prev(&self) -> Self {
        match self {
            SettingsSection::General => SettingsSection::Security,
            SettingsSection::Naming => SettingsSection::General,
            SettingsSection::History => SettingsSection::Naming,
            SettingsSection::Uploads => SettingsSection::History,
            SettingsSection::Security => SettingsSection::Uploads,
        }
    }

    pub fn name(&self) -> &str {
        match self {
            SettingsSection::General => "General",
            SettingsSection::Naming => "Naming",
            SettingsSection::History => "History",
            SettingsSection::Uploads => "Uploads",
            SettingsSection::Security => "Security",
        }
    }

    pub fn field_count(&self) -> usize {
        match self {
            SettingsSection::General => 5,  // local_save_dir, copy_url, clipboard_mode, notifications, default_uploader
            SettingsSection::Naming => 4,   // template, date_format, time_format, extension
            SettingsSection::History => 4,  // enabled, mode, retention_days, max_entries
            SettingsSection::Uploads => 0,  // Read-only for now (complex editing)
            SettingsSection::Security => 2, // use_keychain, encrypt_credentials
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FieldType {
    Text,
    Bool,
    Number,
    Enum,
    Password,
}

#[derive(Debug, Clone)]
pub struct SettingsField {
    pub name: &'static str,
    pub label: &'static str,
    pub field_type: FieldType,
    pub enum_options: Option<Vec<&'static str>>,
}

impl SettingsField {
    pub const fn text(name: &'static str, label: &'static str) -> Self {
        Self { name, label, field_type: FieldType::Text, enum_options: None }
    }
    pub const fn bool(name: &'static str, label: &'static str) -> Self {
        Self { name, label, field_type: FieldType::Bool, enum_options: None }
    }
    pub const fn number(name: &'static str, label: &'static str) -> Self {
        Self { name, label, field_type: FieldType::Number, enum_options: None }
    }
    pub const fn password(name: &'static str, label: &'static str) -> Self {
        Self { name, label, field_type: FieldType::Password, enum_options: None }
    }
    pub fn enumeration(name: &'static str, label: &'static str, options: Vec<&'static str>) -> Self {
        Self { name, label, field_type: FieldType::Enum, enum_options: Some(options) }
    }
}

pub fn get_section_fields(section: SettingsSection) -> Vec<SettingsField> {
    match section {
        SettingsSection::General => vec![
            SettingsField::text("local_save_dir", "Local Save Directory"),
            SettingsField::bool("copy_url_to_clipboard", "Copy URL to Clipboard"),
            SettingsField::enumeration("clipboard_copy_mode", "Clipboard Copy Mode", vec!["auto", "url", "path"]),
            SettingsField::bool("show_notifications", "Show Notifications"),
            SettingsField::text("default_uploader", "Default Uploader"),
        ],
        SettingsSection::Naming => vec![
            SettingsField::text("template", "Template"),
            SettingsField::text("date_format", "Date Format"),
            SettingsField::text("time_format", "Time Format"),
            SettingsField::text("default_extension", "Default Extension"),
        ],
        SettingsSection::History => vec![
            SettingsField::bool("enabled", "Enabled"),
            SettingsField::enumeration("mode", "Mode", vec!["metadata", "thumbnails", "full"]),
            SettingsField::number("retention_days", "Retention Days"),
            SettingsField::number("max_entries", "Max Entries"),
        ],
        SettingsSection::Uploads => vec![],  // Handled separately
        SettingsSection::Security => vec![
            SettingsField::bool("use_system_keychain", "Use System Keychain"),
            SettingsField::bool("encrypt_credentials", "Encrypt Credentials"),
        ],
    }
}

pub fn get_uploader_fields(uploader_type: &str) -> Vec<SettingsField> {
    let mut fields = vec![
        SettingsField::bool("enabled", "Enabled"),
        SettingsField::enumeration("type", "Type", vec!["local", "sftp", "ssh"]),
    ];

    match uploader_type {
        "local" => {
            fields.push(SettingsField::text("local_path", "Local Path"));
        }
        "sftp" | "ssh" => {
            fields.push(SettingsField::text("host", "Host"));
            fields.push(SettingsField::number("port", "Port"));
            fields.push(SettingsField::text("username", "Username"));
            fields.push(SettingsField::text("remote_path", "Remote Path"));
            fields.push(SettingsField::text("base_url", "Base URL"));
            fields.push(SettingsField::bool("use_key_auth", "Use Key Auth"));
            fields.push(SettingsField::text("key_path", "Key Path"));
            // Always show password field for SSH/SFTP - user can set it for password auth
            fields.push(SettingsField::password("password", "Password"));
            fields.push(SettingsField::number("timeout", "Timeout (s)"));
        }
        _ => {}
    }

    fields
}

#[derive(Debug, Clone)]
pub enum UploadStatus {
    InProgress,
    Success { url: String },
    Error { message: String },
}

impl App {
    pub fn new() -> Result<Self> {
        let config = Config::load().unwrap_or_default();

        // Try to load history
        let (history_manager, history) = if config.history.enabled {
            match HistoryManager::new(config.history.clone()) {
                Ok(manager) => {
                    let entries = manager.get_recent(100).unwrap_or_default();
                    (Some(manager), entries)
                }
                Err(_) => (None, vec![]),
            }
        } else {
            (None, vec![])
        };

        // Try to create clipboard manager
        let mut clipboard_manager = ClipboardManager::new().ok();
        let clipboard_has_image = clipboard_manager
            .as_mut()
            .map(|c| c.has_image())
            .unwrap_or(false);

        // Build list of available uploaders
        let available_uploaders: Vec<(String, UploadConfig)> = config
            .uploads
            .iter()
            .filter(|(_, cfg)| cfg.enabled)
            .map(|(name, cfg)| (name.clone(), cfg.clone()))
            .collect();

        // Get uploader names for settings editing
        let uploader_names: Vec<String> = config.uploads.keys().cloned().collect();

        // Initialize keychain manager
        let keychain_manager = Some(KeychainManager::new(&config.security));

        Ok(Self {
            screen: Screen::Home,
            config,
            history,
            should_quit: false,
            status_message: None,
            clipboard_has_image,
            history_manager,
            clipboard_manager,
            history_selected: 0,
            show_reupload_menu: false,
            reupload_selected: 0,
            available_uploaders,
            settings_section: SettingsSection::General,
            settings_selected: 0,
            settings_editing: false,
            edit_buffer: String::new(),
            edit_cursor: 0,
            uploader_names,
            uploader_selected: 0,
            uploader_field_selected: 0,
            uploader_editing: false,
            show_add_uploader: false,
            new_uploader_name: String::new(),
            new_uploader_type: 0,
            show_password_prompt: false,
            password_buffer: String::new(),
            pending_reupload: None,
            keychain_manager,
            upload_progress: None,
            upload_result: None,
        })
    }

    pub fn handle_key(&mut self, key: KeyEvent) -> Result<()> {
        // Global key bindings
        match key.code {
            KeyCode::Char('q') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.should_quit = true;
                return Ok(());
            }
            KeyCode::Tab if key.modifiers.contains(KeyModifiers::SHIFT) => {
                self.screen = self.screen.prev();
                return Ok(());
            }
            KeyCode::Tab => {
                self.screen = self.screen.next();
                return Ok(());
            }
            KeyCode::Char('u') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.start_upload();
                return Ok(());
            }
            _ => {}
        }

        // Screen-specific key bindings
        match self.screen {
            Screen::Home => self.handle_home_key(key),
            Screen::History => self.handle_history_key(key),
            Screen::Settings => self.handle_settings_key(key),
            Screen::Upload => self.handle_upload_key(key),
        }
    }

    fn handle_home_key(&mut self, key: KeyEvent) -> Result<()> {
        match key.code {
            KeyCode::Char('u') => {
                self.start_upload();
            }
            KeyCode::Char('r') => {
                self.refresh_clipboard_status();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_history_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle password prompt if open
        if self.show_password_prompt {
            match key.code {
                KeyCode::Esc => {
                    self.show_password_prompt = false;
                    self.password_buffer.clear();
                    self.pending_reupload = None;
                    self.status_message = Some("Upload cancelled".to_string());
                }
                KeyCode::Enter => {
                    self.execute_reupload_with_password();
                }
                KeyCode::Backspace => {
                    self.password_buffer.pop();
                }
                KeyCode::Char(c) => {
                    self.password_buffer.push(c);
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle reupload menu if open
        if self.show_reupload_menu {
            match key.code {
                KeyCode::Up | KeyCode::Char('k') => {
                    if self.reupload_selected > 0 {
                        self.reupload_selected -= 1;
                    }
                }
                KeyCode::Down | KeyCode::Char('j') => {
                    if self.reupload_selected < self.available_uploaders.len().saturating_sub(1) {
                        self.reupload_selected += 1;
                    }
                }
                KeyCode::Enter => {
                    self.perform_reupload();
                }
                KeyCode::Esc | KeyCode::Char('q') => {
                    self.show_reupload_menu = false;
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal history navigation
        match key.code {
            KeyCode::Up | KeyCode::Char('k') => {
                if self.history_selected > 0 {
                    self.history_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.history_selected < self.history.len().saturating_sub(1) {
                    self.history_selected += 1;
                }
            }
            KeyCode::Enter => {
                self.copy_selected_url();
            }
            KeyCode::Char('d') => {
                self.delete_selected_entry();
            }
            KeyCode::Char('c') => {
                self.copy_selected_url();
            }
            KeyCode::Char('r') => {
                self.show_reupload_selector();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_settings_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle Uploads section separately
        if self.settings_section == SettingsSection::Uploads {
            return self.handle_uploads_key(key);
        }

        let fields = get_section_fields(self.settings_section);
        let field_count = fields.len();

        // If currently editing
        if self.settings_editing {
            match key.code {
                KeyCode::Esc => {
                    // Cancel editing
                    self.settings_editing = false;
                    self.edit_buffer.clear();
                    self.edit_cursor = 0;
                }
                KeyCode::Enter => {
                    // Save the edited value
                    self.apply_edit();
                    self.settings_editing = false;
                    self.edit_buffer.clear();
                    self.edit_cursor = 0;
                }
                KeyCode::Backspace => {
                    if self.edit_cursor > 0 {
                        self.edit_cursor -= 1;
                        self.edit_buffer.remove(self.edit_cursor);
                    }
                }
                KeyCode::Delete => {
                    if self.edit_cursor < self.edit_buffer.len() {
                        self.edit_buffer.remove(self.edit_cursor);
                    }
                }
                KeyCode::Left => {
                    if self.edit_cursor > 0 {
                        self.edit_cursor -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.edit_cursor < self.edit_buffer.len() {
                        self.edit_cursor += 1;
                    }
                }
                KeyCode::Home => {
                    self.edit_cursor = 0;
                }
                KeyCode::End => {
                    self.edit_cursor = self.edit_buffer.len();
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.insert(self.edit_cursor, c);
                    self.edit_cursor += 1;
                }
                _ => {}
            }
            return Ok(());
        }

        // Not editing - normal navigation
        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.settings_section = self.settings_section.prev();
                self.settings_selected = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.settings_section = self.settings_section.next();
                self.settings_selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.settings_selected > 0 {
                    self.settings_selected -= 1;
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if field_count > 0 && self.settings_selected < field_count - 1 {
                    self.settings_selected += 1;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if field_count > 0 && self.settings_selected < field_count {
                    let field = &fields[self.settings_selected];
                    match field.field_type {
                        FieldType::Bool => {
                            // Toggle boolean immediately
                            self.toggle_bool_field(field.name);
                        }
                        FieldType::Enum => {
                            // Cycle through enum options
                            self.cycle_enum_field(field.name, &field.enum_options);
                        }
                        FieldType::Text | FieldType::Number | FieldType::Password => {
                            // Start editing
                            self.edit_buffer = self.get_field_value(field.name);
                            self.edit_cursor = self.edit_buffer.len();
                            self.settings_editing = true;
                        }
                    }
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_config();
            }
            _ => {}
        }
        Ok(())
    }

    fn handle_uploads_key(&mut self, key: KeyEvent) -> Result<()> {
        // Handle add uploader popup
        if self.show_add_uploader {
            match key.code {
                KeyCode::Esc => {
                    self.show_add_uploader = false;
                    self.new_uploader_name.clear();
                    self.new_uploader_type = 0;
                }
                KeyCode::Enter => {
                    if !self.new_uploader_name.is_empty() {
                        self.add_new_uploader();
                    }
                }
                KeyCode::Tab => {
                    self.new_uploader_type = (self.new_uploader_type + 1) % 3;
                }
                KeyCode::Backspace => {
                    self.new_uploader_name.pop();
                }
                KeyCode::Char(c) => {
                    self.new_uploader_name.push(c);
                }
                _ => {}
            }
            return Ok(());
        }

        // Handle editing uploader field
        if self.uploader_editing {
            match key.code {
                KeyCode::Esc => {
                    self.uploader_editing = false;
                    self.edit_buffer.clear();
                    self.edit_cursor = 0;
                }
                KeyCode::Enter => {
                    self.apply_uploader_edit();
                    self.uploader_editing = false;
                    self.edit_buffer.clear();
                    self.edit_cursor = 0;
                }
                KeyCode::Backspace => {
                    if self.edit_cursor > 0 {
                        self.edit_cursor -= 1;
                        self.edit_buffer.remove(self.edit_cursor);
                    }
                }
                KeyCode::Delete => {
                    if self.edit_cursor < self.edit_buffer.len() {
                        self.edit_buffer.remove(self.edit_cursor);
                    }
                }
                KeyCode::Left => {
                    if self.edit_cursor > 0 {
                        self.edit_cursor -= 1;
                    }
                }
                KeyCode::Right => {
                    if self.edit_cursor < self.edit_buffer.len() {
                        self.edit_cursor += 1;
                    }
                }
                KeyCode::Home => {
                    self.edit_cursor = 0;
                }
                KeyCode::End => {
                    self.edit_cursor = self.edit_buffer.len();
                }
                KeyCode::Char(c) => {
                    self.edit_buffer.insert(self.edit_cursor, c);
                    self.edit_cursor += 1;
                }
                _ => {}
            }
            return Ok(());
        }

        // Normal navigation in uploads
        let uploader_count = self.uploader_names.len();
        let fields = self.get_current_uploader_fields();
        let field_count = fields.len();

        match key.code {
            KeyCode::Left | KeyCode::Char('h') => {
                self.settings_section = self.settings_section.prev();
                self.settings_selected = 0;
            }
            KeyCode::Right | KeyCode::Char('l') => {
                self.settings_section = self.settings_section.next();
                self.settings_selected = 0;
            }
            KeyCode::Up | KeyCode::Char('k') => {
                if self.uploader_field_selected > 0 {
                    self.uploader_field_selected -= 1;
                } else if self.uploader_selected > 0 {
                    self.uploader_selected -= 1;
                    // Move to last field of previous uploader
                    let prev_fields = self.get_current_uploader_fields();
                    self.uploader_field_selected = prev_fields.len().saturating_sub(1);
                }
            }
            KeyCode::Down | KeyCode::Char('j') => {
                if self.uploader_field_selected < field_count.saturating_sub(1) {
                    self.uploader_field_selected += 1;
                } else if self.uploader_selected < uploader_count.saturating_sub(1) {
                    self.uploader_selected += 1;
                    self.uploader_field_selected = 0;
                }
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                if uploader_count > 0 && self.uploader_field_selected < field_count {
                    let field = &fields[self.uploader_field_selected];
                    match field.field_type {
                        FieldType::Bool => {
                            self.toggle_uploader_bool(field.name);
                        }
                        FieldType::Enum => {
                            self.cycle_uploader_enum(field.name, &field.enum_options);
                        }
                        FieldType::Text | FieldType::Number => {
                            self.edit_buffer = self.get_uploader_field_value(field.name);
                            self.edit_cursor = self.edit_buffer.len();
                            self.uploader_editing = true;
                        }
                        FieldType::Password => {
                            // For password, start with empty buffer (don't show existing password)
                            self.edit_buffer = String::new();
                            self.edit_cursor = 0;
                            self.uploader_editing = true;
                        }
                    }
                }
            }
            KeyCode::Char('a') => {
                // Add new uploader
                self.show_add_uploader = true;
                self.new_uploader_name.clear();
                self.new_uploader_type = 0;
            }
            KeyCode::Char('d') => {
                // Delete current uploader
                if uploader_count > 0 {
                    self.delete_current_uploader();
                }
            }
            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                self.save_config();
            }
            _ => {}
        }
        Ok(())
    }

    /// Get the fields for the current uploader
    fn get_current_uploader_fields(&self) -> Vec<SettingsField> {
        if self.uploader_selected >= self.uploader_names.len() {
            return vec![];
        }
        let name = &self.uploader_names[self.uploader_selected];
        let Some(uploader) = self.config.uploads.get(name) else {
            return vec![];
        };
        get_uploader_fields(&uploader.uploader_type)
    }

    /// Get the fields for a specific uploader by name
    fn get_uploader_fields_for(&self, uploader_name: &str) -> Vec<SettingsField> {
        let Some(uploader) = self.config.uploads.get(uploader_name) else {
            return vec![];
        };
        get_uploader_fields(&uploader.uploader_type)
    }

    fn get_uploader_field_value(&self, field_name: &str) -> String {
        if self.uploader_selected >= self.uploader_names.len() {
            return String::new();
        }
        let name = &self.uploader_names[self.uploader_selected];
        let Some(uploader) = self.config.uploads.get(name) else {
            return String::new();
        };

        match field_name {
            "host" => uploader.host.clone().unwrap_or_default(),
            "port" => uploader.port.map(|p| p.to_string()).unwrap_or_else(|| "22".to_string()),
            "username" => uploader.username.clone().unwrap_or_default(),
            "remote_path" => uploader.remote_path.clone().unwrap_or_default(),
            "base_url" => uploader.base_url.clone().unwrap_or_default(),
            "local_path" => uploader.local_path.clone().unwrap_or_default(),
            "key_path" => uploader.key_path.clone().unwrap_or_default(),
            "timeout" => uploader.timeout.map(|t| t.to_string()).unwrap_or_else(|| "30".to_string()),
            "password" => {
                // Check if password is stored in keychain
                let keychain_key = format!("ssh_password_{}", name);
                if let Some(ref km) = self.keychain_manager {
                    if km.get(&keychain_key).ok().flatten().is_some() {
                        return "••••••••".to_string(); // Password is set
                    }
                }
                String::new() // No password set
            }
            _ => String::new(),
        }
    }

    fn apply_uploader_edit(&mut self) {
        if self.uploader_selected >= self.uploader_names.len() {
            return;
        }
        let name = self.uploader_names[self.uploader_selected].clone();
        let fields = self.get_uploader_fields_for(&name);

        if self.uploader_field_selected >= fields.len() {
            return;
        }
        let field = &fields[self.uploader_field_selected];
        let value = self.edit_buffer.clone();

        // Handle password field specially - store in keychain
        if field.name == "password" {
            if !value.is_empty() {
                let keychain_key = format!("ssh_password_{}", name);
                match &self.keychain_manager {
                    Some(km) => {
                        match km.set(&keychain_key, &value) {
                            Ok(_) => {
                                self.status_message = Some(format!("Password saved securely for {}", name));
                            }
                            Err(e) => {
                                self.status_message = Some(format!("Failed to save password: {}", e));
                            }
                        }
                    }
                    None => {
                        self.status_message = Some("Error: Keychain manager not initialized".to_string());
                    }
                }
            } else {
                self.status_message = Some("Password cannot be empty".to_string());
            }
            return;
        }

        if let Some(uploader) = self.config.uploads.get_mut(&name) {
            match field.name {
                "host" => uploader.host = if value.is_empty() { None } else { Some(value) },
                "port" => uploader.port = value.parse().ok(),
                "username" => uploader.username = if value.is_empty() { None } else { Some(value) },
                "remote_path" => uploader.remote_path = if value.is_empty() { None } else { Some(value) },
                "base_url" => uploader.base_url = if value.is_empty() { None } else { Some(value) },
                "local_path" => uploader.local_path = if value.is_empty() { None } else { Some(value) },
                "key_path" => uploader.key_path = if value.is_empty() { None } else { Some(value) },
                "timeout" => uploader.timeout = value.parse().ok(),
                _ => {}
            }
        }
        self.status_message = Some("Value updated (Ctrl+S to save)".to_string());
    }

    fn toggle_uploader_bool(&mut self, field_name: &str) {
        if self.uploader_selected >= self.uploader_names.len() {
            return;
        }
        let name = self.uploader_names[self.uploader_selected].clone();

        if let Some(uploader) = self.config.uploads.get_mut(&name) {
            match field_name {
                "enabled" => {
                    uploader.enabled = !uploader.enabled;
                    // Update available_uploaders list
                    self.refresh_available_uploaders();
                }
                "use_key_auth" => {
                    uploader.use_key_auth = Some(!uploader.use_key_auth.unwrap_or(true));
                }
                _ => {}
            }
        }
        self.status_message = Some("Value toggled (Ctrl+S to save)".to_string());
    }

    fn cycle_uploader_enum(&mut self, field_name: &str, options: &Option<Vec<&'static str>>) {
        let Some(opts) = options else { return };
        if opts.is_empty() { return; }

        if self.uploader_selected >= self.uploader_names.len() {
            return;
        }
        let name = self.uploader_names[self.uploader_selected].clone();

        if let Some(uploader) = self.config.uploads.get_mut(&name) {
            if field_name == "type" {
                let current = &uploader.uploader_type;
                let idx = opts.iter().position(|&o| o == current).unwrap_or(0);
                let next_idx = (idx + 1) % opts.len();
                uploader.uploader_type = opts[next_idx].to_string();
                // Reset field selection since fields change based on type
                self.uploader_field_selected = 0;
            }
        }
        self.status_message = Some("Value changed (Ctrl+S to save)".to_string());
    }

    fn add_new_uploader(&mut self) {
        let name = self.new_uploader_name.clone();
        let uploader_type = match self.new_uploader_type {
            0 => "local",
            1 => "sftp",
            _ => "ssh",
        };

        let new_config = UploadConfig {
            uploader_type: uploader_type.to_string(),
            enabled: true,
            host: if uploader_type != "local" { Some("example.com".to_string()) } else { None },
            port: if uploader_type != "local" { Some(22) } else { None },
            username: if uploader_type != "local" { Some("user".to_string()) } else { None },
            remote_path: if uploader_type != "local" { Some("/path/to/uploads".to_string()) } else { None },
            base_url: None,
            local_path: if uploader_type == "local" { Some("~/Pictures/Screenshots".to_string()) } else { None },
            use_key_auth: if uploader_type != "local" { Some(true) } else { None },
            key_path: if uploader_type != "local" { Some("~/.ssh/id_rsa".to_string()) } else { None },
            timeout: if uploader_type != "local" { Some(30) } else { None },
        };

        self.config.uploads.insert(name.clone(), new_config);
        self.uploader_names.push(name);
        self.uploader_selected = self.uploader_names.len() - 1;
        self.uploader_field_selected = 0;
        self.show_add_uploader = false;
        self.new_uploader_name.clear();
        self.refresh_available_uploaders();
        self.status_message = Some("Uploader added (Ctrl+S to save)".to_string());
    }

    fn delete_current_uploader(&mut self) {
        if self.uploader_selected >= self.uploader_names.len() {
            return;
        }
        let name = self.uploader_names.remove(self.uploader_selected);
        self.config.uploads.remove(&name);

        if self.uploader_selected >= self.uploader_names.len() && self.uploader_selected > 0 {
            self.uploader_selected -= 1;
        }
        self.uploader_field_selected = 0;
        self.refresh_available_uploaders();
        self.status_message = Some(format!("Uploader '{}' deleted (Ctrl+S to save)", name));
    }

    fn refresh_available_uploaders(&mut self) {
        self.available_uploaders = self.config.uploads
            .iter()
            .filter(|(_, cfg)| cfg.enabled)
            .map(|(name, cfg)| (name.clone(), cfg.clone()))
            .collect();
    }

    fn get_field_value(&self, field_name: &str) -> String {
        match self.settings_section {
            SettingsSection::General => match field_name {
                "local_save_dir" => self.config.general.local_save_dir.clone().unwrap_or_default(),
                "default_uploader" => self.config.general.default_uploader.clone(),
                _ => String::new(),
            },
            SettingsSection::Naming => match field_name {
                "template" => self.config.naming.template.clone(),
                "date_format" => self.config.naming.date_format.clone(),
                "time_format" => self.config.naming.time_format.clone(),
                "default_extension" => self.config.naming.default_extension.clone(),
                _ => String::new(),
            },
            SettingsSection::History => match field_name {
                "retention_days" => self.config.history.retention_days.to_string(),
                "max_entries" => self.config.history.max_entries.to_string(),
                _ => String::new(),
            },
            _ => String::new(),
        }
    }

    fn apply_edit(&mut self) {
        let fields = get_section_fields(self.settings_section);
        if self.settings_selected >= fields.len() {
            return;
        }
        let field = &fields[self.settings_selected];
        let value = self.edit_buffer.clone();

        match self.settings_section {
            SettingsSection::General => match field.name {
                "local_save_dir" => {
                    self.config.general.local_save_dir = if value.is_empty() { None } else { Some(value) };
                }
                "default_uploader" => {
                    self.config.general.default_uploader = value;
                }
                _ => {}
            },
            SettingsSection::Naming => match field.name {
                "template" => self.config.naming.template = value,
                "date_format" => self.config.naming.date_format = value,
                "time_format" => self.config.naming.time_format = value,
                "default_extension" => self.config.naming.default_extension = value,
                _ => {}
            },
            SettingsSection::History => match field.name {
                "retention_days" => {
                    if let Ok(n) = value.parse() {
                        self.config.history.retention_days = n;
                    }
                }
                "max_entries" => {
                    if let Ok(n) = value.parse() {
                        self.config.history.max_entries = n;
                    }
                }
                _ => {}
            },
            _ => {}
        }
        self.status_message = Some("Value updated (Ctrl+S to save)".to_string());
    }

    fn toggle_bool_field(&mut self, field_name: &str) {
        match self.settings_section {
            SettingsSection::General => match field_name {
                "copy_url_to_clipboard" => {
                    self.config.general.copy_url_to_clipboard = !self.config.general.copy_url_to_clipboard;
                }
                "show_notifications" => {
                    self.config.general.show_notifications = !self.config.general.show_notifications;
                }
                _ => {}
            },
            SettingsSection::History => match field_name {
                "enabled" => {
                    self.config.history.enabled = !self.config.history.enabled;
                }
                _ => {}
            },
            SettingsSection::Security => match field_name {
                "use_system_keychain" => {
                    self.config.security.use_system_keychain = !self.config.security.use_system_keychain;
                }
                "encrypt_credentials" => {
                    self.config.security.encrypt_credentials = !self.config.security.encrypt_credentials;
                }
                _ => {}
            },
            _ => {}
        }
        self.status_message = Some("Value toggled (Ctrl+S to save)".to_string());
    }

    fn cycle_enum_field(&mut self, field_name: &str, options: &Option<Vec<&'static str>>) {
        let Some(opts) = options else { return };
        if opts.is_empty() { return; }

        match self.settings_section {
            SettingsSection::General => match field_name {
                "clipboard_copy_mode" => {
                    let current = match self.config.general.clipboard_copy_mode {
                        ClipboardCopyMode::Auto => "auto",
                        ClipboardCopyMode::Url => "url",
                        ClipboardCopyMode::Path => "path",
                    };
                    let idx = opts.iter().position(|&o| o == current).unwrap_or(0);
                    let next_idx = (idx + 1) % opts.len();
                    self.config.general.clipboard_copy_mode = match opts[next_idx] {
                        "url" => ClipboardCopyMode::Url,
                        "path" => ClipboardCopyMode::Path,
                        _ => ClipboardCopyMode::Auto,
                    };
                }
                _ => {}
            },
            SettingsSection::History => match field_name {
                "mode" => {
                    let current = match self.config.history.mode {
                        HistoryMode::Metadata => "metadata",
                        HistoryMode::Thumbnails => "thumbnails",
                        HistoryMode::Full => "full",
                    };
                    let idx = opts.iter().position(|&o| o == current).unwrap_or(0);
                    let next_idx = (idx + 1) % opts.len();
                    self.config.history.mode = match opts[next_idx] {
                        "thumbnails" => HistoryMode::Thumbnails,
                        "full" => HistoryMode::Full,
                        _ => HistoryMode::Metadata,
                    };
                }
                _ => {}
            },
            _ => {}
        }
        self.status_message = Some("Value changed (Ctrl+S to save)".to_string());
    }

    fn handle_upload_key(&mut self, _key: KeyEvent) -> Result<()> {
        // Upload screen is mostly passive
        Ok(())
    }

    fn start_upload(&mut self) {
        self.screen = Screen::Upload;
        self.upload_progress = Some(0.0);
        self.upload_result = None;
        self.status_message = Some("Starting upload...".to_string());

        // Simulate upload (in a real app, this would be async)
        // For now, just set to success
        self.upload_progress = Some(100.0);
        self.upload_result = Some(UploadStatus::Success {
            url: "https://example.com/screenshot.png".to_string(),
        });
        self.status_message = Some("Upload completed!".to_string());
    }

    fn refresh_clipboard_status(&mut self) {
        if let Some(ref mut clipboard) = self.clipboard_manager {
            self.clipboard_has_image = clipboard.has_image();
            self.status_message = Some(if self.clipboard_has_image {
                "Clipboard has image".to_string()
            } else {
                "No image in clipboard".to_string()
            });
        }
    }

    fn copy_selected_url(&mut self) {
        if let Some(entry) = self.history.get(self.history_selected) {
            if let Some(ref url) = entry.url {
                if let Some(ref mut clipboard) = self.clipboard_manager {
                    if clipboard.set_text(url).is_ok() {
                        self.status_message = Some(format!("Copied URL to clipboard: {}", url));
                    } else {
                        self.status_message = Some("Failed to copy URL".to_string());
                    }
                }
            } else {
                self.status_message = Some("No URL available for this entry".to_string());
            }
        }
    }

    fn delete_selected_entry(&mut self) {
        if let Some(entry) = self.history.get(self.history_selected) {
            let id = entry.id;
            if let Some(ref manager) = self.history_manager {
                if manager.delete(id).is_ok() {
                    self.history.remove(self.history_selected);
                    if self.history_selected >= self.history.len() && self.history_selected > 0 {
                        self.history_selected -= 1;
                    }
                    self.status_message = Some("Entry deleted".to_string());
                } else {
                    self.status_message = Some("Failed to delete entry".to_string());
                }
            }
        }
    }

    fn save_config(&mut self) {
        if let Err(e) = self.config.save() {
            self.status_message = Some(format!("Failed to save config: {}", e));
        } else {
            self.status_message = Some("Configuration saved".to_string());
        }
    }

    pub fn get_last_upload(&self) -> Option<&HistoryEntry> {
        self.history.first()
    }

    fn show_reupload_selector(&mut self) {
        if self.history.is_empty() {
            self.status_message = Some("No entry selected".to_string());
            return;
        }

        if self.available_uploaders.is_empty() {
            self.status_message = Some("No enabled uploaders available".to_string());
            return;
        }

        // Check if the selected entry has a local copy or thumbnail we can reupload
        if let Some(entry) = self.history.get(self.history_selected) {
            if entry.local_copy_path.is_none() && entry.thumbnail_path.is_none() {
                self.status_message = Some("No local copy available for re-upload".to_string());
                return;
            }
        }

        self.show_reupload_menu = true;
        self.reupload_selected = 0;
    }

    fn perform_reupload(&mut self) {
        self.show_reupload_menu = false;

        let entry = match self.history.get(self.history_selected) {
            Some(e) => e.clone(),
            None => {
                self.status_message = Some("No entry selected".to_string());
                return;
            }
        };

        let (uploader_name, uploader_config) = match self.available_uploaders.get(self.reupload_selected) {
            Some(u) => u.clone(),
            None => {
                self.status_message = Some("No uploader selected".to_string());
                return;
            }
        };

        // Get the local file path
        let file_path = entry.local_copy_path.clone().or(entry.thumbnail_path.clone());

        let Some(path_str) = file_path else {
            self.status_message = Some("No local file available for re-upload".to_string());
            return;
        };

        // Expand ~ to home directory
        let expanded_path = if path_str.starts_with("~/") {
            if let Ok(home) = std::env::var("HOME") {
                path_str.replacen("~", &home, 1)
            } else {
                path_str.clone()
            }
        } else {
            path_str.clone()
        };

        let path = PathBuf::from(&expanded_path);

        // Read the file
        let file_data = match fs::read(&path) {
            Ok(data) => data,
            Err(e) => {
                self.status_message = Some(format!("Failed to read file: {}", e));
                return;
            }
        };

        // For local uploader, no password needed
        if uploader_config.uploader_type == "local" {
            self.execute_upload(entry, uploader_name, uploader_config, file_data, None);
            return;
        }

        // For SSH/SFTP, try to get password from keychain first
        let keychain_key = format!("ssh_password_{}", uploader_name);
        let stored_password = self.keychain_manager
            .as_ref()
            .and_then(|km| km.get(&keychain_key).ok().flatten());

        if let Some(password) = stored_password {
            // Try with stored password
            self.status_message = Some(format!("Uploading {} to {}...", entry.filename, uploader_name));
            self.execute_upload(entry, uploader_name, uploader_config, file_data, Some(password));
        } else {
            // No stored password, prompt for it
            self.pending_reupload = Some(PendingReupload {
                entry,
                uploader_name,
                uploader_config,
                file_data,
            });
            self.show_password_prompt = true;
            self.password_buffer.clear();
            self.status_message = Some("Enter SSH password:".to_string());
        }
    }

    fn execute_reupload_with_password(&mut self) {
        let password = self.password_buffer.clone();
        self.show_password_prompt = false;
        self.password_buffer.clear();

        let pending = match self.pending_reupload.take() {
            Some(p) => p,
            None => {
                self.status_message = Some("No pending upload".to_string());
                return;
            }
        };

        self.status_message = Some(format!("Uploading {} to {}...", pending.entry.filename, pending.uploader_name));

        let success = self.execute_upload(
            pending.entry,
            pending.uploader_name.clone(),
            pending.uploader_config,
            pending.file_data,
            Some(password.clone()),
        );

        // If successful, store password in keychain
        if success {
            if let Some(ref keychain) = self.keychain_manager {
                let keychain_key = format!("ssh_password_{}", pending.uploader_name);
                if let Err(e) = keychain.set(&keychain_key, &password) {
                    self.status_message = Some(format!(
                        "{} (Warning: failed to save password: {})",
                        self.status_message.as_deref().unwrap_or(""),
                        e
                    ));
                }
            }
        }
    }

    fn execute_upload(
        &mut self,
        entry: HistoryEntry,
        uploader_name: String,
        uploader_config: UploadConfig,
        file_data: Vec<u8>,
        password: Option<String>,
    ) -> bool {
        // Create uploader based on type with password
        let uploader: Box<dyn Uploader> = match uploader_config.uploader_type.as_str() {
            "sftp" => {
                let mut u = SftpUploader::new(uploader_name.clone(), uploader_config.clone());
                if let Some(ref pwd) = password {
                    u.set_password(pwd.clone());
                }
                Box::new(u)
            }
            "ssh" => {
                let mut u = SshUploader::new(uploader_name.clone(), uploader_config.clone());
                if let Some(ref pwd) = password {
                    u.set_password(pwd.clone());
                }
                Box::new(u)
            }
            "local" => Box::new(LocalUploader::new(uploader_name.clone(), uploader_config.clone())),
            _ => {
                self.status_message = Some(format!("Unknown uploader type: {}", uploader_config.uploader_type));
                return false;
            }
        };

        // Validate uploader config
        if let Err(e) = uploader.validate() {
            self.status_message = Some(format!("Invalid uploader config: {}", e));
            return false;
        }

        // Run the async upload in a blocking manner
        let rt = match tokio::runtime::Runtime::new() {
            Ok(rt) => rt,
            Err(e) => {
                self.status_message = Some(format!("Failed to create runtime: {}", e));
                return false;
            }
        };

        let filename = entry.filename.clone();
        let result = rt.block_on(async {
            uploader.upload(&file_data, &filename).await
        });

        match result {
            Ok(upload_result) => {
                let url_or_path = upload_result.url.as_ref().unwrap_or(&upload_result.remote_path);

                // Copy to clipboard
                if let Some(ref mut clipboard) = self.clipboard_manager {
                    let _ = clipboard.set_text(url_or_path);
                }

                self.status_message = Some(format!(
                    "✓ Re-uploaded to {}: {}",
                    uploader_name,
                    url_or_path
                ));
                true
            }
            Err(e) => {
                let error_msg = format!("{}", e);
                // Check if it's an auth error - prompt for password
                if error_msg.contains("authentication") || error_msg.contains("password") || error_msg.contains("Authentication") {
                    self.pending_reupload = Some(PendingReupload {
                        entry,
                        uploader_name,
                        uploader_config,
                        file_data,
                    });
                    self.show_password_prompt = true;
                    self.password_buffer.clear();
                    self.status_message = Some("Authentication failed. Enter password:".to_string());
                } else {
                    self.status_message = Some(format!("✗ Upload failed: {}", e));
                }
                false
            }
        }
    }
}
