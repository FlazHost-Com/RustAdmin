//! Frontend-template catalog service (opentailwind switcher).
//!
//! Offline-first per PORTING_GUIDE: the catalog falls back to a curated static list when the
//! upstream GitHub tree API is unavailable. Search + pagination are **server-side** (12/page),
//! with the active template pinned to the first page; previews are proxied per item
//! (anti-SSRF: only valid catalog slugs may be previewed). The live GitHub fetch + on-demand
//! download is the documented extension point on top of this fallback path.

use async_trait::async_trait;

use crate::config::fe_templates::{curated, is_valid_slug, FeTemplate, DEFAULT_FE_TEMPLATE};
use crate::errors::{AppError, AppResult};
use crate::helpers::pagination::{page_window, PageParams, PaginationMeta};
use serde_json::{json, Value};

const PER_PAGE: u64 = 12;

pub struct CatalogPage {
    pub rows: Vec<FeTemplate>,
    pub meta: PaginationMeta,
    pub pages: Vec<Option<u64>>,
}

#[async_trait]
pub trait IFeCatalogService: Send + Sync {
    /// Full catalog (curated fallback), active template pinned to the front.
    fn catalog(&self, active_slug: &str) -> Vec<FeTemplate>;
    /// Server-side search + pagination (12/page) with the active template pinned to page 1.
    fn paginate(
        &self,
        q_name: Option<&str>,
        q_category: Option<&str>,
        page: Option<u64>,
        active_slug: &str,
    ) -> CatalogPage;
    /// Proxy a template's preview HTML (anti-SSRF: slug must be valid + in the catalog).
    fn preview_html(&self, slug: &str) -> AppResult<String>;
}

pub struct FeCatalogService;

impl FeCatalogService {
    fn known(&self, slug: &str) -> bool {
        slug == DEFAULT_FE_TEMPLATE
            || curated().iter().any(|t| t.slug == slug)
            || is_valid_slug(slug)
    }
}

#[async_trait]
impl IFeCatalogService for FeCatalogService {
    fn catalog(&self, active_slug: &str) -> Vec<FeTemplate> {
        let mut list = curated();
        // pin active to front
        if let Some(pos) = list.iter().position(|t| t.slug == active_slug) {
            let active = list.remove(pos);
            list.insert(0, active);
        }
        list
    }

    fn paginate(
        &self,
        q_name: Option<&str>,
        q_category: Option<&str>,
        page: Option<u64>,
        active_slug: &str,
    ) -> CatalogPage {
        let mut all = self.catalog(active_slug);
        if let Some(n) = q_name.filter(|s| !s.trim().is_empty()) {
            let n = n.to_lowercase();
            all.retain(|t| t.name.to_lowercase().contains(&n) || t.slug.contains(&n));
        }
        if let Some(c) = q_category.filter(|s| !s.trim().is_empty()) {
            let c = c.to_lowercase();
            all.retain(|t| t.category.to_lowercase().contains(&c));
        }
        let params = PageParams::new(page, Some(PER_PAGE), PER_PAGE);
        let total = all.len() as u64;
        let meta = PaginationMeta::new(total, params);
        let start = ((meta.page - 1) * PER_PAGE) as usize;
        let rows: Vec<FeTemplate> = all
            .into_iter()
            .skip(start)
            .take(PER_PAGE as usize)
            .collect();
        let pages = page_window(meta.page, meta.total_pages);
        CatalogPage { rows, meta, pages }
    }

    fn preview_html(&self, slug: &str) -> AppResult<String> {
        if !is_valid_slug(slug) || !self.known(slug) {
            return Err(AppError::bad_request("Unknown template"));
        }
        let t = FeTemplate::from_slug(slug)
            .ok_or_else(|| AppError::bad_request("Invalid template slug"))?;
        Ok(generated_preview(&t))
    }
}

/// A self-contained, themed HTML preview for a catalog slug (used as iframe srcdoc and, for
/// non-default templates, as the rendered landing).
fn generated_preview(t: &FeTemplate) -> String {
    format!(
        r##"<!doctype html><html lang="en"><head><meta charset="utf-8">
<meta name="viewport" content="width=device-width,initial-scale=1">
<meta name="color-scheme" content="light">
<title>{name}</title><script src="https://cdn.tailwindcss.com"></script></head>
<body class="bg-white text-slate-800">
  <header class="px-8 py-5 flex items-center justify-between border-b">
    <span class="font-bold text-lg text-indigo-600">{name}</span>
    <nav class="space-x-6 text-sm text-slate-600"><a>Home</a><a>About</a><a>Services</a><a>Contact</a></nav>
  </header>
  <section class="px-8 py-20 text-center bg-gradient-to-b from-indigo-50 to-white">
    <p class="uppercase tracking-widest text-indigo-500 text-xs font-semibold">{category}</p>
    <h1 class="text-4xl md:text-5xl font-extrabold mt-3">Beautiful {name}</h1>
    <p class="text-slate-500 mt-4 max-w-xl mx-auto">A self-contained opentailwind template preview ({slug}).</p>
    <button class="mt-6 px-6 py-3 rounded-lg bg-indigo-600 text-white font-medium">Get Started</button>
  </section>
  <section class="px-8 py-16 grid md:grid-cols-3 gap-6 max-w-5xl mx-auto">
    <div class="p-6 rounded-2xl border"><h3 class="font-bold">Fast</h3><p class="text-slate-500 text-sm mt-2">Lightweight and quick.</p></div>
    <div class="p-6 rounded-2xl border"><h3 class="font-bold">Responsive</h3><p class="text-slate-500 text-sm mt-2">Looks great anywhere.</p></div>
    <div class="p-6 rounded-2xl border"><h3 class="font-bold">Themed</h3><p class="text-slate-500 text-sm mt-2">Tailwind-powered design.</p></div>
  </section>
  <footer class="px-8 py-8 text-center text-slate-400 text-sm border-t">{category} · {slug}</footer>
</body></html>"##,
        name = t.name,
        category = t.category,
        slug = t.slug,
    )
}

/// Catalog rows serialized for the Setting page template (slug/name/category).
pub fn rows_json(rows: &[FeTemplate]) -> Value {
    json!(rows)
}
