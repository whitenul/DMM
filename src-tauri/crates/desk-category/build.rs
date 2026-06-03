const COMMANDS: &[&str] = &[
    "get_categories",
    "create_category",
    "update_category",
    "delete_category",
    "reorder_categories",
    "link_folder",
    "unlink_folder",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
