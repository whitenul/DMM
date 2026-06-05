pub mod connection;
pub mod migration_aggregator;
pub mod migrations;
pub mod path_resolver;
pub mod repos;
pub mod schema_inspector;

pub use connection::{DbState, init_db};
pub use migration_aggregator::{MigrationAggregator, MigrationSource};
pub use migrations::DeskCoreMigrations;
pub use path_resolver::{
    AppDataPath, resolve_app_data_dir, resolve_config_path, resolve_db_path, resolve_icons_dir,
    resolve_logs_dir,
};
pub use repos::{SqliteCategoryRepo, SqliteItemRepo, SqliteSearchRepo, compute_pinyin};
pub use schema_inspector::SchemaInspector;
