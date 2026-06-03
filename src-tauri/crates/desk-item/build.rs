const COMMANDS: &[&str] = &[
    "get_items_by_category",
    "create_item",
    "update_item",
    "launch_item",
    "move_item",
    "reorder_items",
    "delete_item",
    "toggle_pin_item",
    "batch_delete_items",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
