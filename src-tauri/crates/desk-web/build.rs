const COMMANDS: &[&str] = &["fetch_web_meta"];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
