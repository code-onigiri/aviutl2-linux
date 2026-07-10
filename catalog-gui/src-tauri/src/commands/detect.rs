use serde::Serialize;
use std::path::{Path, PathBuf};
use tauri::Manager;

use crate::commands::catalog::load_catalog;

#[derive(Serialize)]
pub struct DetectResult {
    pub package_id: String,
    pub version: String,
    pub matched: bool,
}

#[tauri::command]
pub async fn detect_installed(
    app: tauri::AppHandle,
    aviutl_root: String,
) -> Result<Vec<DetectResult>, String> {
    let pkgs = load_catalog(&app)?;
    let root = PathBuf::from(&aviutl_root);
    let mut results = Vec::new();

    for pkg in &pkgs {
        if let Some(versions) = &pkg.version {
            for ver_entry in versions.iter().rev() {
                let files = ver_entry.file.as_deref().unwrap_or(&[]);
                if files.is_empty() {
                    continue;
                }
                let all_match = files.iter().all(|f| {
                    let raw_path = f.path.as_deref().unwrap_or("");
                    let expected = f.xxh3_128.as_deref().unwrap_or("");
                    if raw_path.is_empty() || expected.is_empty() {
                        return false;
                    }
                    let resolved = resolve_macro(raw_path, &root, &PathBuf::new());
                    match calc_xxh3_hex(&resolved) {
                        Some(hash) => hash.eq_ignore_ascii_case(expected),
                        None => false,
                    }
                });
                if all_match {
                    results.push(DetectResult {
                        package_id: pkg.id.clone(),
                        version: ver_entry.version.clone(),
                        matched: true,
                    });
                    break;
                }
            }
        }
    }

    // Persist
    if let Ok(config_dir) = app.path().app_config_dir() {
        let mut map = std::collections::HashMap::new();
        for r in &results {
            map.insert(r.package_id.clone(), r.version.clone());
        }
        let path = config_dir.join("installed.json");
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).ok();
        }
        if let Ok(bytes) = serde_json::to_vec_pretty(&map) {
            std::fs::write(&path, &bytes).ok();
        }
    }

    Ok(results)
}

#[tauri::command]
pub fn calc_hash(path: String) -> Result<Option<String>, String> {
    Ok(calc_xxh3_hex(&PathBuf::from(&path)))
}

pub fn calc_xxh3_hex(path: &Path) -> Option<String> {
    let data = std::fs::read(path).ok()?;
    let hash = xxhash_rust::xxh3::xxh3_128(&data);
    Some(format!("{:032x}", hash))
}

pub fn resolve_macro(template: &str, aviutl_root: &Path, tmp_dir: &Path) -> PathBuf {
    let data_dir = get_data_dir(aviutl_root);
    let data_str = data_dir.to_string_lossy().to_string();
    let plugins_str = data_dir.join("Plugin").to_string_lossy().to_string();
    let scripts_str = data_dir.join("Script").to_string_lossy().to_string();

    // Note: on Linux + Proton GE, AviUtl2's data directory (Plugin, Script, Language)
    // lives under the Wine prefix (drive_c/ProgramData/aviutl2/), NOT the repo root.
    // We map {appDir} → dataDir so packages that extract to {appDir} don't
    // overwrite files in the repo root (README.md, etc.).
    let s = template
        .replace("{appDir}", &data_str)
        .replace("{dataDir}", &data_str)
        .replace("{pluginsDir}", &plugins_str)
        .replace("{scriptsDir}", &scripts_str)
        .replace("{tmp}", &tmp_dir.to_string_lossy());

    PathBuf::from(s)
}

fn get_data_dir(aviutl_root: &Path) -> PathBuf {
    // Portable mode: data/ subdirectory with Plugin/
    let data = aviutl_root.join("data");
    if data.exists() && data.join("Plugin").exists() {
        return data;
    }

    // Proton GE prefix: drive_c/ProgramData/aviutl2/
    // Check standard repo layout first
    let pfx_ge = aviutl_root.join("pfx-ge").join("pfx");
    let progdata = pfx_ge.join("drive_c").join("ProgramData").join("aviutl2");
    if progdata.exists() && progdata.join("Plugin").exists() {
        return progdata;
    }

    // Also check WINEPREFIX env var
    if let Ok(wine_prefix) = std::env::var("WINEPREFIX") {
        let wp = PathBuf::from(&wine_prefix);
        let wp_progdata = wp.join("drive_c").join("ProgramData").join("aviutl2");
        if wp_progdata.exists() && wp_progdata.join("Plugin").exists() {
            return wp_progdata;
        }
    }

    // Fallback: root itself
    aviutl_root.to_path_buf()
}
