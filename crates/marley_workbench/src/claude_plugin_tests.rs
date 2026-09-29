//! Tests for Marley's plugin for Claude Code: its files, the lists Claude Code keeps, and the
//! hook's answer.

use std::path::Path;

use gpui::TestAppContext;

use super::*;

fn read_json(path: &Path) -> serde_json::Value {
    let text = std::fs::read_to_string(path).expect("the file");
    serde_json::from_str(&text).expect("valid JSON")
}

#[test]
fn the_plugin_is_a_marketplace_a_plugin_and_an_executable_hook() {
    let dir = tempfile::tempdir().expect("a scratch directory");
    write_plugin_in(dir.path()).expect("written");
    // Written again over itself, as a second install does.
    write_plugin_in(dir.path()).expect("written again");

    let marketplace = read_json(&dir.path().join(".claude-plugin/marketplace.json"));
    assert_eq!(marketplace["name"], MARKETPLACE);
    assert_eq!(marketplace["plugins"][0]["name"], "marley");
    assert_eq!(marketplace["plugins"][0]["source"], "./marley");
    let plugin = read_json(&dir.path().join("marley/.claude-plugin/plugin.json"));
    assert_eq!(plugin["name"], "marley");

    // Marley's banners come from the hook events alone, so no fixed sentence rides beside them
    // (#538).
    let hooks = read_json(&dir.path().join("marley/hooks/hooks.json"));
    assert!(hooks["hooks"]["Notification"].is_null());
    let stop = &hooks["hooks"]["Stop"][0]["hooks"][0]["command"];
    assert_eq!(stop, "\"${CLAUDE_PLUGIN_ROOT}/hooks/event.py\"");

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt as _;
        let mode = std::fs::metadata(dir.path().join("marley/hooks/event.py"))
            .expect("the hook")
            .permissions()
            .mode();
        assert_eq!(mode & 0o111, 0o111, "{mode:o}");
    }
}

#[test]
fn claude_codes_lists_say_what_is_installed_and_known() {
    let config = tempfile::tempdir().expect("a scratch directory");
    let plugins = config.path().join("plugins");
    // Nothing written yet, as on a first run.
    assert!(!installed_in(config.path()));
    assert!(!marketplace_known_in(config.path()));

    std::fs::create_dir_all(&plugins).expect("the directory");
    std::fs::write(
        plugins.join("installed_plugins.json"),
        r#"{"version": 2, "plugins": {"other@elsewhere": []}}"#,
    )
    .expect("written");
    std::fs::write(plugins.join("known_marketplaces.json"), "not JSON").expect("written");
    assert!(!installed_in(config.path()));
    assert!(!marketplace_known_in(config.path()));

    std::fs::write(
        plugins.join("installed_plugins.json"),
        r#"{"version": 2, "plugins": {"marley@marley": []}}"#,
    )
    .expect("written");
    std::fs::write(
        plugins.join("known_marketplaces.json"),
        r#"{"marley": {"source": {"source": "directory"}}}"#,
    )
    .expect("written");
    assert!(installed_in(config.path()));
    assert!(marketplace_known_in(config.path()));
}

#[gpui::test]
fn init_uses_marleys_data_directory_and_claude_codes_own(cx: &TestAppContext) {
    cx.update(init);
    let plugin = cx.update(|cx| cx.global::<ClaudePlugin>().clone());
    assert_eq!(
        plugin.marketplace_dir,
        paths::data_dir().join("claude-code")
    );
    let expected = std::env::var_os("CLAUDE_CONFIG_DIR")
        .map_or_else(|| util::paths::home_dir().join(".claude"), PathBuf::from);
    assert_eq!(plugin.config_dir, expected);
    // `claude` is looked for on the PATH with Claude Code's list, off the main thread.
    cx.run_until_parked();
    let found = cx.update(|cx| cx.global::<ClaudePlugin>().claude.clone());
    assert_eq!(found, which::which("claude").ok());
}
