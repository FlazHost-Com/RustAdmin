//! RustAdmin — library crate.
//!
//! Holds all application logic (config, errors, helpers, db, rbac, security, guards,
//! modules) so that integration tests under `tests/` and the tool binaries under
//! `src/bin/` can import it. The web/api server (`src/main.rs`) is a thin wrapper around
//! [`build_rocket`].
//!
//! Port of NodeAdmin keeping the same *concepts* via native Rust/Rocket idioms — see
//! `AGENTS.md` and `docs/PORTING_GUIDE.md` in the NodeAdmin reference.

#[macro_use]
extern crate rocket;

use std::sync::Arc;

use rocket::fairing::AdHoc;
use rocket::fs::FileServer;
use rocket::{Build, Rocket};
use rocket_dyn_templates::{context, Template};
use sea_orm::DatabaseConnection;

pub mod config;
pub mod db;
pub mod errors;
pub mod guards;
pub mod helpers;
pub mod migrations;
pub mod modules;
pub mod rbac;
pub mod security;

use config::{AppMode, Config};
use modules::access::services::{IUserService, UserService};
use modules::auth::service::{AuthService, IAuthService};
use security::blacklist::{InMemoryTokenStore, TokenStore};
use security::headers::SecurityHeaders;
use security::method_override::MethodOverride;

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

/// Build the Rocket instance from the environment, connecting the DB on ignite.
pub fn build_rocket() -> Rocket<Build> {
    let _ = dotenvy::dotenv();
    let cfg = Config::from_env();
    assemble(cfg, None)
}

/// Build the Rocket instance with a pre-made DB connection (used by integration tests with
/// an in-memory, already-migrated database).
pub fn build_rocket_with_db(cfg: Config, db: DatabaseConnection) -> Rocket<Build> {
    assemble(cfg, Some(db))
}

fn assemble(cfg: Config, db: Option<DatabaseConnection>) -> Rocket<Build> {
    let mode = cfg.app.mode;

    // DI container ≈ Rocket managed state. Services are shared as trait objects.
    let token_store: Arc<dyn TokenStore> = Arc::new(InMemoryTokenStore::new());
    let auth_service: Arc<dyn IAuthService> = Arc::new(AuthService);
    let user_service: Arc<dyn IUserService> = Arc::new(UserService);

    let mut rocket = rocket::build()
        .manage(cfg)
        .manage(token_store)
        .manage(auth_service)
        .manage(user_service)
        .attach(SecurityHeaders)
        .attach(MethodOverride)
        .mount("/", routes![healthz])
        .mount("/api/v1/auth", modules::auth::routes::api::routes())
        .mount("/api/v1", modules::access::routes::api::routes());

    // DB: inject (tests) or connect on ignite (server).
    match db {
        Some(conn) => {
            rocket = rocket.manage(conn);
        }
        None => {
            rocket = rocket.attach(AdHoc::try_on_ignite("Database", |rocket| async move {
                let cfg = rocket.state::<Config>().expect("config managed").clone();
                match db::connect(&cfg).await {
                    Ok(conn) => Ok(rocket.manage(conn)),
                    Err(e) => {
                        error!("database connection failed: {e}");
                        Err(rocket)
                    }
                }
            }));
        }
    }

    // Web-only layer (skipped purely-additively in API-only mode).
    if mode == AppMode::Full {
        rocket = rocket
            .mount("/", routes![index])
            .mount("/admin/v1", modules::access::routes::web::routes())
            .mount("/be/default", FileServer::from("static/be/default").rank(10))
            .mount("/static", FileServer::from("static").rank(11))
            .attach(helpers::view::template_fairing());
    }

    rocket.attach(AdHoc::on_liftoff("Banner", |rocket| {
        Box::pin(async move {
            let rc = rocket.config();
            info!("RustAdmin listening on {}:{}", rc.address, rc.port);
        })
    }))
}
