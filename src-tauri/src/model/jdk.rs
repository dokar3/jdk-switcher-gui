#[derive(Clone, serde::Serialize, serde::Deserialize)]
pub struct Jdk {
    pub name: String,
    pub path: String,
    pub version: String,
    pub arch: String,
    #[serde(default)]
    pub is_current: bool,
    #[serde(default)]
    pub is_valid: bool,
}
