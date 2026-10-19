// Marley: the release watch `crates/assets/build.rs` explains (#815), over the folder
// `GrammarDir` embeds.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        println!("cargo:rerun-if-changed=src");
    }
}
