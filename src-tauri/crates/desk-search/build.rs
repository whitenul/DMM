const COMMANDS: &[&str] = &["search_items"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
