// Marley: without `debug_assertions`, `util::fs_embed!` embeds `assets/` through rust-embed,
// whose compressed embed names no file to cargo, so a new or edited asset stayed out of a release
// build until this crate compiled for some other reason (#815). Watching the folder compiles it
// again when anything under it changes. A debug build reads the checkout at runtime instead, and a
// watch there would recompile everything after this crate on every icon or theme edit.
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        println!("cargo:rerun-if-changed=../../assets");
    }
}
