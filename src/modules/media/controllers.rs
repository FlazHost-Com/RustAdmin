//! Media controller — AJAX file-manager endpoints (session + CSRF header).

use rocket::form::Form;
use rocket::fs::TempFile;
use rocket::http::Status;
use rocket::serde::json::{json, Json};
use rocket::tokio::io::AsyncReadExt;
use serde::Deserialize;
use serde_json::Value;

use crate::errors::AppError;
use crate::guards::CurrentUser;
use crate::modules::media::service;
use crate::security::csrf::CsrfProtected;

type ApiResult = Result<(Status, Json<Value>), AppError>;

#[derive(FromForm)]
pub struct UploadForm<'r> {
    pub file: TempFile<'r>,
}

#[derive(Deserialize)]
pub struct DeleteBody {
    pub key: String,
}

#[get("/media/list")]
pub async fn list(_user: CurrentUser) -> ApiResult {
    let items = service::list()?;
    Ok((Status::Ok, Json(json!({ "success": true, "data": items }))))
}

#[post("/media/upload", data = "<form>")]
pub async fn upload(
    _user: CurrentUser,
    _csrf: CsrfProtected,
    form: Form<UploadForm<'_>>,
) -> ApiResult {
    let mut reader = form
        .file
        .open()
        .await
        .map_err(|e| AppError::internal(format!("read upload: {e}")))?;
    let mut bytes = Vec::new();
    reader
        .read_to_end(&mut bytes)
        .await
        .map_err(|e| AppError::internal(format!("read upload: {e}")))?;
    let data = service::upload(&bytes)?;
    Ok((Status::Ok, Json(json!({ "success": true, "data": data }))))
}

#[post("/media/delete", data = "<body>")]
pub async fn delete(_user: CurrentUser, _csrf: CsrfProtected, body: Json<DeleteBody>) -> ApiResult {
    service::delete(&body.key)?;
    Ok((
        Status::Ok,
        Json(json!({ "success": true, "message": "Deleted" })),
    ))
}

pub fn routes() -> Vec<rocket::Route> {
    routes![list, upload, delete]
}
