//! Following Omarchy's theme (#759), behind `marley.omarchy_theme`, off by default.
//!
//! Omarchy renders a Zed theme from Marley's template (`marley.json.tpl` in its user templates
//! folder) at each theme switch, and Marley's hook in `theme-set.d` copies the render into Marley's
//! themes folder as `omarchy.json`, where Zed's theme watcher reloads it. Turning the setting on
//! writes both, runs `omarchy-theme-refresh` once when the current theme has no render yet, loads
//! the copy and selects the "Omarchy" theme, keeping the selection it replaced; off puts that back
//! while "Omarchy" is still selected and removes what Marley wrote. Marley owns the template while
//! its first line is Marley's marker, so an edited template, the marker deleted, stays. Without
//! Omarchy the setting does nothing.

use std::os::unix::fs::PermissionsExt as _;
use std::path::{Path, PathBuf};

use anyhow::Context as _;
use db::kvp::KeyValueStore;
use fs::Fs;
use gpui::{App, AppContext as _, AsyncApp, Global, TaskExt as _};
use settings::{SettingsStore, ThemeName, ThemeSelection};
use theme::ThemeRegistry;
use util::ResultExt as _;
use workspace::Toast;
use workspace::notifications::NotificationId;

/// The template Omarchy renders, Marley's while its first line starts with [`MARKER`].
const TEMPLATE: &str = include_str!("../omarchy/marley.json.tpl");
/// The hook Omarchy runs after a theme switch, with `@RENDER@` and `@COPY@` for its two paths.
const HOOK: &str = include_str!("../omarchy/theme-set-hook.sh");
/// How the template's first line starts while the file is Marley's.
const MARKER: &str = "// Marley's Omarchy theme (marley.omarchy_theme).";
/// The theme the template names, as the settings select it.
const THEME_NAME: &str = "Omarchy";
/// The copy's name in Marley's themes folder.
const COPY: &str = "omarchy.json";
/// Where the selection the switch replaced is kept.
const SCOPE: &str = "marley-omarchy-theme";
const KEPT: &str = "replaced-theme";

/// Whether the user turned the setting on.
fn wanted(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.omarchy_theme)
        == Some(true)
}

/// Where Omarchy keeps what this touches, under `MARLEY_OMARCHY_HOME`, else the home folder:
/// Omarchy's own scripts use `$HOME/.config/omarchy` and `$HOME/.local/state/omarchy`.
#[derive(Debug, Clone)]
struct OmarchyPaths {
    home: PathBuf,
}

impl OmarchyPaths {
    fn from_env() -> Self {
        let home = std::env::var_os("MARLEY_OMARCHY_HOME")
            .filter(|home| !home.is_empty())
            .map_or_else(|| util::paths::home_dir().clone(), PathBuf::from);
        Self { home }
    }

    fn config(&self) -> PathBuf {
        self.home.join(".config/omarchy")
    }

    fn template(&self) -> PathBuf {
        self.config().join("themed/marley.json.tpl")
    }

    fn hook(&self) -> PathBuf {
        self.config().join("hooks/theme-set.d/marley-theme")
    }

    fn current_theme(&self) -> PathBuf {
        self.home.join(".local/state/omarchy/current/theme")
    }

    fn render(&self) -> PathBuf {
        self.current_theme().join("marley.json")
    }

    /// Whether Omarchy is here: a current theme with its colours, and Omarchy's config folder.
    fn present(&self) -> bool {
        self.current_theme().join("colors.toml").is_file() && self.config().is_dir()
    }
}

/// The program that re-applies Omarchy's current theme: `MARLEY_OMARCHY_REFRESH`, else
/// `omarchy-theme-refresh` on the PATH.
fn refresh_program() -> PathBuf {
    std::env::var_os("MARLEY_OMARCHY_REFRESH")
        .filter(|program| !program.is_empty())
        .map_or_else(|| PathBuf::from("omarchy-theme-refresh"), PathBuf::from)
}

/// Whether the file at `path` is Marley's template: there is none, or its first line is the marker.
fn template_is_marleys(path: &Path) -> bool {
    match std::fs::read_to_string(path) {
        Ok(text) => text.starts_with(MARKER),
        Err(error) => error.kind() == std::io::ErrorKind::NotFound,
    }
}

