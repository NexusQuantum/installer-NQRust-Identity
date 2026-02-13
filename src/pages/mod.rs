mod confirmation;
mod error;
mod installing;
mod registry;
mod success;
mod update;

pub use confirmation::{ConfirmationView, render_confirmation};
pub use error::{ErrorView, render_error};
pub use installing::{InstallingView, render_installing};
pub use registry::{RegistrySetupView, render_registry_setup};
pub use success::{SuccessView, render_success};
pub use update::{UpdateListView, render_update_list};
