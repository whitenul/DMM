use desk_core::error::AppError;
use std::path::PathBuf;
use tauri::{plugin::TauriPlugin, Manager, Runtime};

pub mod migrations;
pub use migrations::DeskWebMigrations;

// ---------------------------------------------------------------------------
// WebMetadata — raw metadata extracted from a web page
// ---------------------------------------------------------------------------

pub struct WebMetadata {
    pub title: Option<String>,
    pub favicon_url: Option<String>,
}

// ---------------------------------------------------------------------------
// WebState — managed Tauri state holding app_data_dir
// ---------------------------------------------------------------------------

pub struct WebState {
    pub app_data_dir: PathBuf,
}

// ---------------------------------------------------------------------------
// Web metadata extraction service
// ---------------------------------------------------------------------------

pub async fn fetch_web_metadata(url: &str) -> Result<WebMetadata, AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) DeskManager/1.0")
        .build()
        .map_err(|e| AppError::Network(e.to_string()))?;

    let response = client
        .get(url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;
    let html = response
        .text()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;

    let document = scraper::Html::parse_document(&html);

    let title = scraper::Selector::parse("title")
        .ok()
        .and_then(|selector| {
            document
                .select(&selector)
                .next()
                .map(|el| el.text().collect::<String>().trim().to_string())
                .filter(|t| !t.is_empty())
        });

    let favicon_url = extract_favicon_url(&document, url);

    Ok(WebMetadata { title, favicon_url })
}

fn extract_favicon_url(document: &scraper::Html, base_url: &str) -> Option<String> {
    let selector_strs = [
        "link[rel='icon'][href]",
        "link[rel='shortcut icon'][href]",
        "link[rel='apple-touch-icon'][href]",
    ];
    for selector_str in &selector_strs {
        if let Ok(selector) = scraper::Selector::parse(selector_str) {
            if let Some(element) = document.select(&selector).next() {
                if let Some(href) = element.value().attr("href") {
                    return Some(resolve_url(base_url, href));
                }
            }
        }
    }
    None
}

fn resolve_url(base_url: &str, href: &str) -> String {
    if href.starts_with("http://") || href.starts_with("https://") {
        return href.to_string();
    }
    if let Ok(base) = url::Url::parse(base_url) {
        if let Ok(joined) = base.join(href) {
            return joined.to_string();
        }
    }
    href.to_string()
}

pub async fn download_favicon(
    favicon_url: &str,
    save_path: &std::path::Path,
) -> Result<(), AppError> {
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .map_err(|e| AppError::Network(e.to_string()))?;
    let response = client
        .get(favicon_url)
        .send()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;
    let bytes = response
        .bytes()
        .await
        .map_err(|e| AppError::Network(e.to_string()))?;
    if let Some(parent) = save_path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(save_path, &bytes)?;
    Ok(())
}

// ---------------------------------------------------------------------------
// Tauri Commands (in a submodule to avoid __cmd__ macro name collisions)
// ---------------------------------------------------------------------------

mod commands {
    use super::{download_favicon, fetch_web_metadata, AppError, WebState};
    use serde::Serialize;
    use tauri::State;

    #[derive(Serialize)]
    pub struct WebMetadataResult {
        pub title: Option<String>,
        pub icon_path: Option<String>,
    }

    #[tauri::command]
    pub async fn fetch_web_meta(
        url: String,
        state: State<'_, WebState>,
    ) -> Result<WebMetadataResult, AppError> {
        let meta = fetch_web_metadata(&url).await?;
        let icon_path = if let Some(favicon_url) = &meta.favicon_url {
            let icons_dir = state.app_data_dir.join("icons");
            let icon_file = icons_dir.join(format!("web_{}.ico", simple_hash(&url)));
            if download_favicon(favicon_url, &icon_file).await.is_ok() {
                Some(icon_file.to_string_lossy().to_string())
            } else {
                None
            }
        } else {
            None
        };
        Ok(WebMetadataResult {
            title: meta.title,
            icon_path,
        })
    }

    fn simple_hash(s: &str) -> u64 {
        let mut hash: u64 = 5381;
        for byte in s.bytes() {
            hash = hash.wrapping_mul(33).wrapping_add(byte as u64);
        }
        hash
    }
}

// ---------------------------------------------------------------------------
// Plugin init
// ---------------------------------------------------------------------------

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    tauri::plugin::Builder::new("desk-web")
        .invoke_handler(tauri::generate_handler![commands::fetch_web_meta])
        .setup(|app, _api| {
            let app_data_dir = app
                .path()
                .app_data_dir()
                .map_err(|e| AppError::Network(e.to_string()))?;
            app.manage(WebState { app_data_dir });
            Ok(())
        })
        .build()
}
