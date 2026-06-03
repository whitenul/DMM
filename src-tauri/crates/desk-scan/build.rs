const COMMANDS: &[&str] = &[
    "scan_start_menu",
    "scan_uwp_apps",
    "scan_folder",
    "import_scanned_apps",
    "auto_scan_on_start",
];

fn main() {
    tauri_plugin::Builder::new(COMMANDS).build();
}
