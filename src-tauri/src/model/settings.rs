#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct SettingsValues {
    pub theme: String,
    pub skip_dir_selection_hint: bool,
}
