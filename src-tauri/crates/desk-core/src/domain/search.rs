use crate::error::AppError;
use crate::domain::item::Item;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SearchResult {
    pub item: Item,
    pub category_name: String,
    pub match_type: String,
    pub score: f64,
}

pub trait SearchPort: Send + Sync {
    fn search(&self, query: &str) -> Result<Vec<SearchResult>, AppError>;
}
