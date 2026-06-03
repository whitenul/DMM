#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScannedApp {
    pub name: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub app_type: String,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
}
