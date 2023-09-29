#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SettingsValues {
    pub theme: String,
    pub show_dir_selection_hint: bool,
}
