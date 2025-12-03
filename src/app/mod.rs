use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEventKind, KeyModifiers};
use ratatui::{DefaultTerminal, Frame};
use std::fs;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, BufReader};
use tokio::process::Command;

use crate::templates::{self, ConfigTemplate};
use crate::ui::{
    self, ConfigSelectionView, ConfirmationView, EnvSetupView, ErrorView, InstallingView,
    SuccessView,
};
use crate::utils;

pub mod form_data;
pub mod state;

pub use form_data::FormData;
pub use state::{AppState, MenuSelection};

#[derive(Debug)]
pub struct App {
    running: bool,
    pub(crate) state: AppState,
    logs: Vec<String>,
    progress: f64,
    current_service: String,
    total_services: usize,
    completed_services: usize,
    pub(crate) env_exists: bool,
    pub(crate) config_exists: bool,
    pub(crate) form_data: FormData,
    pub(crate) menu_selection: MenuSelection,
    config_selection_index: usize,
}

impl App {
    pub fn new() -> Self {
        let env_exists = utils::find_file(".env");
        let config_exists = utils::find_file("config.yaml");

        let initial_state = AppState::Confirmation;

        // Urutan: pilih config dulu, baru bisa isi env
        let initial_menu = if !config_exists {
            MenuSelection::GenerateConfig
        } else if !env_exists {
            MenuSelection::GenerateEnv
        } else {
            MenuSelection::Proceed
        };

        Self {
            running: true,
            state: initial_state,
            logs: Vec::new(),
            progress: 0.0,
            current_service: String::new(),
            total_services: 4,
            completed_services: 0,
            env_exists,
            config_exists,
            form_data: FormData::new(),
            menu_selection: initial_menu,
            config_selection_index: 0,
        }
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while self.running {
            terminal.draw(|frame| self.render(frame))?;

            match &self.state {
                AppState::Confirmation => {
                    if let Some(action) = self.handle_confirmation_events()? {
                        match action {
                            MenuSelection::Proceed => {
                                if self.env_exists && self.config_exists {
                                    self.state = AppState::Installing;
                                    self.logs
                                        .push("🚀 Starting Analytics installation...".to_string());

                                    let result = self.run_docker_compose().await;

                                    match result {
                                        Ok(_) => {
                                            self.state = AppState::Success;
                                            self.progress = 100.0;
                                        }
                                        Err(e) => {
                                            self.state = AppState::Error(format!(
                                                "Installation failed: {}",
                                                e
                                            ));
                                        }
                                    }
                                }
                            }
                            MenuSelection::GenerateEnv => {
                                // Pastikan config sudah dipilih
                                if !self.config_exists {
                                    // Should not happen, but safety check - go to config selection
                                    if templates::CONFIG_TEMPLATES.is_empty() {
                                        self.state = AppState::Error(
                                            "No configuration templates available".to_string(),
                                        );
                                    } else {
                                        self.config_selection_index = 0;
                                        self.state = AppState::ConfigSelection;
                                    }
                                } else if self.form_data.selected_provider.is_empty() {
                                    // Provider belum dipilih - go to config selection first
                                    if templates::CONFIG_TEMPLATES.is_empty() {
                                        self.state = AppState::Error(
                                            "No configuration templates available".to_string(),
                                        );
                                    } else {
                                        self.config_selection_index = 0;
                                        self.state = AppState::ConfigSelection;
                                    }
                                } else {
                                    self.state = AppState::EnvSetup;
                                }
                            }
                            MenuSelection::GenerateConfig => {
                                if templates::CONFIG_TEMPLATES.is_empty() {
                                    self.state = AppState::Error(
                                        "No configuration templates available".to_string(),
                                    );
                                } else {
                                    self.config_selection_index = 0;
                                    self.state = AppState::ConfigSelection;
                                }
                            }
                            MenuSelection::Cancel => {
                                self.running = false;
                            }
                        }
                    }
                }
                AppState::EnvSetup => {
                    if let Some(proceed) = self.handle_form_events()? {
                        if proceed {
                            if let Err(e) = self.generate_env_file() {
                                self.state =
                                    AppState::Error(format!("Failed to generate .env: {}", e));
                            } else {
                                self.env_exists = true;
                                self.state = AppState::Confirmation;
                                if !self.config_exists {
                                    self.menu_selection = MenuSelection::GenerateConfig;
                                } else {
                                    self.menu_selection = MenuSelection::Proceed;
                                }
                            }
                        } else {
                            self.state = AppState::Confirmation;
                        }
                    }
                }
                AppState::ConfigSelection => {
                    self.handle_config_selection_events()?;
                }
                AppState::Installing => {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                if let KeyCode::Char('c') = key.code {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        self.running = false;
                                    }
                                }
                            }
                        }
                    }
                }
                AppState::Success | AppState::Error(_) => {
                    if event::poll(std::time::Duration::from_millis(100))? {
                        if let Event::Key(key) = event::read()? {
                            if key.kind == KeyEventKind::Press {
                                if let KeyCode::Char('c') = key.code {
                                    if key.modifiers.contains(KeyModifiers::CONTROL) {
                                        self.running = false;
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }

    fn handle_confirmation_events(&mut self) -> Result<Option<MenuSelection>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            self.menu_selection = match self.menu_selection {
                                MenuSelection::Proceed => {
                                    if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else if !self.env_exists {
                                        MenuSelection::GenerateEnv
                                    } else {
                                        MenuSelection::Cancel
                                    }
                                }
                                MenuSelection::GenerateEnv => {
                                    // Urutan: config dulu, baru env
                                    MenuSelection::GenerateConfig
                                }
                                MenuSelection::GenerateConfig => MenuSelection::Cancel,
                                MenuSelection::Cancel => {
                                    if self.env_exists && self.config_exists {
                                        MenuSelection::Proceed
                                    } else if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else {
                                        MenuSelection::GenerateEnv
                                    }
                                }
                            };
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            self.menu_selection = match self.menu_selection {
                                MenuSelection::Proceed => MenuSelection::Cancel,
                                MenuSelection::GenerateEnv => {
                                    if self.config_exists && self.env_exists {
                                        MenuSelection::Proceed
                                    } else {
                                        MenuSelection::Cancel
                                    }
                                }
                                MenuSelection::GenerateConfig => {
                                    // Urutan: config dulu, baru env
                                    if !self.env_exists {
                                        MenuSelection::GenerateEnv
                                    } else {
                                        MenuSelection::Cancel
                                    }
                                }
                                MenuSelection::Cancel => {
                                    if !self.config_exists {
                                        MenuSelection::GenerateConfig
                                    } else if !self.env_exists {
                                        MenuSelection::GenerateEnv
                                    } else {
                                        MenuSelection::Proceed
                                    }
                                }
                            };
                        }
                        KeyCode::Enter => {
                            return Ok(Some(self.menu_selection.clone()));
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            return Ok(Some(MenuSelection::Cancel));
                        }
                        KeyCode::Char('c') => {
                            if key.modifiers.contains(KeyModifiers::CONTROL) {
                                return Ok(Some(MenuSelection::Cancel));
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        Ok(None)
    }

    fn handle_form_events(&mut self) -> Result<Option<bool>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    if self.form_data.editing {
                        match key.code {
                            KeyCode::Enter | KeyCode::Esc => {
                                self.form_data.editing = false;
                            }
                            KeyCode::Char(c) => {
                                self.form_data.get_current_value_mut().push(c);
                            }
                            KeyCode::Backspace => {
                                self.form_data.get_current_value_mut().pop();
                            }
                            _ => {}
                        }
                    } else {
                        match key.code {
                            KeyCode::Enter => {
                                if self.form_data.current_field < self.form_data.get_total_fields()
                                {
                                    self.form_data.editing = true;
                                }
                            }
                            KeyCode::Up => {
                                if self.form_data.current_field > 0 {
                                    self.form_data.current_field -= 1;
                                }
                            }
                            KeyCode::Down | KeyCode::Tab => {
                                if self.form_data.current_field
                                    < self.form_data.get_total_fields() - 1
                                {
                                    self.form_data.current_field += 1;
                                }
                            }
                            KeyCode::Char('s') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                if self.form_data.validate() {
                                    return Ok(Some(true));
                                }
                            }
                            KeyCode::Esc | KeyCode::Char('q') => {
                                return Ok(Some(false));
                            }
                            KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                                return Ok(Some(false));
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        Ok(None)
    }

    fn handle_config_selection_events(&mut self) -> Result<()> {
        let total = templates::CONFIG_TEMPLATES.len();

        if total == 0 {
            self.state = AppState::Error("No configuration templates available".to_string());
            return Ok(());
        }

        if self.config_selection_index >= total {
            self.config_selection_index = total - 1;
        }

        if event::poll(std::time::Duration::from_millis(100))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    match key.code {
                        KeyCode::Up => {
                            if self.config_selection_index == 0 {
                                self.config_selection_index = total - 1;
                            } else {
                                self.config_selection_index -= 1;
                            }
                        }
                        KeyCode::Down | KeyCode::Tab => {
                            self.config_selection_index = (self.config_selection_index + 1) % total;
                        }
                        KeyCode::Enter => {
                            if let Some(template) =
                                templates::CONFIG_TEMPLATES.get(self.config_selection_index)
                            {
                                match self.write_config_yaml(template) {
                                    Ok(_) => {
                                        self.config_exists = true;
                                        // Set selected provider and go to env setup
                                        self.form_data.selected_provider = template.key.to_string();
                                        self.form_data.api_key.clear();
                                        self.form_data.openai_api_key.clear();
                                        self.form_data.current_field = 0;
                                        self.form_data.editing = false;
                                        self.form_data.error_message.clear();

                                        if !self.env_exists {
                                            self.state = AppState::EnvSetup;
                                        } else {
                                            self.state = AppState::Confirmation;
                                            self.menu_selection = MenuSelection::Proceed;
                                        }
                                    }
                                    Err(e) => {
                                        self.state = AppState::Error(format!(
                                            "Failed to generate config.yaml: {}",
                                            e
                                        ));
                                    }
                                }
                            }
                        }
                        KeyCode::Esc | KeyCode::Char('q') => {
                            self.state = AppState::Confirmation;
                        }
                        KeyCode::Char('c') if key.modifiers.contains(KeyModifiers::CONTROL) => {
                            self.running = false;
                        }
                        _ => {}
                    }
                }
            }
        }

        Ok(())
    }

    fn generate_env_file(&self) -> Result<()> {
        let project_root = utils::project_root();
        let env_path = project_root.join(".env");

        let uuid_fragment = uuid::Uuid::new_v4()
            .to_string()
            .split('-')
            .next()
            .unwrap_or("123")
            .to_string();
        let user_uuid = format!("demo-user-{}", uuid_fragment);

        let mut env_content = utils::ENV_TEMPLATE.to_string();

        // Default values
        env_content = env_content.replace("{{ANALYTICS_AI_SERVICE_PORT}}", "5555");
        env_content = env_content.replace("{{USER_UUID}}", user_uuid.as_str());
        env_content = env_content.replace("{{GENERATION_MODEL}}", "default");
        env_content = env_content.replace("{{HOST_PORT}}", "3000");
        env_content = env_content.replace("{{AI_SERVICE_FORWARD_PORT}}", "5555");

        // Set API key based on provider
        let env_key = self.form_data.get_env_key_name();
        let api_key_value = self.form_data.api_key.trim();
        let openai_api_key_value = self.form_data.openai_api_key.trim();

        // Handle different providers
        if env_key.is_empty() {
            // No API key needed (ollama)
            env_content = env_content.replace("{{OPENAI_API_KEY}}", "");
        } else {
            // First, handle provider-specific API key
            if env_key == "OPENAI_API_KEY" {
                // If provider is OpenAI itself, use the main API key
                env_content = env_content.replace("{{OPENAI_API_KEY}}", api_key_value);
            } else {
                // Process all lines to handle both provider key and OpenAI key
                let lines: Vec<&str> = env_content.lines().collect();
                let mut new_lines: Vec<String> = Vec::new();
                let mut added_provider_key = false;
                let mut added_openai_key = false;
                let needs_openai =
                    self.form_data.needs_openai_embedding() && !openai_api_key_value.is_empty();

                for line in lines {
                    let trimmed = line.trim();

                    // Skip the placeholder line for OPENAI_API_KEY if we're not using it as main key
                    if trimmed == "OPENAI_API_KEY={{OPENAI_API_KEY}}"
                        || trimmed == "OPENAI_API_KEY="
                    {
                        // Skip this line, we'll add it later if needed
                        continue;
                    }

                    // Check if this line already has the provider key we want to add
                    if trimmed.starts_with(&format!("{}=", env_key)) {
                        // Replace existing key
                        new_lines.push(format!("{}={}", env_key, api_key_value));
                        added_provider_key = true;
                    } else if trimmed.starts_with("OPENAI_API_KEY=") {
                        // Handle existing OPENAI_API_KEY line
                        if needs_openai {
                            new_lines.push(format!("OPENAI_API_KEY={}", openai_api_key_value));
                            added_openai_key = true;
                        }
                        // If not needed, skip this line
                    } else {
                        new_lines.push(line.to_string());
                        // Add after vendor keys comment if not added yet
                        if line.contains("# vendor keys") {
                            if !added_provider_key {
                                new_lines.push(format!("{}={}", env_key, api_key_value));
                                added_provider_key = true;
                            }
                            if needs_openai && !added_openai_key {
                                new_lines.push(format!("OPENAI_API_KEY={}", openai_api_key_value));
                                added_openai_key = true;
                            }
                        }
                    }
                }

                // Add provider key if not found
                if !added_provider_key {
                    new_lines.push(format!("{}={}", env_key, api_key_value));
                }

                // Add OpenAI key if needed and not found
                if needs_openai && !added_openai_key {
                    new_lines.push(format!("OPENAI_API_KEY={}", openai_api_key_value));
                }

                env_content = new_lines.join("\n");
            }
        }

        fs::write(env_path, env_content)?;
        Ok(())
    }

    fn write_config_yaml(&self, template: &ConfigTemplate) -> Result<()> {
        let project_root = utils::project_root();
        let config_path = project_root.join("config.yaml");
        fs::write(config_path, template.render())?;
        Ok(())
    }

    async fn run_docker_compose(&mut self) -> Result<()> {
        self.add_log("🔨 Step 1/2: Building images (no cache)...");
        self.add_log("📦 Executing: docker compose build --no-cache");

        let mut build_child = Command::new("docker")
            .args(["compose", "build", "--no-cache"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let build_stdout = build_child.stdout.take().expect("Failed to capture stdout");
        let build_stderr = build_child.stderr.take().expect("Failed to capture stderr");

        let mut build_stdout_reader = BufReader::new(build_stdout).lines();
        let mut build_stderr_reader = BufReader::new(build_stderr).lines();

        loop {
            tokio::select! {
                result = build_stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stdout: {}", e));
                            break;
                        }
                    }
                }
                result = build_stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stderr: {}", e));
                            break;
                        }
                    }
                }
            }
        }

        let build_status = build_child.wait().await?;

        if !build_status.success() {
            return Err(color_eyre::eyre::eyre!("Docker Compose build failed"));
        }

        self.add_log("✅ Build completed successfully!");
        self.progress = 50.0;

        self.add_log("🚀 Step 2/2: Starting services...");
        self.add_log("📦 Executing: docker compose up -d");

        let mut up_child = Command::new("docker")
            .args(["compose", "up", "-d"])
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;

        let up_stdout = up_child.stdout.take().expect("Failed to capture stdout");
        let up_stderr = up_child.stderr.take().expect("Failed to capture stderr");

        let mut up_stdout_reader = BufReader::new(up_stdout).lines();
        let mut up_stderr_reader = BufReader::new(up_stderr).lines();

        loop {
            tokio::select! {
                result = up_stdout_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stdout: {}", e));
                            break;
                        }
                    }
                }
                result = up_stderr_reader.next_line() => {
                    match result {
                        Ok(Some(line)) => self.process_log_line(&line),
                        Ok(None) => break,
                        Err(e) => {
                            self.add_log(&format!("❌ Error reading stderr: {}", e));
                            break;
                        }
                    }
                }
            }
        }

        let up_status = up_child.wait().await?;

        if up_status.success() {
            self.add_log("✅ All services started successfully!");
            self.progress = 100.0;
            Ok(())
        } else {
            Err(color_eyre::eyre::eyre!("Docker Compose up failed"))
        }
    }

    fn process_log_line(&mut self, line: &str) {
        let lower = line.to_lowercase();

        if lower.contains("pulling") {
            if let Some(service) = self.extract_service_name(line) {
                self.current_service = service.clone();
                self.add_log(&format!("⬇️  Pulling image for {}...", service));
            }
        } else if lower.contains("pulled") {
            self.add_log("✓ Image pulled");
        } else if lower.contains("creating") {
            if let Some(service) = self.extract_service_name(line) {
                self.current_service = service.clone();
                self.add_log(&format!("🔨 Creating container {}...", service));
            }
        } else if lower.contains("created") {
            self.add_log("✓ Container created");
        } else if lower.contains("starting") {
            if let Some(service) = self.extract_service_name(line) {
                self.current_service = service.clone();
                self.add_log(&format!("▶️  Starting service {}...", service));
            }
        } else if lower.contains("started") {
            self.completed_services += 1;
            self.progress =
                50.0 + (self.completed_services as f64 / self.total_services as f64) * 50.0;
            self.add_log(&format!(
                "✅ Service started ({}/{})",
                self.completed_services, self.total_services
            ));
        } else if lower.contains("running") {
            self.add_log("🟢 Service is running");
        } else if lower.contains("error") || lower.contains("failed") {
            self.add_log(&format!("❌ {}", line));
        } else if !line.trim().is_empty() {
            self.add_log(&format!("ℹ️  {}", line));
        }
    }

    fn extract_service_name(&self, line: &str) -> Option<String> {
        let services = [
            "analytics-service",
            "qdrant",
            "northwind-db",
            "analytics-ui",
        ];

        for service in services {
            if line.to_lowercase().contains(service) {
                return Some(service.to_string());
            }
        }
        None
    }

    fn add_log(&mut self, message: &str) {
        self.logs.push(message.to_string());

        if self.logs.len() > 100 {
            self.logs.remove(0);
        }
    }

    fn render(&mut self, frame: &mut Frame) {
        match &self.state {
            AppState::Confirmation => {
                let view = ConfirmationView {
                    env_exists: self.env_exists,
                    config_exists: self.config_exists,
                    menu_selection: &self.menu_selection,
                };
                ui::render_confirmation(frame, &view);
            }
            AppState::EnvSetup => {
                let view = EnvSetupView {
                    form_data: &self.form_data,
                };
                ui::render_env_setup(frame, &view);
            }
            AppState::ConfigSelection => {
                let view = ConfigSelectionView {
                    templates: templates::CONFIG_TEMPLATES,
                    selected_index: self.config_selection_index,
                };
                ui::render_config_selection(frame, &view);
            }
            AppState::Installing => {
                let view = InstallingView {
                    progress: self.progress,
                    current_service: &self.current_service,
                    completed_services: self.completed_services,
                    total_services: self.total_services,
                    logs: &self.logs,
                };
                ui::render_installing(frame, &view);
            }
            AppState::Success => {
                let view = SuccessView { logs: &self.logs };
                ui::render_success(frame, &view);
            }
            AppState::Error(err) => {
                let view = ErrorView { logs: &self.logs };
                ui::render_error(frame, err, &view);
            }
        }
    }
}
