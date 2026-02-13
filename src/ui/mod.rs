mod ascii_art;

pub use ascii_art::{ASCII_HEADER, get_orange_accent, get_orange_color};
pub use crate::pages::{
    ConfirmationView,
    ErrorView,
    InstallingView,
    RegistrySetupView,
    SuccessView,
    UpdateListView,
    render_confirmation,
    render_error,
    render_installing,
    render_registry_setup,
    render_success,
    render_update_list,
};
