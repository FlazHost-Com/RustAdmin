//! RustAdmin — library crate.
//!
//! Holds all application logic (config, errors, helpers, db, rbac, modules) so that
//! integration tests under `tests/` and the tool binaries under `src/bin/` can import
//! it. The web/api server (`src/main.rs`) is a thin wrapper around [`build_rocket`].
//!
//! Port of NodeAdmin keeping the same *concepts* via native Rust/Rocket idioms — see
//! `AGENTS.md` and `docs/PORTING_GUIDE.md` in the NodeAdmin reference.

#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket::{Build, Rocket};
use rocket_dyn_templates::{context, Template};

pub mod config;
pub mod db;
pub mod errors;
pub mod helpers;
pub mod migrations;
pub mod modules;
pub mod rbac;

/// Health endpoint — always mounted in both `full` and `api` modes.
#[get("/healthz")]
fn healthz() -> &'static str {
    "ok"
}

/// Temporary placeholder home (replaced by the `home` module in a later phase).
#[get("/")]
fn index() -> Template {
    Template::render("placeholder", context! { app_name: "RustAdmin" })
}

/// Build the Rocket instance. Branches on [`config::AppMode`] so an API-only install
/// (`APP_MODE=api`) skips the web layer purely-additively (see PORTING_GUIDE).
pub fn build_rocket() -> Rocket<Build> {
    // Load `.env` for local dev (no-op if absent).
    let _ = dotenvy::dotenv();

    let cfg = config::Config::from_env();

    let mut rocket = rocket::build()
        .manage(cfg.clone())
        .mount("/", routes![healthz]);

    if cfg.app.mode == config::AppMode::Full {
        rocket = rocket.mount("/", routes![index]).attach(Template::fairing());
    }

    rocket.attach(AdHoc::on_liftoff("Banner", |rocket| {
        Box::pin(async move {
            let rc = rocket.config();
            info!("RustAdmin listening on {}:{}", rc.address, rc.port);
        })
    }))
}
