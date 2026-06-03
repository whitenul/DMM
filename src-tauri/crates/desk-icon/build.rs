const COMMANDS: &[&str] = &["extract_icon_for_item", "get_item_icon_base64"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
