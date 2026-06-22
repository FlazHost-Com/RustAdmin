//! Web routes for the access module (mounted at `/admin/v1`).

use rocket::Route;

use crate::modules::access::controllers::web;

pub fn routes() -> Vec<Route> {
    routes![
        web::user::index,
        web::user::create,
        web::user::store,
        web::user::edit,
        web::user::update,
        web::user::delete,
        web::user::delete_selected,
    ]
}
