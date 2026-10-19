// Marley: the release watch `crates/assets/build.rs` explains (#815), over the folders
// `SettingsAssets` embeds.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        println!("cargo:rerun-if-changed=../../assets/settings");
        println!("cargo:rerun-if-changed=../../assets/keymaps");
    }
}
