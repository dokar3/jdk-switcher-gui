use std::path::PathBuf;

use platform_dirs::AppDirs;

const APP_DATA_DIR: &str = "JdkSwitcher";

pub fn app_data_dir() -> PathBuf {
    AppDirs::new(Some(APP_DATA_DIR), false).unwrap().data_dir
}

pub fn jdks_json_path() -> PathBuf {
    app_data_dir().join("data").join("jdks.json")
}

pub fn settings_json_path() -> PathBuf {
    app_data_dir().join("settings.json")
}
