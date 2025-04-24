const COMMANDS: &[&str] = &["track", "identify", "alias", "opt_in", "opt_out", "is_opted_out"];

fn main() {
  tauri_plugin::Builder::new(COMMANDS)
    .android_path("android")
    .ios_path("ios")
    .build();
}
