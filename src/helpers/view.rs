//! Tera rendering helper (mirrors NodeAdmin `renderView()` in `utils/view.ts`).
//!
//! Every admin render injects the theme locals (`theme`, `themeName`, `themes`) so the
//! layout chrome can map them to CSS variables + the inline Tailwind config — switching
//! theme changes the whole UI without a rebuild. The `setting` local is layered in once the
//! Setting module exists (Phase 6).

use rocket_dyn_templates::Template;
use serde_json::{json, Map, Value};

use crate::config::themes::{get_theme, DEFAULT_THEME, THEMES};

/// All palettes as JSON (for the theme switcher swatches).
pub fn themes_json() -> Value {
    json!(THEMES)
}

/// Render a backend template, merging the standard theme locals into `locals`.
///
/// `locals` must be a JSON object; the active theme is resolved from `theme_name`
/// (falling back to the default).
pub fn render_view(name: &str, locals: Value, theme_name: Option<&str>) -> Template {
    let active = theme_name.unwrap_or(DEFAULT_THEME);
    let palette = get_theme(active);

    let mut ctx: Map<String, Value> = match locals {
        Value::Object(m) => m,
        Value::Null => Map::new(),
        other => {
            let mut m = Map::new();
            m.insert("data".into(), other);
            m
        }
    };

    ctx.entry("themes").or_insert_with(themes_json);
    ctx.entry("themeName")
        .or_insert_with(|| json!(active));
    ctx.entry("theme").or_insert_with(|| json!(palette));

    Template::render(name.to_string(), Value::Object(ctx))
}
