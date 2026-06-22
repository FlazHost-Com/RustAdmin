//! Access services (trait + impl). Shared as `State<Arc<dyn I*Service>>` (DI container).

pub mod user_service;

pub use user_service::{IUserService, UserService};
