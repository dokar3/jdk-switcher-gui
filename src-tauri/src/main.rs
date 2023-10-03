// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app_settings;
mod app_view_model;
mod errors;
mod jdk_finder;
mod jdk_switcher;
mod model;
mod repo;
mod util;

use app_view_model::AppViewModel;
use errors::AppError;
use indoc::formatdoc;
use model::jdk::Jdk;
use std::path::PathBuf;
use tauri::Manager;
use tauri_plugin_dialog::DialogExt;

fn main() {
    tauri::Builder::default()
        .setup(|app| {
            let view_model = AppViewModel::new();
            app.manage(view_model);
            Ok(())
        })
        .plugin(tauri_plugin_dialog::init())
        .invoke_handler(tauri::generate_handler![
            open_about_dialog,
            open_folder,
            listen_ui_state_stream,
            load_jdks,
            add_jdks_from_dir,
            remove_jdk_by_path,
            switch_to_jdk,
            update_app_theme,
            update_skip_dir_selection_hint,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

#[tauri::command]
async fn open_about_dialog(window: tauri::Window) {
    let package = util::cargo_manifest::read_cargo_package();
    let message = formatdoc! {
        r#"
        {}
        ----------
        {}
        ----------
        Version: {}
        Repository: {}"#,
        package.name,
        package.description,
        package.version,
        package.repository,
    };
    window
        .dialog()
        .message(message)
        .title("About")
        .kind(tauri_plugin_dialog::MessageDialogKind::Info)
        .parent(&window)
        .show(|_| {});
}

#[tauri::command]
async fn open_folder(path: String) -> Result<(), String> {
    let path = PathBuf::from(path);
    if !path.exists() {
        return Err("Dir does not exist.".to_string());
    }
    if !path.is_dir() {
        return Err("Target path is not a directory.".to_string());
    }
    open::that(path).map_err(|e| e.to_string())
}

#[tauri::command]
async fn listen_ui_state_stream(
    window: tauri::Window,
    view_model: tauri::State<'_, AppViewModel>,
) -> Result<(), ()> {
    let receiver = view_model.ui_state_stream();
    while !receiver.is_closed() {
        let ui_state = receiver.recv().await.unwrap();
        window.emit("ui-state-stream", ui_state).unwrap();
    }
    Ok(())
}

#[tauri::command]
async fn load_jdks(
    view_model: tauri::State<'_, AppViewModel>,
) -> Result<(), ()> {
    view_model.load_jdks();
    Ok(())
}

#[tauri::command]
async fn add_jdks_from_dir(
    view_model: tauri::State<'_, AppViewModel>,
    dir: String,
) -> Result<usize, AppError> {
    view_model.try_add_jdks_from_dir(&dir)
}

#[tauri::command]
async fn switch_to_jdk(
    view_model: tauri::State<'_, AppViewModel>,
    jdk: Jdk,
) -> Result<(), AppError> {
    view_model.switch_to_jdk(&jdk)
}

#[tauri::command]
async fn remove_jdk_by_path(
    view_model: tauri::State<'_, AppViewModel>,
    path: String,
) -> Result<(), AppError> {
    view_model.remove_jdk_by_path(&path)
}

#[tauri::command]
async fn update_app_theme(
    view_model: tauri::State<'_, AppViewModel>,
    theme: String,
) -> Result<(), AppError> {
    view_model.update_app_theme(&theme)
}

#[tauri::command]
async fn update_skip_dir_selection_hint(
    view_model: tauri::State<'_, AppViewModel>,
    value: bool,
) -> Result<(), AppError> {
    view_model.update_skip_dir_selection_hint(value)
}
