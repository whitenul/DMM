const COMMANDS: &[&str] = &["load_settings", "update_settings", "save_window_position"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
