use crate::error::AppError;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Category {
    pub id: i64,
    pub name: String,
    pub parent_id: Option<i64>,
    pub sort_order: i32,
    pub icon: Option<String>,
    pub folder_path: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

/// 分类仓库 trait
pub trait CategoryRepo: Send + Sync {
    fn get_all(&self) -> Result<Vec<Category>, AppError>;
    fn get_by_id(&self, id: i64) -> Result<Option<Category>, AppError>;
    fn create(&self, name: &str, parent_id: Option<i64>, icon: Option<&str>) -> Result<Category, AppError>;
    fn update(&self, id: i64, name: Option<&str>, icon: Option<&str>, parent_id: Option<Option<i64>>, folder_path: Option<&str>) -> Result<(), AppError>;
    fn delete(&self, id: i64) -> Result<(), AppError>;
    fn reorder(&self, orders: &[(i64, i32)]) -> Result<(), AppError>;
    fn link_folder(&self, id: i64, folder_path: &str) -> Result<(), AppError>;
    fn unlink_folder(&self, id: i64) -> Result<(), AppError>;
}
