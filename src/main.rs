//! RustAdmin — bootstrap admin panel (Rust + Rocket).
//!
//! Port of NodeAdmin keeping the same *concepts* (SOLID/DI, central error handling,
//! route-driven RBAC, theme + frontend-template switcher, multi-DB) using native
//! Rust/Rocket idioms. Built in verifiable phases — see `enchanted-cuddling-frog`
//! plan / AGENTS.md.

#[macro_use]
extern crate rocket;

use rocket::fairing::AdHoc;
use rocket_dyn_templates::{context, Template};

/// Health endpoint — always available (both `full` and `api` modes).
#[get("/healthz")]
fn healthz() -> &'static str {
    "ok"
}

/// Temporary placeholder home (replaced by the `home` module in a later phase).
#[get("/")]
fn index() -> Template {
    Template::render("placeholder", context! { app_name: "RustAdmin" })
}

#[launch]
fn rocket() -> _ {
    // Load `.env` for local development (no-op if absent). Real, validated config
    // lands in `src/config/env.rs` in the next phase.
    let _ = dotenvy::dotenv();

    rocket::build()
        .mount("/", routes![index, healthz])
        .attach(Template::fairing())
        .attach(AdHoc::on_liftoff("Banner", |rocket| {
            Box::pin(async move {
                let cfg = rocket.config();
                info!("RustAdmin listening on {}:{}", cfg.address, cfg.port);
            })
        }))
}
