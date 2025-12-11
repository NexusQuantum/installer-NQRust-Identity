#[derive(Debug, Default)]
pub struct RegistryForm {
    pub token: String,
    pub current_field: usize,
    pub editing: bool,
    pub error_message: String,
}

impl RegistryForm {
    pub fn new() -> Self {
        Self {
            token: String::new(),
            current_field: 0,
            editing: false,
            error_message: String::new(),
        }
    }

    pub fn total_items(&self) -> usize {
        2
    }

    pub fn is_input_field(index: usize) -> bool {
        index == 0
    }

    pub fn get_current_value_mut(&mut self) -> &mut String {
        &mut self.token
    }

    pub fn validate(&mut self) -> bool {
        if self.token.trim().is_empty() {
            self.error_message = "Personal access token is required".to_string();
            return false;
        }

        self.error_message.clear();
        true
    }
}
