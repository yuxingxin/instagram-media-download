mod download;
mod parse;

pub use download::{download_post, download_posts, download_target, find_instaloader, DownloadResult};
pub use parse::{parse_links, parse_shortcode, parse_target, DownloadTarget, ParseError};

use std::path::PathBuf;
use tauri::Manager;

#[tauri::command]
fn parse_input(text: String) -> Result<Vec<String>, String> {
    parse::parse_links(&text)
        .map(|targets| targets.into_iter().map(|t| t.display_label()).collect())
        .map_err(|e| e.to_string())
}

#[tauri::command]
fn default_destination() -> Result<String, String> {
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map_err(|_| "HOME is not set".to_string())?;
    let dest = PathBuf::from(home).join("Pictures").join("Instagram");
    std::fs::create_dir_all(&dest).map_err(|e| e.to_string())?;
    Ok(dest.display().to_string())
}

#[tauri::command]
fn pick_folder(app: tauri::AppHandle) -> Result<Option<String>, String> {
    use tauri_plugin_dialog::DialogExt;
    let folder = app.dialog().file().blocking_pick_folder();
    match folder {
        None => Ok(None),
        Some(path) => {
            let path = path.into_path().map_err(|e| e.to_string())?;
            Ok(Some(path.display().to_string()))
        }
    }
}

#[tauri::command]
async fn download_links(links: String, destination: String) -> Result<Vec<DownloadResult>, String> {
    tauri::async_runtime::spawn_blocking(move || {
        if destination.trim().is_empty() {
            return Err("请选择保存目录".into());
        }
        let targets = parse::parse_links(&links).map_err(|e| e.to_string())?;
        Ok(download::download_posts(
            &targets,
            std::path::Path::new(&destination),
        ))
    })
    .await
    .map_err(|e| e.to_string())?
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_dialog::init())
        .setup(|app| {
            if let Ok(dir) = app.path().resource_dir() {
                std::env::set_var("INS_RESOURCE_DIR", dir);
            }
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            parse_input,
            default_destination,
            pick_folder,
            download_links
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
