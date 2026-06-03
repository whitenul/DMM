use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Item {
    pub id: i64,
    pub category_id: i64,
    pub name: String,
    pub pinyin_name: Option<String>,
    pub item_type: String,
    pub path: String,
    pub icon_path: Option<String>,
    pub arguments: Option<String>,
    pub working_dir: Option<String>,
    pub sort_order: i32,
    pub is_pinned: bool,
    pub created_at: String,
    pub updated_at: String,
}

pub trait ItemRepo: Send + Sync {
    fn get_by_category(&self, category_id: i64) -> Result<Vec<Item>, AppError>;
    fn get_by_id(&self, id: i64) -> Result<Option<Item>, AppError>;
    fn get_path_and_type(&self, id: i64) -> Result<(String, String), AppError>;
    fn create(&self, category_id: i64, name: &str, item_type: &str, path: &str, arguments: Option<&str>, working_dir: Option<&str>) -> Result<Item, AppError>;
    fn update(&self, id: i64, name: Option<&str>, item_type: Option<&str>, path: Option<&str>, arguments: Option<&str>, working_dir: Option<&str>) -> Result<(), AppError>;
    fn delete(&self, id: i64) -> Result<(), AppError>;
    fn move_to_category(&self, id: i64, category_id: i64) -> Result<(), AppError>;
    fn reorder(&self, orders: &[(i64, i32)]) -> Result<(), AppError>;
    fn toggle_pin(&self, id: i64) -> Result<(), AppError>;
    fn batch_delete(&self, ids: &[i64]) -> Result<usize, AppError>;
    fn update_icon_path(&self, id: i64, icon_path: &str) -> Result<(), AppError>;
    fn find_id_by_path_and_category(&self, path: &str, category_id: i64, icon_null_only: bool) -> Result<Option<i64>, AppError>;
    fn exists_by_path_and_category(&self, path: &str, category_id: i64) -> Result<bool, AppError>;
    #[allow(clippy::too_many_arguments)]
    fn create_with_pinyin(&self, category_id: i64, name: &str, pinyin_name: &str, item_type: &str, path: &str, arguments: Option<&str>, working_dir: Option<&str>, sort_order: i32) -> Result<(), AppError>;
}
