#[derive(Debug, Clone, PartialEq)]
pub enum AppState {
    RegistrySetup,
    Confirmation,
    UpdateList,
    UpdatePulling,
    Installing,
    Success,
    Error(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum MenuSelection {
    Proceed,
    UpdateToken,
    CheckUpdates,
    Cancel,
}
