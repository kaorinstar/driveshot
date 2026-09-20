// Generates the code Tauri needs from tauri.conf.json and the capability files beside it. It has
// to run before the crate is compiled, which is what a build script is for.
fn main() {
    tauri_build::build();
}
