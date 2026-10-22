//! Viewed marks in the project diff (#731).
//!
//! Each file's header in the project diff gets a Viewed tick at its end. Ticking folds the file and
//! keeps a mark: the file's absolute path with a sha256 of its diff's base text and its current
//! text, in the key-value store, so the file comes back ticked and folded after a restart. When the
//! file's diff changes (an agent's write, a save, a commit that moves its base), the fingerprint no
//! longer matches: the mark goes and the file unfolds. Unticking unfolds and forgets.

use std::collections::HashMap;

use db::kvp::KeyValueStore;
use editor::{Addon, Editor, EditorEvent};
use git_ui::project_diff::ProjectDiff;
use gpui::{AnyElement, App, AppContext as _, Context, Entity, Subscription, WeakEntity, Window};
use language::{BufferId, BufferSnapshot};
use multi_buffer::ExcerptBoundaryInfo;
use sha2::{Digest as _, Sha256};
use ui::{Checkbox, ToggleState, prelude::*};
use util::ResultExt as _;

/// The key-value store's namespace for the marks.
const STORE: &str = "marley-viewed";

/// Gives every project diff its Viewed ticks; `review_notes::init` calls it once.
pub fn init(cx: &App) {
    cx.observe_new(|diff: &mut ProjectDiff, _, cx: &mut Context<ProjectDiff>| {
        let editor = diff.editor(cx).read(cx).rhs_editor().clone();
        let marks = cx.new(|cx| ViewedMarks::new(&editor, cx));
        editor.update(cx, |editor, _| {
            editor.register_addon(ViewedAddon { marks });
        });
    })
    .detach();
}

/// The marks of one project diff's files.
struct ViewedMarks {
    editor: WeakEntity<Editor>,
    /// The files ticked, by buffer, with the fingerprint each was ticked at.
    ticked: HashMap<BufferId, String>,
    /// Each file's diff, followed for changes.
    watched: HashMap<BufferId, Subscription>,
    _editor_events: Subscription,
}

impl ViewedMarks {
    fn new(editor: &Entity<Editor>, cx: &mut Context<Self>) -> Self {
        let editor_events =
            cx.subscribe(
                editor,
                |marks, editor, event: &EditorEvent, cx| match event {
                    EditorEvent::BufferRangesUpdated { buffer, .. } => {
                        let id = buffer.read(cx).remote_id();
                        marks.watch(&editor, id, cx);
                    }
                    EditorEvent::BuffersRemoved { removed_buffer_ids } => {
                        for id in removed_buffer_ids {
                            marks.watched.remove(id);
                            marks.ticked.remove(id);
                        }
                    }
                    _ => {}
                },
            );
        let mut marks = Self {
            editor: editor.downgrade(),
            ticked: HashMap::new(),
            watched: HashMap::new(),
            _editor_events: editor_events,
        };
        let ids: Vec<BufferId> = editor
            .read(cx)
            .buffer()
            .read(cx)
            .all_buffers()
            .iter()
            .map(|buffer| buffer.read(cx).remote_id())
            .collect();
        for id in ids {
            marks.watch(editor, id, cx);
        }
        marks
    }

    /// Follows the diff of the file `id` once it has one, and checks its mark.
    fn watch(&mut self, editor: &Entity<Editor>, id: BufferId, cx: &mut Context<Self>) {
        if !self.watched.contains_key(&id)
            && let Some(diff) = editor.read(cx).buffer().read(cx).diff_for(id)
        {
            let subscription = cx.subscribe(&diff, move |marks, _, _, cx| marks.check(id, cx));
            self.watched.insert(id, subscription);
        }
        self.check(id, cx);
    }

