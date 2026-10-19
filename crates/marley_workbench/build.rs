//! Compiles the crate again when a file the docs bundle embeds changes, in a release build (#815).
//!
//! Without `debug_assertions`, `util::fs_embed!` in `docs_tools.rs` embeds these files through
//! rust-embed, whose compressed embed names no file to cargo, so a new or edited page stayed out
//! of a release build until the crate compiled for some other reason. A debug build reads the
//! checkout at runtime and needs no watch. The paths are `DocsBundle`'s and `RootBundle`'s.

use std::io::Write as _;

fn main() -> std::io::Result<()> {
    let mut stdout = std::io::stdout().lock();
    writeln!(stdout, "cargo:rerun-if-changed=build.rs")?;
    if std::env::var_os("CARGO_CFG_DEBUG_ASSERTIONS").is_none() {
        for path in [
            "../../docs/src",
            "../../docs/marley/guide.md",
            "../../docs/marley/walkthrough.md",
            "../../CHANGELOG.md",
        ] {
            writeln!(stdout, "cargo:rerun-if-changed={path}")?;
        }
    }
    Ok(())
}
