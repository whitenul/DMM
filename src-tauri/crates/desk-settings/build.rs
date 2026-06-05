const COMMANDS: &[&str] = &[
    "load_settings",
    "update_settings",
    "save_window_position",
    "get_system_accent_color",
    "list_custom_themes",
    "save_custom_theme",
    "delete_custom_theme",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