    /// Brings the file `id`'s tick in line with its diff: a tick whose fingerprint no longer
    /// matches goes, and the file unfolds; a file not ticked here comes back ticked and folded when
    /// the stored mark matches, and a stale stored mark is forgotten.
    fn check(&mut self, id: BufferId, cx: &mut Context<Self>) {
        let Some((path, fingerprint)) = self.fingerprint(id, cx) else {
            return;
        };
        if let Some(marked) = self.ticked.get(&id) {
            if *marked != fingerprint {
                self.ticked.remove(&id);
                forget(path, cx);
                self.fold(id, false, cx);
            }
            return;
        }
        let stored = KeyValueStore::global(cx)
            .scoped(STORE)
            .read(&path)
            .log_err()
            .flatten();
        match stored {
            Some(stored) if stored == fingerprint => {
                self.ticked.insert(id, fingerprint);
                self.fold(id, true, cx);
            }
            Some(_) => forget(path, cx),
            None => {}
        }
    }

    /// Ticks or unticks the file `id`, from its header.
    fn set(&mut self, id: BufferId, viewed: bool, cx: &mut Context<Self>) {
        let Some((path, fingerprint)) = self.fingerprint(id, cx) else {
            return;
        };
        if viewed {
            let store = KeyValueStore::global(cx);
            let value = fingerprint.clone();
            cx.background_spawn(async move { store.scoped(STORE).write(path, value).await })
                .detach_and_log_err(cx);
            self.ticked.insert(id, fingerprint);
        } else {
            self.ticked.remove(&id);
            forget(path, cx);
        }
        self.fold(id, viewed, cx);
    }

    /// The file `id`'s absolute path, and a sha256 of its diff's base text and its current text;
    /// none for a file not on this machine.
    fn fingerprint(&self, id: BufferId, cx: &App) -> Option<(String, String)> {
        let editor = self.editor.upgrade()?;
        let multibuffer = editor.read(cx).buffer().read(cx);
        let buffer = multibuffer.buffer(id)?;
        let buffer = buffer.read(cx);
        let path = buffer.file()?.as_local()?.abs_path(cx);
        let base = multibuffer
            .diff_for(id)
            .and_then(|diff| diff.read(cx).base_text_string(cx))
            .unwrap_or_default();
        let mut hasher = Sha256::new();
        hasher.update(base.as_bytes());
        hasher.update([0]);
        hasher.update(buffer.text().as_bytes());
        Some((
            path.to_string_lossy().into_owned(),
            format!("{:x}", hasher.finalize()),
        ))
    }

    fn fold(&self, id: BufferId, folded: bool, cx: &mut Context<Self>) {
        if let Some(editor) = self.editor.upgrade() {
            editor.update(cx, |editor, cx| {
                if folded {
                    editor.fold_buffer(id, cx);
                } else {
                    editor.unfold_buffer(id, cx);
                }
            });
        }
        cx.notify();
    }
}

/// How many files of the project diff whose editor is `editor` are ticked Viewed (#893).
pub(crate) fn viewed(editor: &Editor, cx: &App) -> usize {
    editor
        .addon::<ViewedAddon>()
        .map_or(0, |addon| addon.marks.read(cx).ticked.len())
}

/// Drops the stored mark of `path`.
fn forget(path: String, cx: &App) {
    let store = KeyValueStore::global(cx);
    cx.background_spawn(async move { store.scoped(STORE).delete(path).await })
        .detach_and_log_err(cx);
}

/// The Viewed tick at the end of each file's header in a project diff.
struct ViewedAddon {
    marks: Entity<ViewedMarks>,
}

impl Addon for ViewedAddon {
    fn render_buffer_header_trailing_controls(
        &self,
        _: &ExcerptBoundaryInfo,
        buffer: &BufferSnapshot,
        _: &Window,
        cx: &App,
    ) -> Option<AnyElement> {
        buffer.file()?.as_local()?;
        let id = buffer.remote_id();
        let ticked = self.marks.read(cx).ticked.contains_key(&id);
        let marks = self.marks.clone();
        Some(
            Checkbox::new(("marley-viewed", id.to_proto()), ToggleState::from(ticked))
                .label("Viewed")
                .label_size(LabelSize::Small)
                .on_click(move |state: &ToggleState, _, cx| {
                    // The header folds on a click of its own.
                    cx.stop_propagation();
                    let viewed = state.selected();
                    marks.update(cx, |marks, cx| marks.set(id, viewed, cx));
                })
                .into_any_element(),
        )
    }

    fn to_any(&self) -> &dyn std::any::Any {
        self
    }
}
