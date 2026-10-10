//! The `rustal-ste` skill for Marley's agents (#725).
//!
//! The skill is Simplified Technical English with Rustal's glossary, verbs and message shapes. Its
//! files are a copy of Rusty's skill store (`agent_skills/rustal-ste/SOURCE.md`). While
//! `marley.rustal_ste_skill` is on:
//! - Zed's agent has it as a built-in skill, which asks no permission;
//! - Claude Code in Marley's local terminals and the Marley entry load it as a plugin folder under
//!   Marley's data directory, named by the files' digest and written read-only;
//! - Codex finds a copy in `$CODEX_HOME/skills`, marked as Marley's.
//!
//! Off, each goes, and the Codex copy is removed only when Marley's mark is in it.

use std::fs;
use std::io::{self, Write as _};
use std::path::{Path, PathBuf};

use gpui::{App, AppContext as _, TaskExt as _};
use marley_terminal::identity::{set_skill_plugin, skill_plugin};
use settings::SettingsStore;
use sha2::{Digest as _, Sha256};
use util::ResultExt as _;

/// The skill's name, as its SKILL.md names it.
const NAME: &str = "rustal-ste";

/// The skill's SKILL.md.
const SKILL_MD: &str = include_str!("../agent_skills/rustal-ste/SKILL.md");

/// The skill's files, by their path in the skill, in a fixed order the digest follows.
const FILES: [(&str, &str); 8] = [
    ("SKILL.md", SKILL_MD),
    (
        "LICENSE",
        include_str!("../agent_skills/rustal-ste/LICENSE"),
    ),
    (
        "SOURCE.md",
        include_str!("../agent_skills/rustal-ste/SOURCE.md"),
    ),
    (
        "references/glossary.md",
        include_str!("../agent_skills/rustal-ste/references/glossary.md"),
    ),
    (
        "references/message-shapes.md",
        include_str!("../agent_skills/rustal-ste/references/message-shapes.md"),
    ),
    (
        "references/writing-rules.md",
        include_str!("../agent_skills/rustal-ste/references/writing-rules.md"),
    ),
    (
        "scripts/ste-lint.py",
        include_str!("../agent_skills/rustal-ste/scripts/ste-lint.py"),
    ),
    (
        "scripts/rustal-terms.json",
        include_str!("../agent_skills/rustal-ste/scripts/rustal-terms.json"),
    ),
];

/// The file in Marley's Codex copy that says Marley wrote it, holding the files' digest.
const MARK: &str = ".marley-owned";

/// Whether `marley.rustal_ste_skill` is on; on unless the user turned it off.
fn wanted(cx: &App) -> bool {
    cx.global::<SettingsStore>()
        .merged_settings()
        .marley
        .as_ref()
        .and_then(|marley| marley.rustal_ste_skill)
        != Some(false)
}

/// Follows the setting; [`crate::init`] calls it once.
pub fn init(cx: &mut App) {
    reconcile(cx);
    cx.observe_global::<SettingsStore>(reconcile).detach();
}

/// The plugin folder Claude Code loads the skill from while the setting is on, whether or not it
/// is written yet: its path is the files' digest under Marley's data directory.
pub(crate) fn plugin_folder(cx: &App) -> Option<PathBuf> {
    wanted(cx).then(|| plugin_root_in(paths::data_dir()).join(digest()))
}

fn plugin_root_in(data_dir: &Path) -> PathBuf {
    data_dir.join("claude-code").join(NAME)
}

/// Codex's skills folder: `$CODEX_HOME/skills`, `~/.codex/skills` when it is unset.
fn codex_skills_dir() -> PathBuf {
    std::env::var_os("CODEX_HOME")
        .filter(|home| !home.is_empty())
        .map_or_else(|| paths::home_dir().join(".codex"), PathBuf::from)
        .join("skills")
}

/// Gives each agent the skill, or takes it away, as the setting says.
fn reconcile(cx: &mut App) {
    let codex = codex_skills_dir();
    if !wanted(cx) {
        agent_skills::unregister_builtin_skill(NAME);
        set_skill_plugin(None);
        cx.background_spawn(futures::future::lazy(move |_| remove_codex_copy_in(&codex)))
            .detach_and_log_err(cx);
        return;
    }
    agent_skills::register_builtin_skill(NAME, SKILL_MD);
    if skill_plugin().is_some() {
        return;
    }
    let root = plugin_root_in(paths::data_dir());
    let written = cx.background_spawn(futures::future::lazy(move |_| {
        write_codex_copy_in(&codex).log_err();
        install_plugin_in(&root)
    }));
    cx.spawn(async move |cx| {
        let Some(folder) = written.await.log_err() else {
            return;
        };
        // The setting may have changed while the folder was written.
        cx.update(|cx| {
            if wanted(cx) {
                set_skill_plugin(Some(folder));
            }
        });
    })
    .detach();
}