/// Writes the template while it is Marley's and the hook, and copies a render newer than the
/// copy into `themes_dir`; whether a render was there.
fn install_in(paths: &OmarchyPaths, themes_dir: &Path) -> anyhow::Result<bool> {
    let template = paths.template();
    if template_is_marleys(&template) {
        write_file(&template, TEMPLATE)?;
    }
    let quote = |path: &Path| -> anyhow::Result<String> {
        shlex::try_quote(&path.to_string_lossy())
            .map(std::borrow::Cow::into_owned)
            .with_context(|| format!("{} cannot be named in a shell", path.display()))
    };
    let hook = HOOK
        .replace("@RENDER@", &quote(&paths.render())?)
        .replace("@COPY@", &quote(&themes_dir.join(COPY))?);
    let hook_path = paths.hook();
    write_file(&hook_path, &hook)?;
    std::fs::set_permissions(&hook_path, std::fs::Permissions::from_mode(0o755))
        .with_context(|| format!("making {} runnable", hook_path.display()))?;
    let render = paths.render();
    if !render.is_file() {
        return Ok(false);
    }
    copy_render(&render, &themes_dir.join(COPY))?;
    Ok(true)
}

/// Copies `render` to `copy` through a temporary file and a rename, when it is newer than the copy
/// and has no placeholder left, as the hook does.
fn copy_render(render: &Path, copy: &Path) -> anyhow::Result<()> {
    let modified = |path: &Path| std::fs::metadata(path).and_then(|metadata| metadata.modified());
    if let (Ok(rendered), Ok(copied)) = (modified(render), modified(copy))
        && rendered <= copied
    {
        return Ok(());
    }
    let text =
        std::fs::read_to_string(render).with_context(|| format!("reading {}", render.display()))?;
    if text.contains("{{") {
        return Ok(());
    }
    write_file(copy, &text)
}

/// Writes `text` to `path` through a temporary file beside it and a rename, the folder made.
fn write_file(path: &Path, text: &str) -> anyhow::Result<()> {
    let folder = path
        .parent()
        .with_context(|| format!("{} has no folder", path.display()))?;
    std::fs::create_dir_all(folder).with_context(|| format!("making {}", folder.display()))?;
    let temporary = folder.join(format!(".marley-{}.tmp", uuid::Uuid::new_v4().simple()));
    std::fs::write(&temporary, text).with_context(|| format!("writing {}", temporary.display()))?;
    std::fs::rename(&temporary, path).with_context(|| format!("writing {}", path.display()))
}

/// Removes the hook and the copy, and the template while it is Marley's.
fn remove_in(paths: &OmarchyPaths, themes_dir: &Path) -> anyhow::Result<()> {
    let template = paths.template();
    let mut gone = vec![paths.hook(), themes_dir.join(COPY)];
    if template_is_marleys(&template) {
        gone.push(template);
    }
    for path in gone {
        match std::fs::remove_file(&path) {
            Ok(()) => {}
            Err(error) if error.kind() == std::io::ErrorKind::NotFound => {}
            Err(error) => {
                return Err(error).with_context(|| format!("removing {}", path.display()));
            }
        }
    }
    Ok(())
}

/// The state applied last, so a settings change that does not flip the switch costs a comparison.
#[derive(Default)]
struct OmarchyTheme {
    /// What was applied: none before the first reconcile, at start.
    applied: Option<bool>,
    /// Whether an apply runs; the reconcile after it catches a change made meanwhile.
    running: bool,
}

impl Global for OmarchyTheme {}

/// Follows the setting from start; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    cx.set_global(OmarchyTheme::default());
    reconcile(cx);
    cx.observe_global::<SettingsStore>(reconcile).detach();
}

fn reconcile(cx: &mut App) {
    let wanted = wanted(cx);
    let state = cx.global::<OmarchyTheme>();
    if state.running || state.applied == Some(wanted) {
        return;
    }
    let starting = state.applied.is_none();
    let state = cx.global_mut::<OmarchyTheme>();
    state.applied = Some(wanted);
    if !wanted && starting {
        return;
    }
    state.running = true;
    let paths = OmarchyPaths::from_env();
    let themes_dir = paths::themes_dir().clone();
    cx.spawn(async move |cx| {
        if wanted {
            turn_on(paths, themes_dir, starting, cx).await;
        } else {
            turn_off(paths, themes_dir, cx).await;
        }
        cx.update(|cx| {
            cx.global_mut::<OmarchyTheme>().running = false;
            reconcile(cx);
        });
    })
    .detach();
}

