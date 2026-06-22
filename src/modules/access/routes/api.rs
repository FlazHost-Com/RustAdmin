//! API routes for the access module (mounted at `/api/v1`).

use rocket::Route;

use crate::modules::access::controllers::api;

pub fn routes() -> Vec<Route> {
    routes![
        api::user::index,
        api::user::store,
        api::user::edit,
        api::user::update,
        api::user::delete,
        api::user::delete_selected,
    ]
}
