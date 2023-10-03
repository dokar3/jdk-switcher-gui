use std::fs::{File, OpenOptions};

use crate::{
    errors::AppError, model::settings::SettingsValues,
    util::paths::settings_json_path,
};

pub struct AppSettings {}

impl AppSettings {
    pub fn load() -> SettingsValues {
        let json_path = settings_json_path();
        if !json_path.exists() {
            return AppSettings::default_values();
        }
        let Ok(file) = File::open(json_path) else {
            return AppSettings::default_values();
        };
        let Ok(values) = serde_json::from_reader::<File, SettingsValues>(file) else {
            return AppSettings::default_values();
        };
        return values;
    }

    pub fn update(values: &SettingsValues) -> Result<(), AppError> {
        let json_path = settings_json_path();

        let file = if !json_path.exists() {
            if let Some(parent) = json_path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent.clone()).map_err(|e| {
                        AppError::new(format!(
                            "Cannot create dir: {}, error: {}",
                            parent.to_str().unwrap(),
                            e.to_string()
                        ))
                    })?;
                }
            }
            File::create(json_path).map_err(|e| {
                AppError::new(format!(
                    "Cannot create settings.json: {}",
                    e.to_string()
                ))
            })?
        } else {
            OpenOptions::new()
                .read(true)
                .write(true)
                .truncate(true)
                .open(json_path)
                .map_err(|e| {
                    AppError::new(format!(
                        "Cannot open settings.json: {}",
                        e.to_string()
                    ))
                })?
        };

        serde_json::to_writer(file, values).map_err(|e| {
            AppError::new(format!(
                "Cannot write to settings.json: {}",
                e.to_string()
            ))
        })
    }

    fn default_values() -> SettingsValues {
        SettingsValues {
            theme: AppSettings::default_theme(),
            skip_dir_selection_hint: false,
        }
    }

    fn default_theme() -> String {
        match dark_light::detect() {
            dark_light::Mode::Dark => "dark",
            dark_light::Mode::Light => "light",
            dark_light::Mode::Default => "default",
        }
        .to_string()
    }
}