/// Writes the template and the hook; turned on now, not at start, also renders when no render is
/// there and selects the theme.
async fn turn_on(paths: OmarchyPaths, themes_dir: PathBuf, starting: bool, cx: &AsyncApp) {
    let installed = cx
        .background_spawn({
            let paths = paths.clone();
            let themes_dir = themes_dir.clone();
            futures::future::lazy(move |_| {
                if paths.present() {
                    install_in(&paths, &themes_dir).map(Some)
                } else {
                    Ok(None)
                }
            })
        })
        .await;
    let rendered = match installed {
        Ok(Some(rendered)) => rendered,
        Ok(None) => {
            log::info!(
                "omarchy theme: no Omarchy under {}, so nothing follows its theme",
                paths.home.display()
            );
            return;
        }
        Err(error) => {
            warn(
                &format!("Marley could not follow Omarchy's theme: {error:#}"),
                cx,
            );
            return;
        }
    };
    if starting {
        return;
    }
    // Omarchy renders the template and runs the hook, so the copy lands in the themes folder.
    if !rendered {
        let program = refresh_program();
        if let Err(error) = crate::process::run_program(&program, &[]).await {
            warn(
                &format!(
                    "{} did not re-apply Omarchy's theme: {error:#}",
                    program.display()
                ),
                cx,
            );
            return;
        }
    }
    // The switch may have been turned off while Omarchy rendered.
    if !cx.update(|cx| wanted(cx)) {
        return;
    }
    let copy = themes_dir.join(COPY);
    let bytes = cx
        .background_spawn(futures::future::lazy(move |_| std::fs::read(&copy)))
        .await;
    match bytes {
        Ok(bytes) => cx.update(|cx| select(&bytes, cx)),
        Err(error) => warn(
            &format!("Omarchy rendered no theme for Marley yet: {error}"),
            cx,
        ),
    }
}

/// Loads the copy and selects "Omarchy", keeping the selection it replaces (D4).
fn select(bytes: &[u8], cx: &mut App) {
    let registry = ThemeRegistry::global(cx);
    if let Err(error) = theme_settings::load_user_theme(&registry, bytes) {
        show_warning(
            &format!("Marley could not read Omarchy's theme: {error:#}"),
            cx,
        );
        return;
    }
    theme_settings::reload_theme(cx);
    let current = user_theme(cx);
    if !names_omarchy(current.as_ref()) {
        let kept = current.map_or_else(
            || Ok(String::new()),
            |selection| serde_json::to_string(&selection),
        );
        match kept {
            Ok(kept) => {
                let store = KeyValueStore::global(cx);
                cx.background_spawn(async move {
                    store.scoped(SCOPE).write(KEPT.to_string(), kept).await
                })
                .detach_and_log_err(cx);
            }
            Err(error) => log::warn!("omarchy theme: the replaced theme: {error}"),
        }
    }
    settings::update_settings_file(<dyn Fs>::global(cx), cx, |content, _| {
        content.theme.theme = Some(ThemeSelection::Static(ThemeName(THEME_NAME.into())));
    });
}

/// Puts back the kept selection while "Omarchy" is still selected, then removes what Marley wrote.
async fn turn_off(paths: OmarchyPaths, themes_dir: PathBuf, cx: &AsyncApp) {
    cx.update(|cx| {
        if !names_omarchy(user_theme(cx).as_ref()) {
            return;
        }
        let kept = KeyValueStore::global(cx)
            .scoped(SCOPE)
            .read(KEPT)
            .log_err()
            .flatten()
            .filter(|kept| !kept.is_empty())
            .and_then(|kept| serde_json::from_str::<ThemeSelection>(&kept).log_err());
        settings::update_settings_file(<dyn Fs>::global(cx), cx, move |content, _| {
            content.theme.theme = kept;
        });
    });
    let removed = cx
        .background_spawn(futures::future::lazy(move |_| {
            remove_in(&paths, &themes_dir)
        }))
        .await;
    if let Err(error) = removed {
        warn(
            &format!("Marley could not remove its Omarchy theme files: {error:#}"),
            cx,
        );
    }
}

/// The `theme` the user's settings file selects.
fn user_theme(cx: &App) -> Option<ThemeSelection> {
    cx.global::<SettingsStore>()
        .raw_user_settings()
        .and_then(|user| user.content.theme.theme.clone())
}

/// Whether `selection` names the "Omarchy" theme, statically or for either appearance.
fn names_omarchy(selection: Option<&ThemeSelection>) -> bool {
    match selection {
        Some(ThemeSelection::Static(name)) => name.0.as_ref() == THEME_NAME,
        Some(ThemeSelection::Dynamic { light, dark, .. }) => {
            light.0.as_ref() == THEME_NAME || dark.0.as_ref() == THEME_NAME
        }
        None => false,
    }
}

fn warn(message: &str, cx: &AsyncApp) {
    cx.update(|cx| show_warning(message, cx));
}

/// A toast in every workspace, and the log.
fn show_warning(message: &str, cx: &mut App) {
    log::warn!("omarchy theme: {message}");
    for workspace in crate::mcp::all_workspaces(cx) {
        let toast = Toast::new(
            NotificationId::unique::<OmarchyTheme>(),
            message.to_string(),
        );
        workspace.update(cx, |workspace, cx| workspace.show_toast(toast, cx));
    }
}
