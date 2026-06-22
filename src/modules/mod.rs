//! Feature modules (one `mod` per feature — mirrors NodeAdmin `src/modules/*`).
//!
//! Each module owns its layers: models (SeaORM entities), services (trait + impl),
//! validators, controllers (Rocket routes), views (Tera), and tests. Modules are mounted
//! explicitly in [`crate::build_rocket`]. UI-only modules are mounted under a presence
//! guard so an API-only build (`APP_MODE=api`) drops them purely-additively.

pub mod access;
pub mod setting;
