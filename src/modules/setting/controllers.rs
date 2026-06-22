//! Setting web controller — theme switcher + site config form (singleton).

use std::sync::Arc;

use rocket::form::Form;
use rocket::http::CookieJar;
use rocket::request::FlashMessage;
use rocket::response::{Flash, Redirect};
use rocket::State;
use rocket_dyn_templates::Template;
use sea_orm::DatabaseConnection;
use serde_json::{json, Value};

use crate::errors::AppError;
use crate::guards::Authorized;
use crate::helpers::view::{nav_for, render_view};
use crate::modules::setting::services::{ISettingService, SettingInput};
use crate::security::csrf::{ensure_token, CsrfProtected};

const INDEX_URL: &str = "/admin/v1/setting";

fn chrome(user: &crate::guards::CurrentUser, csrf: &str) -> Value {
    json!({
        "auth": { "name": user.name, "picture": null },
        "nav": nav_for(user.is_admin, &user.perms),
        "csrf_token": csrf,
        "active": "setting",
    })
}

#[derive(rocket::FromForm, Debug, Default)]
pub struct SettingForm {
    pub initial: Option<String>,
    pub name: Option<String>,
    pub description: Option<String>,
    pub icon: Option<String>,
    pub logo: Option<String>,
    pub login_image: Option<String>,
    pub phone: Option<String>,
    pub address: Option<String>,
    pub email: Option<String>,
    pub copyright: Option<String>,
    pub theme: Option<String>,
    pub fe_template: Option<String>,
}

impl From<SettingForm> for SettingInput {
    fn from(f: SettingForm) -> Self {
        SettingInput {
            initial: f.initial,
            name: f.name,
            description: f.description,
            icon: f.icon,
            logo: f.logo,
            login_image: f.login_image,
            phone: f.phone,
            address: f.address,
            email: f.email,
            copyright: f.copyright,
            theme: f.theme,
            fe_template: f.fe_template,
        }
    }
}

#[get("/setting")]
pub async fn index(
    auth: Authorized,
    db: &State<DatabaseConnection>,
    svc: &State<Arc<dyn ISettingService>>,
    cookies: &CookieJar<'_>,
    flash: Option<FlashMessage<'_>>,
) -> Result<Template, AppError> {
    let setting = svc.get(db.inner()).await?;
    let csrf = ensure_token(cookies);
    let flash_v = match &flash {
        Some(m) => json!({ "key": m.kind(), "message": m.message() }),
        None => json!({}),
    };
    let mut page = json!({ "setting": setting, "flash": flash_v });
    // merge chrome
    if let (Value::Object(b), Value::Object(c)) = (&mut page, chrome(&auth.0, &csrf)) {
        for (k, v) in c {
            b.insert(k, v);
        }
    }
    Ok(render_view("be/default/setting/index", page, None))
}

#[put("/setting/update", data = "<form>")]
pub async fn update(
    _auth: Authorized,
    _csrf: CsrfProtected,
    db: &State<DatabaseConnection>,
    svc: &State<Arc<dyn ISettingService>>,
    form: Form<SettingForm>,
) -> Flash<Redirect> {
    match svc.update(db.inner(), form.into_inner().into()).await {
        Ok(_) => Flash::success(Redirect::to(INDEX_URL), "Setting saved successfully"),
        Err(e) => Flash::error(Redirect::to(INDEX_URL), e.message().to_string()),
    }
}

pub fn routes() -> Vec<rocket::Route> {
    routes![index, update]
}