/// The files' digest: each file's path, then its text, each preceded by its length as a
/// little-endian u64, in [`FILES`]' order.
fn digest() -> String {
    let mut hash = Sha256::new();
    for (path, text) in FILES {
        hash.update((path.len() as u64).to_le_bytes());
        hash.update(path);
        hash.update((text.len() as u64).to_le_bytes());
        hash.update(text);
    }
    format!("{:x}", hash.finalize())
}

/// The plugin folder under `root`, named by the digest: a plugin manifest and the skill, checked
/// when it is there, else written read-only by staging it beside and renaming it into place.
///
/// # Errors
///
/// When the folder can't be written, or one that is there no longer holds the skill's files.
fn install_plugin_in(root: &Path) -> io::Result<PathBuf> {
    let folder = root.join(digest());
    if folder.exists() {
        check_in(&folder.join("skills").join(NAME))?;
        return Ok(folder);
    }
    make_dir(root)?;
    let staged = root.join(format!(".staged-{}", uuid::Uuid::new_v4()));
    make_dir(&staged.join(".claude-plugin"))?;
    write_read_only(
        &staged.join(".claude-plugin").join("plugin.json"),
        &serde_json::json!({
            "name": NAME,
            "description": "Simplified Technical English with Rustal's glossary, verbs and message shapes, for what one agent writes to another (Marley #725)",
            "version": "1.0.0",
        })
        .to_string(),
    )?;
    write_skill_in(&staged.join("skills").join(NAME))?;
    if let Err(error) = fs::rename(&staged, &folder) {
        // Another Marley may have put it there first.
        fs::remove_dir_all(&staged).log_err();
        if !folder.exists() {
            return Err(error);
        }
    }
    check_in(&folder.join("skills").join(NAME))?;
    Ok(folder)
}

/// Writes Marley's copy of the skill into Codex's skills folder `skills`, with its mark: when
/// there is no `rustal-ste` folder there, or Marley's own of another digest. A folder Marley did not
/// write is left alone.
///
/// # Errors
///
/// When the copy can't be written.
fn write_codex_copy_in(skills: &Path) -> io::Result<()> {
    let folder = skills.join(NAME);
    match fs::read_to_string(folder.join(MARK)) {
        Ok(mark) if mark.trim() == digest() => return Ok(()),
        Ok(_) => fs::remove_dir_all(&folder)?,
        Err(_) if folder.exists() => {
            log::info!(
                "{} is not Marley's, so Marley leaves it as it is",
                folder.display()
            );
            return Ok(());
        }
        Err(_) => {}
    }
    make_dir(skills)?;
    let staged = skills.join(format!(".{NAME}-staged-{}", uuid::Uuid::new_v4()));
    write_skill_in(&staged)?;
    write_read_only(&staged.join(MARK), &digest())?;
    fs::rename(&staged, &folder).inspect_err(|_| {
        fs::remove_dir_all(&staged).log_err();
    })
}

/// Removes Marley's copy of the skill from Codex's skills folder `skills`, only when Marley's mark
/// is in it.
///
/// # Errors
///
/// When Marley's copy can't be removed.
fn remove_codex_copy_in(skills: &Path) -> io::Result<()> {
    let folder = skills.join(NAME);
    if folder.join(MARK).exists() {
        fs::remove_dir_all(&folder)?;
    }
    Ok(())
}

/// Writes the skill's files into `folder`, each read-only.
fn write_skill_in(folder: &Path) -> io::Result<()> {
    for (path, text) in FILES {
        let file = folder.join(path);
        if let Some(parent) = file.parent() {
            make_dir(parent)?;
        }
        write_read_only(&file, text)?;
    }
    Ok(())
}

/// Whether `folder` still holds exactly the skill's files.
fn check_in(folder: &Path) -> io::Result<()> {
    for (path, text) in FILES {
        if fs::read_to_string(folder.join(path))? != text {
            return Err(io::Error::other(format!(
                "the rustal-ste skill's {path} in {} was altered, so Marley doesn't load it; remove \
                 the folder to have Marley write it again",
                folder.display()
            )));
        }
    }
    Ok(())
}

/// Makes `dir`, readable by its owner alone, when it is missing.
fn make_dir(dir: &Path) -> io::Result<()> {
    let mut builder = fs::DirBuilder::new();
    builder.recursive(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::DirBuilderExt as _;
        builder.mode(0o700);
    }
    builder.create(dir)
}

/// Writes `text` to a new `file` its owner can only read.
fn write_read_only(file: &Path, text: &str) -> io::Result<()> {
    let mut options = fs::OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt as _;
        options.mode(0o400);
    }
    let mut written = options.open(file)?;
    written.write_all(text.as_bytes())?;
    written.sync_all()
}
