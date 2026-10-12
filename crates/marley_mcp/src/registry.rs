//! The tool-family REGISTRY (D4) — first-class `(family, verb)` typed data, ONE source of truth. Three
//! families: `fleet` (read) and `session` (write) from L1, and `terminal` (read, #491), the one the server
//! lists today. Adding `editor`/`browser` later is a new `Family` variant + a [`REGISTRY`] row + its
//! `tool_schemas`/`dispatch` arm — additive, no rework of permissions (REQ-011). PURE.

use std::collections::BTreeSet;

use crate::clients::{Principal, permits};
use crate::permission::Tier;
use serde_json::{Value, json};

/// A tool FAMILY — a CLOSED enum (D4). The public wire name keeps family + verb as separate typed fields;
/// the join is the single [`ToolSpec::name`] fn.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Family {
    /// The fleet read family.
    Fleet,
    /// The session write family (`surface_to_human`, and Layer-2 verbs later).
    Session,
    /// Marley's terminals and their blocks, read by the app (#491).
    Terminal,
    /// The page in Marley's Browser tab, seen and driven by the app (#492).
    Browser,
    /// The ports each project's processes listen on, read by the app (#521).
    Ports,
    /// Zed's open editors, listed, read and opened for an agent (#704).
    Editor,
    /// Files opened for an agent's own editor key and waited on, for Marley's `marley-edit`
    /// (#649); never listed, and Marley's own. Named `editor_*` until #704.
    Prompt,
    /// The Agent Panel's threads, listed, read, posted into and answered for an agent (#706).
    Thread,
    /// Zed's palette actions an agent may run, listed and run (#707).
    Action,
    /// Zed's docs and Marley's guide, as this build ships them, searched and read (#681).
    Docs,
    /// The settings' schema and the values each settings file gives (#681).
    Settings,
    /// The actions and the keys bound to them (#681).
    Actions,
    /// The user's key bindings (#686).
    Keymap,
    /// Seats on the harness Marley follows, set up through its own command (#692).
    Seat,
    /// Messages an agent drafts for the user to review and send (#769).
    Draft,
    /// Findings a tool puts into the project diff as review notes the user keeps or drops (#730).
    Review,
}

impl Family {
    /// The wire prefix for this family.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Fleet => "fleet",
            Self::Session => "session",
            Self::Terminal => "terminal",
            Self::Browser => "browser",
            Self::Ports => "ports",
            Self::Editor => "editor",
            Self::Prompt => "prompt",
            Self::Thread => "thread",
            Self::Action => "action",
            Self::Docs => "docs",
            Self::Settings => "settings",
            Self::Actions => "actions",
            Self::Keymap => "keymap",
            Self::Seat => "seat",
            Self::Draft => "draft",
            Self::Review => "review",
        }
    }

    /// Whether `tools/list` lists this family's tools. The fleet and session families wait for prong
    /// 2's C1 to feed them; a client that names one of their tools still reaches it.
    #[must_use]
    pub const fn is_served(self) -> bool {
        matches!(
            self,
            Self::Terminal
                | Self::Browser
                | Self::Ports
                | Self::Editor
                | Self::Thread
                | Self::Action
                | Self::Docs
                | Self::Settings
                | Self::Actions
                | Self::Keymap
                | Self::Seat
                | Self::Draft
                | Self::Review
        )
    }
}

/// One registry entry: a family + verb, its permission tier, the write grant-class it needs (empty for a
/// read tool), and a description.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ToolSpec {
    /// The tool's family.
    pub family: Family,
    /// The verb within the family.
    pub verb: &'static str,
    /// Read (loose) or Write (grant-gated).
    pub tier: Tier,
    /// The permission grant-class a WRITE tool requires (empty for read tools).
    pub grant_class: &'static str,
    /// One-line description (surfaced in `tools/list`).
    pub description: &'static str,
}

impl ToolSpec {
    /// The wire tool name `family_verb` (D4 — the one place family+verb are joined).
    #[must_use]
    pub fn name(&self) -> String {
        tool_name(self.family, self.verb)
    }
}

/// Compose a wire tool name from a family + verb (`fleet` + `snapshot` → `fleet_snapshot`). Claude
/// Code and the Anthropic API take tool names without dots (#491 D4).
#[must_use]
pub fn tool_name(family: Family, verb: &str) -> String {
    format!("{}_{}", family.as_str(), verb)
}

/// The L1 tool table (D4) as a CONST slice — the SINGLE source of truth. `tools_list` derives its names +
/// descriptions from this, so the wire list can never drift from what `lookup`/`dispatch` know (the D4
/// charter; the inspect-caught duplication is gone). Adding `editor`/`browser` = a new row here + its
/// `tool_schemas` arm + its `dispatch` arm.
const REGISTRY: &[ToolSpec] = &[
    ToolSpec {
        family: Family::Fleet,
        verb: "snapshot",
        tier: Tier::Read,
        grant_class: "",
        description: "Return the current fleet snapshot (all seats + their state).",
    },
    ToolSpec {
        family: Family::Session,
        verb: "surface_to_human",
        tier: Tier::Write,
        grant_class: "session.write",
        description: "Bring a session's surface into the human's view (focus it).",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List Marley's terminals: each one's id, its `terminal_id` (the \
                      `MARLEY_TERMINAL_ID` its programs see), title, project, working directory, \
                      the command running in it, how many blocks it holds, and `self` for the \
                      terminal this call comes from.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "blocks",
        tier: Tier::Read,
        grant_class: "",
        description: "List a terminal's blocks (the calling terminal's when `terminal` is left \
                      out), the commands run in it, oldest first: each command, \
                      whether the shell's own hook reported it, its exit code, working directory, \
                      start time and duration, whether it still runs, and whether its output is \
                      still in the scrollback. A block marked `restored` ran before Marley \
                      restarted: its output is the earlier session's, drawn again, and it is \
                      never the terminal's latest command. Secrets in commands come back as \
                      `[redacted: <kind>]`, counted in `redacted`, unless the user turned \
                      redaction off.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "Read one block's output as text, from the calling terminal when `terminal` is \
                      left out, a page at a time: the newest whole lines that fit in 12,000 \
                      bytes, numbered from the block's first line (`first_line`, `last_line`, \
                      `total_lines`). When `previous` is set, pass it as `before` to read the \
                      page before; a line longer than a page keeps its end (`line_cut`). Secrets \
                      come back as `[redacted: <kind>]`, counted over the whole output in \
                      `redacted`, unless the user turned redaction off. Refusals carry a `code`: \
                      `no_terminal`, `no_block`, `output_gone` or `bad_argument`.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "find",
        tier: Tier::Read,
        grant_class: "",
        description: "Find the line of a block's output that matches a query in words, such as \
                      \"where the server refused the connection\", from the calling terminal \
                      when `terminal` is left out: the line's number and up to three \
                      candidates. A line holding every word of the query answers at once; the \
                      System One model ranks what the words leave open. When it is not sure, \
                      read the block with terminal_read. Secrets come back as \
                      `[redacted: <kind>]`. Listed while the user turns it on \
                      (marley.system_one.uses.terminal_find).",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "screen",
        tier: Tier::Read,
        grant_class: "",
        description: "Read what a terminal's screen shows now, from the calling terminal when \
                      `terminal` is left out: its rows, the cursor, whether a full-screen \
                      program has the alternate screen, the program in the foreground (none \
                      while the shell waits at its prompt), and who controls it: `generation`, \
                      which terminal_type needs, `taken_over`, and whether the user approved \
                      writes to this program. Secrets come back as `[redacted: <kind>]`.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "type",
        tier: Tier::Write,
        grant_class: "terminal.write",
        description: "Type into the program running in a terminal's foreground (psql, a \
                      debugger, a REPL, a dev server's prompt): `text` as a paste, then each of \
                      `keys` by name (escape, ctrl-c, up, tab), then Enter when `submit` is \
                      true, at most 4,096 bytes. Give the `generation` terminal_screen gave; a \
                      write is refused when the program changed since, at the shell's prompt, \
                      into an agent CLI, while the user has taken over, or when the user denies \
                      it or does not answer within 25 seconds: Marley asks before the first \
                      write to each program unless the user chose otherwise.",
    },
    ToolSpec {
        family: Family::Terminal,
        verb: "run",
        tier: Tier::Write,
        grant_class: "terminal.write",
        description: "Run a command at a terminal's shell prompt, in the user's view, as a \
                      block: typed as the user would type it, then answered with the block's \
                      index, exit code, duration and output once it ends or `wait_seconds` \
                      (at most 20) pass, when it answers `running: true` and terminal_read \
                      reads the rest. The output is the newest page, as terminal_read gives it, \
                      with `previous` for the page before. Refused while a program or an agent CLI runs there, \
                      while the user has typed at the prompt, while the user has taken over, \
                      or when the user refuses: a command the user's denylist matches waits for \
                      Run or Refuse under the terminal, one the allowlist matches runs at once, \
                      and the user's setting decides the rest. One line, at most 4,096 bytes. \
                      Secrets in the output come back as `[redacted: <kind>]`.",
    },
    browser_read(
        "tabs",
        "List Marley's Browser tabs, one per page, across every project's browser (each project \
         has a Chromium and a profile of its own): each tab's id, which the other browser tools \
         take as `tab`, its title and URL, whether it loads, its project, whether the user \
         focused it last, and `default`, the one the tools act on when a call names no tab: the \
         one the user focused last in the project you run in.",
    ),
    browser_read(
        "look",
        "See a page in Marley's Browser tabs as the user sees it: its URL, title, viewport, \
         scroll, whether it loads, the focused element and the selection, and the frame on the \
         screen as an image.",
    ),
    browser_read(
        "snapshot",
        "A page's accessibility tree as text: its interactive elements (every node with \
         `full`), each with a ref for the write tools, cross-site iframes included.",
    ),
    browser_read(
        "find",
        "Find the element of the page that matches a query in words, such as \"the sign in \
         button\": its ref, which browser_click takes, and up to three candidates. An element \
         whose role and name hold every word of the query answers at once; the System One model \
         ranks what the words leave open. When it is not sure, read the page with \
         browser_snapshot. The refs are a new snapshot's, as browser_snapshot's are. Listed while \
         the user turns it on (marley.system_one.uses.browser_find).",
    ),
    browser_read(
        "console",
        "A page's latest console messages and uncaught errors, oldest first, at most 200, \
         with secrets as `[redacted: <kind>]` unless the user turned redaction off.",
    ),
    browser_read(
        "network",
        "A page's latest requests, oldest first, at most 200: method, URL with secret-looking \
         values hidden, type, status, duration and failure; no headers or bodies.",
    ),
    browser_read(
        "picks",
        "List the elements the user picked in the Browser tabs this session, oldest first: each \
         pick's id, its tab, the page's URL and title, what the element is, the user's caption, \
         and whether the user sent it to you. Secret-looking text is redacted.",
    ),
    browser_read(
        "pick",
        "Read an element the user picked in a Browser tab, by its id from the user's line or \
         browser_picks, as it was at the pick: its locators, the most durable first, with \
         whether each finds it alone; its role and name; the listeners on it and its ancestors \
         with their scripts, lines and columns, and, through each script's source map, the file \
         in the project and the line they were written at; what would block a click on it; its \
         box in the page; its HTML, without scripts, field values, secret-looking attribute values \
         or URL queries, at most 4,096 characters; sixteen of its computed styles; its siblings' \
         texts and the page's selection; on a React dev build, the components around it and the \
         file and line it was written at; the page around it as an image; and its latest \
         check from browser_check_pick, when one ran. Secret-looking text is redacted.",
    ),
    browser_read(
        "recordings",
        "List the recordings the user saved with Record this in a Browser tab: each one's id, \
         its tab, the page's URL and title, when it was saved, how many seconds it spans, and \
         how many frames and entries it holds.",
    ),
    browser_read(
        "recording",
        "Read a recording the user saved with Record this, the minute before it, by its id \
         from browser_recordings: what the user did (presses with their places, keys by name, \
         typing as counts, and each click, fill and key press with the target's locators; a \
         fill keeps an ordinary field's text, at most 1,000 characters, and only the fact of it \
         for a password or other secret field), what the agent did, the console, the requests \
         with secret-looking URL values hidden, navigations, accessibility snapshots, and the \
         frames; `frame` gives one frame, from 1, as an image. Secret-looking text is redacted.",
    ),
    browser_read(
        "draft_test",
        "Draft a Playwright test from a recording, by its id from browser_recordings, for the \
         project it was recorded in, writing nothing: the test replays each click, fill and key \
         press with the most durable locator that found its target alone at the event \
         (getByTestId, getByRole with the exact name, getByLabel, getByPlaceholder, getByText, \
         then a CSS path), sets baseURL from the first action's origin, and expects the URL \
         after each navigation an action caused. A password or other secret field's fill reads \
         an environment variable, which the test names when it is unset. Answers the test, a \
         path for it in the project (the testDir its playwright.config sets, else tests/), the \
         variables it reads, what it skipped and why, and the command that runs it. Write the \
         file with your own tools.",
    ),
    browser_read(
        "annotations",
        "List the boxes and notes drawn over a page in a Browser tab, by the user or an agent: \
         each one's id, its box in page coordinates (the document's CSS pixels, which stay on \
         the content as the page scrolls), its note, who drew it and when.",
    ),
    browser_write(
        "annotate",
        "Draw a box with a note over a page in a Browser tab, for the user to see: around an \
         element by its ref from browser_snapshot, scrolled into view, or over an area of the \
         viewport; it stays on that content as the page scrolls, marked as the agent's. \
         `clear` removes the agent's own boxes instead.",
    ),
    browser_write(
        "check_pick",
        "Check an element the user picked, by its id from the user's line or browser_picks, \
         after a change such as your fix: find it again in the pick's own tab by the most \
         durable locator that still finds anything (its test id, id, role and name, text, then \
         CSS path; of several matches, the one nearest its old box), scroll it into view when it \
         is off screen, crop it as the pick was cropped, and say what changed since the pick: \
         its box, its computed styles, its text, role and name, or its HTML. Answers whether and \
         by what it was found, the changes, the element as it is now, and the new crop as the \
         image. Secret-looking text is redacted. The user's tray shows the check's verdict.",
    ),
    browser_write(
        "navigate",
        "Load an http or https URL in a Browser tab, or in a new tab with `new_tab`; with no \
         `tab`, in the tab the user focused last in the project you run in, opening one there \
         when it has none. A new tab opens in your project's browser, with that project's \
         cookies and logins. Answers once the page has loaded, with the tab's id.",
    ),
    browser_write(
        "open_url",
        "Open a URL a program in `directory` asked to open, as Marley's `BROWSER` opener does \
         (#561): in a Browser tab of the project whose folder holds `directory`, with the focus, \
         when `marley.terminal_links` sends that URL to a Browser tab (a local http or https \
         URL, by default), bringing forward a tab of that project already on it. A local HTML \
         page, a `file:` URL or a path to a `.html` or `.htm` file, opens the same way (#586); a \
         folder or any other file does not. It never opens the system browser: `opened: false` \
         says why it opened nothing.",
    ),
    browser_write(
        "back",
        "Go back in a Browser tab's history; answers once the page has loaded.",
    ),
    browser_write(
        "click",
        "Click an element by its ref from browser_snapshot, or a point of the viewport, as the \
         user's mouse does.",
    ),
    browser_write(
        "type",
        "Type text as key presses into an element by its ref, or where the focus is; `submit` \
         presses Enter after.",
    ),
    browser_write(
        "press",
        "Press a key or a chord: Enter, Tab, Escape, ArrowDown, Ctrl+A, Shift+Tab.",
    ),
    browser_write(
        "scroll",
        "Scroll the page by pixels (dy down, dx right), or an element by its ref into view.",
    ),
    browser_write(
        "hover",
        "Move the user's mouse onto an element by its ref, or to a point of the viewport, and \
         leave it there, so the page shows what it shows on hover (#756).",
    ),
    browser_write(
        "select",
        "Choose options of a select list by its ref: `values` by their value, or `labels` by \
         the text shown, several for a list that takes several. The page gets input and change \
         as from the user's choice. An option not there is refused with the options there.",
    ),
    browser_write(
        "drag",
        "Drag from an element by its ref (`from_ref`) or a point (`from_x`, `from_y`) to another \
         (`to_ref`, or `to_x` and `to_y`), as the user's mouse does: press, move, release. A \
         drag the page starts with HTML drag and drop is finished with its drop events.",
    ),
    browser_write(
        "upload",
        "Set the files of a file input by its ref, or, with no ref, answer the file chooser the \
         page waits on (browser_tabs reports it): `paths`, absolute paths of files inside one of \
         the user's open projects; a file a secret glob matches is refused.",
    ),
    browser_write(
        "dialog",
        "Answer the alert, confirm, prompt or leave-page dialog open in a Browser tab: `accept` \
         as OK, or not as Cancel, with `text` for a prompt. A tool whose action opened a dialog \
         says so in its answer.",
    ),
    browser_write(
        "resize",
        "Lay the page out at a device's size (`preset`, such as \"iPhone 15 Pro\" or \"iPad \
         Air\"), at `width` and `height` in CSS pixels, or back at the tab's own size with \
         `fill`. The tab draws the page at that size, scaled to fit; the user agent stays.",
    ),
    browser_write(
        "color_scheme",
        "Make the page's prefers-color-scheme `dark` or `light`, or follow the system again \
         with `system`.",
    ),
    browser_read(
        "wait",
        "Wait until a page shows `text`, stops showing `text_gone`, has an element of `role` \
         (with `name`), or is at a URL holding `url`: every condition given, looked for every \
         250 ms for up to `timeout_ms` (10000 by default, 20000 at most). Answers what held and \
         how long it took; refused `not_met` with what did not hold.",
    ),
    ToolSpec {
        family: Family::Ports,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List the TCP ports the processes of each open project listen on, its dev \
                      servers among them: for each, the project, the project folder that holds \
                      the process's working directory, the address and port, the URL that \
                      reaches it, the pid, the process's name and its working directory. A \
                      listener outside every project is not listed.",
    },
    ToolSpec {
        family: Family::Prompt,
        verb: "open",
        tier: Tier::Write,
        grant_class: "editor.write",
        description: "Open a file in a tab of the workspace of the Marley terminal calling, for \
                      an agent's editor key (#649); the edit ends when the tab closes.",
    },
    ToolSpec {
        family: Family::Prompt,
        verb: "wait",
        tier: Tier::Read,
        grant_class: "",
        description: "Wait up to wait_seconds for an edit prompt_open began to end, its tab \
                      closed.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List the editors open in Marley: each one's id, its file's absolute path, \
                      its project, whether it has unsaved changes, its language, whether it is \
                      its pane's active tab, and its cursor and selections as 1-based lines and \
                      columns.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "Read an open editor's text as it is now, unsaved changes included, by its \
                      id from editor_list or its file's path, a page of lines at a time, keys and \
                      tokens masked. A file matching the user's secret patterns is refused.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "open",
        tier: Tier::Write,
        grant_class: "editor.write",
        description: "Open a file inside one of the open projects' folders in that project's \
                      window, its tab in front, the cursor at the line and column given. The \
                      user may be asked first.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "edit",
        tier: Tier::Write,
        grant_class: "editor.write",
        description: "Change an open editor's text: replace old_text, which must appear exactly \
                      once unless replace_all is true, with new_text. One undo takes the change \
                      back, and it stays unsaved. The user may be asked first.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "save",
        tier: Tier::Write,
        grant_class: "editor.write",
        description: "Save an open editor's file. The user is asked each time unless they \
                      allowed agents to act in editors without asking.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "diagnostics",
        tier: Tier::Read,
        grant_class: "",
        description: "The language servers' errors, warnings and notes: for one file by its \
                      absolute path, or for every file of a project with errors or warnings, \
                      as the servers last reported them, each with its 1-based line and \
                      column, severity, message, source and code, a page at a time.",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "definition",
        tier: Tier::Read,
        grant_class: "",
        description: "Where a symbol is defined, as its language server answers: give the \
                      file's absolute path, the 1-based line, and the symbol's name on that line \
                      (or its 1-based column, counted as editor_list counts it).",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "references",
        tier: Tier::Read,
        grant_class: "",
        description: "Where a symbol is used, as its language server answers, a page at a time: \
                      give the file's absolute path, the 1-based line, and the symbol's name on \
                      that line (or its 1-based column).",
    },
    ToolSpec {
        family: Family::Editor,
        verb: "hover",
        tier: Tier::Read,
        grant_class: "",
        description: "A symbol's hover text from its language server, such as its type and \
                      documentation, and the range it covers: give the file's absolute path, \
                      the 1-based line, and the symbol's name on that line (or its 1-based \
                      column).",
    },
    ToolSpec {
        family: Family::Thread,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List the Agent Panel's threads in Marley's windows: each one's id, title, \
                      agent, project, whether it is running, and the permission it waits on, if \
                      any, with whether an agent may answer it.",
    },
    ToolSpec {
        family: Family::Thread,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "Read a thread by its id from thread_list as Markdown, its messages and \
                      tool calls, a page of lines at a time, keys and tokens masked.",
    },
    ToolSpec {
        family: Family::Thread,
        verb: "post",
        tier: Tier::Write,
        grant_class: "thread.write",
        description: "Send a message into a thread, as the user would type it; a running thread \
                      queues it. The user may be asked first.",
    },
    ToolSpec {
        family: Family::Thread,
        verb: "answer",
        tier: Tier::Write,
        grant_class: "thread.write",
        description: "Answer the permission a thread waits on: allow it once, or reject it. The \
                      user is asked each time unless they allowed agents to act on threads \
                      without asking. A sandbox escalation is never answered.",
    },
    ToolSpec {
        family: Family::Action,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "List the palette actions an agent may run in Marley: each one's name, its \
                      documentation, and whether the user allowed it by name.",
    },
    ToolSpec {
        family: Family::Action,
        verb: "run",
        tier: Tier::Write,
        grant_class: "action.write",
        description: "Run a palette action from action_list by name, with its JSON arguments, \
                      in the window of the project given (else the active one), as the command \
                      palette would. The user may be asked first.",
    },
    ToolSpec {
        family: Family::Docs,
        verb: "search",
        tier: Tier::Read,
        grant_class: "",
        description: "Search Zed's docs and Marley's guide, walkthrough and changelog, as this \
                      Marley ships them, for a query in words, such as \"terminal font size\" or \
                      \"rail filter\": up to 10 sections, best first, each with its page, heading \
                      and a snippet. A changelog entry is a section of its own, so a question of \
                      what changed and when finds its ticket. Read one with docs_read. The words, \
                      or their forms without -s, -es, -ing or -ed, must appear; a query in other \
                      words can miss.",
    },
    ToolSpec {
        family: Family::Docs,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "Read a page of Zed's docs (`zed/<path>`) or Marley's (`marley/guide.md`, \
                      `marley/walkthrough.md`, `marley/CHANGELOG.md`), whole or one section by \
                      `heading`, from the top a page \
                      at a time: the lines that fit in 12,000 bytes, with `next` to pass back as \
                      `after` for the rest. Key placeholders show the keys bound in this Marley.",
    },
    ToolSpec {
        family: Family::Settings,
        verb: "schema",
        tier: Tier::Read,
        grant_class: "",
        description: "What a setting is, by its key path in settings.json, such as \
                      `terminal.font_size` or `marley.rail_order`: its type, description, \
                      allowed values and default, and for an object its keys. An empty key lists \
                      the top-level keys.",
    },
    ToolSpec {
        family: Family::Settings,
        verb: "read",
        tier: Tier::Read,
        grant_class: "",
        description: "What a setting is set to, by its key path: the value in each settings file \
                      that sets it, highest precedence first (each open project's \
                      .zed/settings.json, the user's settings, the defaults), which one wins, the \
                      value in effect, and `global`, the value where no project file sets it. \
                      Values under names that read as secrets come back as `[redacted: setting]`.",
    },
    ToolSpec {
        family: Family::Settings,
        verb: "change",
        tier: Tier::Write,
        grant_class: "settings.write",
        description: "Propose a value for one key of the user's settings, by its key path and a \
                      JSON value: Marley shows the user the value now and the value proposed and \
                      writes it, comments kept, only on Apply. Answers `applied` with the value \
                      before and after, or `unchanged`; refused with `no_setting` for a key the \
                      schema lacks, `invalid_value` for a value Zed would not parse, `declined`, \
                      `no_answer` after 25 seconds, or `changed` if the file changed meanwhile.",
    },
    ToolSpec {
        family: Family::Keymap,
        verb: "change",
        tier: Tier::Write,
        grant_class: "settings.write",
        description: "Propose a key binding for the user's keymap: `keystrokes` as Zed's keymap \
                      writes them (`ctrl-alt-m`), an `action` by its name (actions_list finds it), \
                      an optional `context` (`Workspace`) and optional JSON `arguments`. Marley \
                      shows the user the binding and adds it only on Apply; it works at once. \
                      Refused with `no_action` for an action Marley lacks, `bad_argument` for \
                      keystrokes or a context that do not parse, `declined`, `no_answer` after 25 \
                      seconds, or `changed`.",
    },
    ToolSpec {
        family: Family::Seat,
        verb: "add",
        tier: Tier::Write,
        grant_class: "harness.write",
        description: "Propose a seat on the harness Marley follows: a `name`, the `agent` \
                      (`claude` or `codex`), the folder it works in (`cwd`) and an optional \
                      `role` (`manager` makes it the root's manager). Marley shows the user the \
                      seat and, only on Apply, runs the harness's `seat add`, answers `starting`, \
                      then starts it; it shows in the rail's Harness section. Refused with the \
                      harness's own code (`seat_exists`, `seat_role_reserved`, \
                      `claude_signin_undeclared` …), `tool_off` while the user's \
                      `marley.harness_writes` is off, `declined`, or `no_answer` after 25 seconds.",
    },
    ToolSpec {
        family: Family::Seat,
        verb: "stop",
        tier: Tier::Write,
        grant_class: "harness.write",
        description: "Propose to stop a seat on the harness Marley follows, by its `name`. The \
                      harness ends the seat's sessions and their supervision. The profile stays, \
                      so the seat can start again. Marley asks the user and runs the harness's \
                      `seat stop` only on Apply. The answer lists the stopped sessions. A \
                      refusal has the harness's code (`seat_unknown`, `seat_runtime`, \
                      `seat_stop_failed`), `tool_off` while `marley.harness_writes` is off, \
                      `declined`, or `no_answer` after 25 seconds.",
    },
    ToolSpec {
        family: Family::Seat,
        verb: "remove",
        tier: Tier::Write,
        grant_class: "harness.write",
        description: "Propose to remove a seat on the harness Marley follows, by its `name`. \
                      The harness stops the seat, then deletes its profile, so the name is free \
                      again. Marley asks the user and runs the harness's `seat remove` only on \
                      Apply. The answer lists the stopped sessions and the removed profile. A \
                      refusal has the harness's code (`seat_unknown`, `seat_runtime`, \
                      `seat_stop_failed`, `seat_profile`), `tool_off`, `declined`, or \
                      `no_answer`.",
    },
    ToolSpec {
        family: Family::Draft,
        verb: "message",
        tier: Tier::Write,
        grant_class: "draft.write",
        description: "Draft a message to another agent for the user to review: `to` names an \
                      agent CLI's terminal (`{\"terminal\": <id from terminal_list>}`) or a \
                      thread (`{\"thread\": \"<id from thread_list>\"}`). Marley opens the \
                      draft in a tab of the user's, who may edit it, then Send or Drop it. It \
                      answers `opened` at once and sends nothing itself. Refused `no_terminal`, \
                      `not_an_agent` for a terminal running no agent CLI, `no_thread`, or \
                      `bad_argument`.",
    },
    ToolSpec {
        family: Family::Review,
        verb: "notes_add",
        tier: Tier::Write,
        grant_class: "review.write",
        description: "Put findings into the project diff as review notes for the user to keep or \
                      drop: `source` names what found them (your name when left out), `notes` \
                      up to 50, each an absolute `path`, `first_line`, optional `last_line` (from \
                      1, on the changed side) and `comment`. Marley opens the project's diff and \
                      shows each note under its lines with Keep and Drop; a kept note goes to the \
                      author agent with the user's own. A note whose lines the diff does not show \
                      is left out and named in `refused` with the reason. Asks the user first, as \
                      the editor tools do.",
    },
    ToolSpec {
        family: Family::Actions,
        verb: "list",
        tier: Tier::Read,
        grant_class: "",
        description: "The actions whose name or documentation holds every word of `query`, up to \
                      50: each action's name, its name in the command palette, the first line of \
                      its documentation, and the keys bound to it with the context each applies \
                      in.",
    },
];

/// A browser tool that reads the page (#492).
const fn browser_read(verb: &'static str, description: &'static str) -> ToolSpec {
    ToolSpec {
        family: Family::Browser,
        verb,
        tier: Tier::Read,
        grant_class: "",
        description,
    }
}

/// A browser tool that acts in the page the user sees (#492): Marley grants `browser.write` when
/// it starts the server; no setting takes it away yet.
const fn browser_write(verb: &'static str, description: &'static str) -> ToolSpec {
    ToolSpec {
        family: Family::Browser,
        verb,
        tier: Tier::Write,
        grant_class: "browser.write",
        description,
    }
}

/// The L1 tool table (the const `REGISTRY`).
#[must_use]
pub const fn registry() -> &'static [ToolSpec] {
    REGISTRY
}

/// The tools listed and called only while the user turns them on (#567): each is the System One
/// use of the same name, on while its mode is not `off`.
pub const CONDITIONAL_TOOLS: [&str; 2] = ["browser_find", "terminal_find"];

/// Whether the tool `name` is off: one of [`CONDITIONAL_TOOLS`] that `enabled` leaves out.
#[must_use]
pub fn is_off(name: &str, enabled: &BTreeSet<String>) -> bool {
    CONDITIONAL_TOOLS.contains(&name) && !enabled.contains(name)
}

/// Look up a tool by its wire name (`None` = unknown tool).
#[must_use]
pub fn lookup(wire_name: &str) -> Option<ToolSpec> {
    REGISTRY
        .iter()
        .copied()
        .find(|spec| spec.name() == wire_name)
}

/// The `tools/list` result (REQ-001) — DERIVED from `REGISTRY`: name + description from the spec,
/// the per-tool input/output schemas from `tool_schemas`, for the families the server serves.
///
/// One source of truth, no hand-repeated parallel table → no drift (a row added to `REGISTRY` is
/// automatically listed once its family is served).
#[must_use]
pub fn tools_list() -> Value {
    tools_list_for(&Principal::Marley, &BTreeSet::new())
}

/// The `tools/list` result for `principal` (#524).
///
/// Marley's own bearer lists every served tool, and an outside client only the tools its grant's
/// list names. A conditional tool is listed while `enabled` holds it (#567).
#[must_use]
pub fn tools_list_for(principal: &Principal, enabled: &BTreeSet<String>) -> Value {
    let tools: Vec<Value> = REGISTRY
        .iter()
        .filter(|spec| {
            let name = spec.name();
            spec.family.is_served() && permits(principal, &name).is_ok() && !is_off(&name, enabled)
        })
        .map(|spec| {
            let (input, output) = tool_schemas(spec);
            json!({
                "name": spec.name(),
                "description": spec.description,
                "inputSchema": input,
                "outputSchema": output,
                "annotations": annotations(spec),
            })
        })
        .collect();
    json!({ "tools": tools })
}

/// The write tools that change or send what the user cannot take back from Marley: MCP's
/// `destructiveHint` (#766).
const DESTRUCTIVE: &[&str] = &[
    "session_surface_to_human",
    "terminal_type",
    "terminal_run",
    "editor_edit",
    "editor_save",
    "thread_post",
    "thread_answer",
    "action_run",
    "settings_change",
    "keymap_change",
    "seat_add",
    "seat_stop",
    "seat_remove",
    "browser_navigate",
    "browser_back",
    "browser_click",
    "browser_type",
    "browser_press",
    "browser_select",
    "browser_drag",
    "browser_upload",
    "browser_dialog",
];

/// The write tools a second call with the same arguments leaves as the first did (#766).
const IDEMPOTENT: &[&str] = &[
    "editor_open",
    "browser_open_url",
    "browser_resize",
    "browser_color_scheme",
];

/// The browser's tools that read Marley's own records, not a page: closed-world (#766).
const BROWSER_RECORDS: &[&str] = &[
    "browser_picks",
    "browser_pick",
    "browser_recordings",
    "browser_recording",
    "browser_draft_test",
    "browser_annotations",
];

/// A tool's MCP annotations (#766): its title and all four hints, since MCP's defaults
/// (destructive, open world) are the wrong guess for most of Marley's tools. A page the browser
/// tools reach is the open world; every other tool stays inside Marley and its projects.
fn annotations(spec: &ToolSpec) -> Value {
    let name = spec.name();
    let read = spec.tier == Tier::Read;
    let open_world = spec.family == Family::Browser && !BROWSER_RECORDS.contains(&name.as_str());
    json!({
        "title": tool_title(spec),
        "readOnlyHint": read,
        "destructiveHint": !read && DESTRUCTIVE.contains(&name.as_str()),
        "idempotentHint": read || IDEMPOTENT.contains(&name.as_str()),
        "openWorldHint": open_world,
    })
}

/// A tool's name in words, as a client shows it: "Browser: color scheme".
fn tool_title(spec: &ToolSpec) -> String {
    let family = spec.family.as_str();
    let mut letters = family.chars();
    let family = letters
        .next()
        .map(|first| first.to_ascii_uppercase().to_string() + letters.as_str())
        .unwrap_or_default();
    format!("{family}: {}", spec.verb.replace('_', " "))
}

/// The input + output JSON Schemas for a tool (the one thing that is NOT a `&'static str` on `ToolSpec`).
/// Matches on the family (EXHAUSTIVE — no catch-all, so adding a `Family` variant is a compile error until
/// its schemas are wired: additivity is compiler-enforced, REQ-011). The terminal family's three tools
/// have an inner `verb` match.
fn tool_schemas(spec: &ToolSpec) -> (Value, Value) {
    match spec.family {
        Family::Terminal => terminal_schemas(spec.verb),
        Family::Browser => browser_schemas(spec.verb),
        Family::Ports => ports_list_schemas(),
        Family::Editor => editor_schemas(spec.verb),
        Family::Prompt => prompt_schemas(spec.verb),
        Family::Thread => thread_schemas(spec.verb),
        Family::Action => action_schemas(spec.verb),
        Family::Docs => docs_schemas(spec.verb),
        Family::Settings => settings_schemas(spec.verb),
        Family::Actions => actions_list_schemas(),
        Family::Keymap => keymap_change_schemas(),
        Family::Seat => seat_schemas(spec.verb),
        Family::Draft => draft_schemas(),
        Family::Review => review_schemas(),
        Family::Fleet => (
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
            fleet_snapshot_schema(),
        ),
        Family::Session => (
            json!({
                "type": "object",
                "properties": { "id": { "type": "string", "description": "The session id to surface." } },
                "required": ["id"],
                "additionalProperties": false
            }),
            receipt_schema(),
        ),
    }
}

/// The schemas of `draft_message` (#769): the target and the text; the answer says it opened.
fn draft_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "to": {
                    "type": "object",
                    "description": "Who the message is for: an agent CLI's terminal or a thread.",
                    "properties": {
                        "terminal": {
                            "type": "integer",
                            "description": "A terminal's id from terminal_list, running an agent CLI."
                        },
                        "thread": {
                            "type": "string",
                            "description": "A thread's id from thread_list."
                        }
                    },
                    "additionalProperties": false
                },
                "text": { "type": "string", "minLength": 1, "description": "The message." }
            },
            "required": ["to", "text"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "draft": { "type": "string" },
                "to": { "type": "string" },
                "opened": { "type": "boolean" }
            },
            "required": ["draft", "to", "opened"]
        }),
    )
}

/// The schemas of `review_notes_add` (#730): the source and the notes; the answer lists the notes
/// placed and the ones left out.
fn review_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "source": {
                    "type": "string",
                    "description": "What found the notes, such as `clippy` or `codex review`."
                },
                "notes": {
                    "type": "array",
                    "minItems": 1,
                    "maxItems": 50,
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string", "description": "The file's absolute path." },
                            "first_line": { "type": "integer", "minimum": 1 },
                            "last_line": { "type": "integer", "minimum": 1 },
                            "comment": { "type": "string", "minLength": 1 }
                        },
                        "required": ["path", "first_line", "comment"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["notes"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "source": { "type": "string" },
                "placed": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "first_line": { "type": "integer" },
                            "last_line": { "type": "integer" }
                        }
                    }
                },
                "refused": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "path": { "type": "string" },
                            "first_line": { "type": "integer" },
                            "reason": { "type": "string" }
                        }
                    }
                }
            },
            "required": ["source", "placed", "refused"]
        }),
    )
}

/// The JSON Schema for a `FleetSnapshot` (the #367 shape — one seam, three consumers, D2). Kept in step
/// with `marley_fleet::FleetSnapshot`/`Session`; a drift is the defect class D2 kills.
fn fleet_snapshot_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "seats": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "id": { "type": "string" },
                        "title": { "type": "string" },
                        "state": { "type": "string", "enum": ["starting", "working", "idle", "waiting", "error", "done"] },
                        "question": { "type": ["object", "null"] },
                        "labels": { "type": "object" },
                        "capabilities": { "type": "object", "additionalProperties": { "type": "string" } },
                        "last_event_ms": { "type": "integer" },
                        "transport": { "type": ["string", "null"], "enum": ["tmux", "bridge", "local", null] }
                    },
                    "required": ["id", "title", "state"]
                }
            }
        },
        "required": ["seats"]
    })
}

/// The input and output schemas of the browser family's tools (#492, #493).
fn browser_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "tabs" => tabs_schemas(),
        "look" => look_schemas(),
        "snapshot" => snapshot_schemas(),
        "find" => find_schemas(),
        "console" | "network" => entries_schemas(verb),
        "annotations" => annotations_schemas(),
        "recordings" => recordings_schemas(),
        "recording" => recording_schemas(),
        "draft_test" => draft_test_schemas(),
        "picks" => picks_schemas(),
        "pick" => pick_schemas(),
        "check_pick" => check_pick_schemas(),
        "open_url" => open_url_schemas(),
        "wait" => wait_schemas(),
        _ => browser_write_schemas(verb),
    }
}

/// `browser_wait` (#756): the conditions and the time allowed; what held and how long it took.
fn wait_schemas() -> (Value, Value) {
    (
        browser_arguments(
            json!({
                "text": { "type": "string", "description": "Text the page shows." },
                "text_gone": { "type": "string", "description": "Text the page no longer shows." },
                "role": { "type": "string", "description": "An element's role, as browser_snapshot names roles." },
                "name": { "type": "string", "description": "The accessible name of the element of `role`." },
                "url": { "type": "string", "description": "Text the page's URL holds." },
                "timeout_ms": { "type": "integer", "minimum": 0, "maximum": 20000 }
            }),
            &[],
        ),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "held": { "type": "array", "items": { "type": "string" } },
                "waited_ms": { "type": "integer" }
            },
            "required": ["tab", "held", "waited_ms"]
        }),
    )
}

/// `browser_open_url` (#561): the URL and the program's folder; whether a tab opened, and in
/// which project, or why not.
fn open_url_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "url": { "type": "string", "description": "The http or https URL the program opens, or a local HTML page as a file: URL or a path." },
                "directory": { "type": "string", "description": "The program's working directory, an absolute path." }
            },
            "required": ["url", "directory"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "opened": { "type": "boolean" },
                "project": { "type": "string", "description": "The project whose Browser tab shows the URL." },
                "reason": { "type": "string", "description": "Why no tab opened." }
            },
            "required": ["opened"]
        }),
    )
}

/// The schema of the `tab` argument every browser tool but `browser_tabs` takes (#493).
fn tab_argument_schema() -> Value {
    json!({
        "type": "string",
        "description": "A tab's id from browser_tabs. Left out: the tab the user focused last in \
                        the project you run in (your terminal's, else your folder's); for a \
                        caller in no project of Marley's, the tab the user focused last."
    })
}

/// A browser tool's arguments: `properties`, with `tab`, and the `required` ones.
fn browser_arguments(mut properties: Value, required: &[&str]) -> Value {
    if let Some(properties) = properties.as_object_mut() {
        properties.extend([("tab".to_string(), tab_argument_schema())]);
    }
    let mut schema = json!({
        "type": "object",
        "properties": properties,
        "additionalProperties": false
    });
    if !required.is_empty() {
        schema["required"] = json!(required);
    }
    schema
}

/// `browser_tabs`: no arguments; each tab.
fn tabs_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "tabs": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "string" },
                            "title": { "type": "string" },
                            "url": { "type": "string" },
                            "loading": { "type": "boolean" },
                            "focused": {
                                "type": "boolean",
                                "description": "Whether the user focused this tab last, anywhere."
                            },
                            "project": {
                                "type": ["string", "null"],
                                "description": "The project the tab belongs to, as Marley's rail names it."
                            },
                            "default": {
                                "type": "boolean",
                                "description": "The tab the tools act on when a call of yours names none."
                            }
                        },
                        "required": ["id", "title", "url", "loading", "focused", "project", "default"]
                    }
                }
            },
            "required": ["tabs"]
        }),
    )
}

fn look_schemas() -> (Value, Value) {
    (
        browser_arguments(json!({}), &[]),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "url": { "type": "string" },
                "title": { "type": "string" },
                "loading": { "type": "boolean" },
                "viewport": { "type": "object" },
                "focused": { "type": ["object", "null"] },
                "selection": { "type": "string" }
            },
            "required": ["tab", "url", "title", "loading", "viewport"]
        }),
    )
}

fn snapshot_schemas() -> (Value, Value) {
    (
        browser_arguments(
            json!({
                "full": { "type": "boolean", "description": "Every node, not only the interactive ones." }
            }),
            &[],
        ),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "snapshot": { "type": "string" },
                "refs": { "type": "integer" },
                "cut": { "type": "boolean" }
            },
            "required": ["tab", "snapshot", "refs", "cut"]
        }),
    )
}

/// The schema of a find's `query`, for `browser_find` and `terminal_find` (#567).
fn query_schema(example: &str) -> Value {
    json!({
        "type": "string",
        "maxLength": 200,
        "description": format!("What to find, in words, such as \"{example}\".")
    })
}

/// The answer's fields both find tools give (#567).
fn found_properties() -> Value {
    json!({
        "query": { "type": "string" },
        "source": {
            "type": "string",
            "enum": ["rules", "model", "none"],
            "description": "What found the answer: the query's words (rules), the System One model, or nothing."
        },
        "sure": { "type": "boolean", "description": "Whether the answer can be acted on without a look." },
        "present": {
            "type": ["string", "null"],
            "enum": ["found", "absent", "unsure", null],
            "description": "The model's reading of whether anything matches, when it was asked."
        },
        "verify": { "type": "boolean", "description": "The candidates are the model's suggestions: look before acting." },
        "next": { "type": ["string", "null"], "description": "The tool to read with when the answer is not sure." },
        "note": { "type": ["string", "null"], "description": "Why the model was not asked or did not answer, when it was not or did not." }
    })
}

/// `browser_find` (#567): a query; the matching element's ref, and up to three candidates.
fn find_schemas() -> (Value, Value) {
    let mut output = found_properties();
    if let Some(properties) = output.as_object_mut() {
        properties.extend([
            ("tab".to_string(), json!({ "type": "string" })),
            (
                "ref".to_string(),
                json!({ "type": ["string", "null"], "description": "The element's ref, when the answer is sure." }),
            ),
            (
                "candidates".to_string(),
                json!({
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "ref": { "type": "string" },
                            "role": { "type": "string" },
                            "name": { "type": "string" },
                            "probability": { "type": ["number", "null"] }
                        },
                        "required": ["ref", "role", "name"]
                    }
                }),
            ),
        ]);
    }
    (
        browser_arguments(
            json!({
                "query": query_schema("the sign in button"),
                "full": { "type": "boolean", "description": "Look among every node, not only the interactive ones." }
            }),
            &["query"],
        ),
        json!({
            "type": "object",
            "properties": output,
            "required": ["tab", "query", "source", "sure", "candidates"]
        }),
    )
}

/// `browser_console` and `browser_network`: the entries of the tab's ring.
fn entries_schemas(verb: &str) -> (Value, Value) {
    let item = if verb == "console" {
        json!({
            "type": "object",
            "properties": {
                "level": { "type": "string" },
                "text": { "type": "string" },
                "source": { "type": ["string", "null"] },
                "line": { "type": ["integer", "null"] },
                "time_ms": { "type": ["number", "null"] }
            },
            "required": ["level", "text"]
        })
    } else {
        json!({
            "type": "object",
            "properties": {
                "method": { "type": "string" },
                "url": { "type": "string" },
                "kind": { "type": ["string", "null"] },
                "status": { "type": ["integer", "null"] },
                "duration_ms": { "type": ["integer", "null"] },
                "failure": { "type": ["string", "null"] }
            },
            "required": ["method", "url"]
        })
    };
    (
        browser_arguments(json!({}), &[]),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "entries": { "type": "array", "items": item }
            },
            "required": ["tab", "entries"]
        }),
    )
}

/// A box in page coordinates, the document's CSS pixels.
fn page_box_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "x": { "type": "number" },
            "y": { "type": "number" },
            "width": { "type": "number" },
            "height": { "type": "number" }
        },
        "required": ["x", "y", "width", "height"]
    })
}

/// What `browser_recordings` and `browser_recording` say of a recording besides its entries.
fn recording_properties() -> Value {
    json!({
        "id": { "type": "string" },
        "tab": { "type": "string" },
        "url": { "type": "string" },
        "title": { "type": "string" },
        "recorded_at": { "type": "integer", "description": "Seconds since the Unix epoch." },
        "seconds": { "type": "number", "description": "How long it spans." },
        "frames": { "type": "integer" }
    })
}

/// `browser_recordings`: no arguments; each recording, without its entries (#499).
fn recordings_schemas() -> (Value, Value) {
    let mut item = recording_properties();
    if let Some(properties) = item.as_object_mut() {
        properties.extend([(
            "entries".to_string(),
            json!({ "type": "integer", "description": "How many entries it holds." }),
        )]);
    }
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "recordings": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": item,
                        "required": ["id", "tab", "url", "title", "recorded_at", "seconds", "frames", "entries"]
                    }
                }
            },
            "required": ["recordings"]
        }),
    )
}

/// `browser_recording`: a recording's id and, with `frame`, one frame as the answer's image.
fn recording_schemas() -> (Value, Value) {
    let mut properties = recording_properties();
    if let Some(object) = properties.as_object_mut() {
        object.extend([
            (
                "entries".to_string(),
                json!({
                    "type": "array",
                    "description": "What happened, oldest first: each entry's `kind` (click, scroll, key, typed, navigation, action, console, request, snapshot, agent or frame), `at_ms` from the recording's start, and its details. An action is a click, fill or press with the target's locators, most durable first, each marked `unique` when it found the element alone.",
                    "items": { "type": "object", "properties": { "kind": { "type": "string" }, "at_ms": { "type": "integer" } }, "required": ["kind", "at_ms"] }
                }),
            ),
            (
                "snapshot".to_string(),
                json!({ "type": "string", "description": "The page's accessibility snapshot when it was saved." }),
            ),
        ]);
    }
    (
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "A recording's id from browser_recordings." },
                "frame": { "type": "integer", "minimum": 1, "description": "A frame to see, from 1, as an entry of kind frame names it." }
            },
            "required": ["id"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": properties,
            "required": ["id", "tab", "url", "title", "recorded_at", "seconds", "frames", "entries", "snapshot"]
        }),
    )
}

/// `browser_annotations`: the tab's annotations (#498).
fn annotations_schemas() -> (Value, Value) {
    (
        browser_arguments(json!({}), &[]),
        json!({
            "type": "object",
            "properties": {
                "tab": { "type": "string" },
                "annotations": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "box": page_box_schema(),
                            "note": { "type": "string" },
                            "maker": { "type": "string", "enum": ["user", "agent"] },
                            "made_at": { "type": "integer", "description": "Seconds since the Unix epoch." }
                        },
                        "required": ["id", "box", "note", "maker", "made_at"]
                    }
                }
            },
            "required": ["tab", "annotations"]
        }),
    )
}

/// `browser_draft_test` (#506): a recording's id; the drafted test and where it goes.
fn draft_test_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string", "description": "A recording's id from browser_recordings." }
            },
            "required": ["id"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "string" },
                "test": { "type": "string", "description": "The Playwright test, a TypeScript file's text." },
                "path": { "type": "string", "description": "Where the test goes: in the recording's project, when it names one." },
                "project": { "type": ["string", "null"], "description": "The root of the project the recording was made in." },
                "env": {
                    "type": "array",
                    "description": "The environment variables the test reads, one per secret field it fills.",
                    "items": { "type": "string" }
                },
                "skipped": {
                    "type": "array",
                    "description": "What the test leaves out, and why.",
                    "items": { "type": "string" }
                },
                "start": { "type": "string", "description": "The page the test starts on." },
                "run": { "type": "string", "description": "The command that runs it, from the project's root." },
                "note": { "type": ["string", "null"] }
            },
            "required": ["id", "test", "path", "env", "skipped", "start", "run"]
        }),
    )
}

/// What `browser_picks` and `browser_pick` say of a pick besides its bundle.
fn pick_properties() -> Value {
    json!({
        "id": { "type": "integer" },
        "tab": { "type": "string", "description": "The tab the pick was made in." },
        "url": { "type": "string" },
        "title": { "type": "string" },
        "summary": { "type": "string", "description": "The element's role and name, or its tag and text." },
        "caption": { "type": "string", "description": "What the user said of it when sending it." },
        "sent": { "type": "boolean", "description": "Whether the user sent it to the agent." }
    })
}

/// `browser_picks`: no arguments; each pick of the session (#496).
fn picks_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "picks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": pick_properties(),
                        "required": ["id", "tab", "url", "title", "summary", "caption", "sent"]
                    }
                }
            },
            "required": ["picks"]
        }),
    )
}

/// What #518 adds to a pick's bundle: the element's HTML, styles, sibling texts, the page's
/// selection and the React component.
fn element_context_properties() -> Value {
    json!({
        "html": {
            "type": "string",
            "description": "The element's HTML without scripts, field values, secret-looking attribute values or URL queries and fragments; at most 4,096 characters, ending ' (truncated)' when cut."
        },
        "styles": {
            "type": "object",
            "description": "Sixteen of its computed styles, by their CSS names.",
            "additionalProperties": { "type": "string" }
        },
        "nearby_text": {
            "type": "array",
            "description": "Its siblings' texts, the nearest first, before and after in turn; at most ten of 200 characters.",
            "items": { "type": "string" }
        },
        "selected_text": {
            "type": ["string", "null"],
            "description": "The page's selection at the pick, at most 500 characters; none when it lay in a field."
        },
        "component": {
            "type": ["object", "null"],
            "description": "On a React dev build: the components around the element and where it was written.",
            "properties": {
                "chain": {
                    "type": "array",
                    "description": "The components around it, the outermost first, at most six.",
                    "items": { "type": "string" }
                },
                "source": {
                    "type": ["object", "null"],
                    "description": "Where it was written: React's debug source, or React 19's debug stack through the scripts' source maps.",
                    "properties": {
                        "from": { "type": "string", "enum": ["debug source", "debug stack"] },
                        "source": { "type": "string", "description": "The source as React or the source map names it." },
                        "file": {
                            "type": ["string", "null"],
                            "description": "The file in the user's project that holds it, relative to its worktree."
                        },
                        "line": { "type": "integer", "description": "From 1." },
                        "column": {
                            "type": ["integer", "null"],
                            "description": "From 1 for a debug stack; as the JSX transform wrote it for a debug source."
                        }
                    },
                    "required": ["from", "source", "line"]
                }
            },
            "required": ["chain"]
        }
    })
}

/// `browser_pick`: a pick's id; the pick with its bundle and its latest check, and its crop as the
/// answer's image.
fn pick_schemas() -> (Value, Value) {
    let mut properties = pick_properties();
    if let Some(properties) = properties.as_object_mut() {
        properties.extend([
            ("bundle".to_string(), bundle_schema()),
            ("check".to_string(), check_schema()),
        ]);
    }
    (
        pick_id_schema(),
        json!({
            "type": "object",
            "properties": properties,
            "required": ["id", "tab", "url", "title", "summary", "caption", "sent", "bundle"]
        }),
    )
}

/// The arguments of a tool that takes a pick: its id.
fn pick_id_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "id": {
                "type": "integer",
                "minimum": 1,
                "description": "A pick's id, as the user's line or browser_picks names it."
            }
        },
        "required": ["id"],
        "additionalProperties": false
    })
}

/// A pick's latest check (#505), as `browser_pick` gives it.
fn check_schema() -> Value {
    let bundle = nullable(bundle_schema());
    json!({
        "type": ["object", "null"],
        "description": "The pick's latest check from browser_check_pick; none before one ran.",
        "properties": {
            "found_by": {
                "type": ["string", "null"],
                "description": "The kind of locator that found the element again: test id, id, role and name, text or css; none when nothing did."
            },
            "changes": {
                "type": "array",
                "description": "What changed since the pick, a line each.",
                "items": { "type": "string" }
            },
            "bundle": bundle,
            "checked_at": { "type": "integer", "description": "When it ran, in milliseconds since the Unix epoch." }
        },
        "required": ["found_by", "changes", "checked_at"]
    })
}

/// `browser_check_pick` (#505): a pick's id; whether and how its element was found again, what
/// changed, and the element now, with its crop as the answer's image.
fn check_pick_schemas() -> (Value, Value) {
    let bundle = nullable(bundle_schema());
    (
        pick_id_schema(),
        json!({
            "type": "object",
            "properties": {
                "id": { "type": "integer" },
                "tab": { "type": "string", "description": "The pick's tab, where the check ran." },
                "found": { "type": "boolean" },
                "found_by": {
                    "type": ["string", "null"],
                    "description": "The kind of locator that found it: test id, id, role and name, text or css."
                },
                "changes": {
                    "type": "array",
                    "description": "What changed since the pick, a line each: the box, each computed style, the text, the role, the name, or that only the HTML did.",
                    "items": { "type": "string" }
                },
                "bundle": bundle,
                "checked_at": { "type": "integer", "description": "In milliseconds since the Unix epoch." }
            },
            "required": ["id", "tab", "found", "found_by", "changes", "checked_at"]
        }),
    )
}

/// `schema`, an object's, taking null too.
fn nullable(mut schema: Value) -> Value {
    schema["type"] = json!(["object", "null"]);
    schema
}

/// A pick's bundle: the element as a pick or a check read it.
fn bundle_schema() -> Value {
    let mut bundle = json!({
        "type": "object",
        "properties": {
            "tag": { "type": "string" },
            "role": { "type": ["string", "null"] },
            "name": { "type": ["string", "null"] },
            "text": { "type": "string" },
            "locators": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "kind": { "type": "string", "description": "test id, id, text or css." },
                        "value": { "type": "string" },
                        "unique": { "type": ["boolean", "null"] }
                    },
                    "required": ["kind", "value"]
                }
            },
            "listeners": {
                "type": "array",
                "items": {
                    "type": "object",
                    "properties": {
                        "event": { "type": "string" },
                        "on": { "type": "string", "description": "The element, an ancestor, the document or the window." },
                        "script": { "type": ["string", "null"] },
                        "line": { "type": "integer", "description": "In the script, from 1." },
                        "column": { "type": "integer", "description": "In the script, from 1." },
                        "source_map": { "type": ["string", "null"] },
                        "original": {
                            "type": ["object", "null"],
                            "description": "Where the script's source map says the listener was written.",
                            "properties": {
                                "source": { "type": "string", "description": "The source, as its map names it." },
                                "file": {
                                    "type": ["string", "null"],
                                    "description": "The file in the user's project that holds it, relative to its worktree."
                                },
                                "line": { "type": "integer", "description": "From 1." },
                                "column": { "type": "integer", "description": "From 1." }
                            },
                            "required": ["source", "line", "column"]
                        }
                    },
                    "required": ["event", "on", "line", "column"]
                }
            },
            "blockers": { "type": "array", "items": { "type": "string" } },
            "page_box": {
                "type": "object",
                "description": "In the document's CSS pixels.",
                "properties": {
                    "x": { "type": "number" },
                    "y": { "type": "number" },
                    "width": { "type": "number" },
                    "height": { "type": "number" }
                },
                "required": ["x", "y", "width", "height"]
            }
        },
        "required": ["tag", "text", "locators", "listeners", "blockers", "page_box", "html", "styles", "nearby_text"]
    });
    if let (Some(bundle_properties), Value::Object(added)) = (
        bundle.get_mut("properties").and_then(Value::as_object_mut),
        element_context_properties(),
    ) {
        bundle_properties.extend(added);
    }
    bundle
}

/// The write tools' arguments; each answers with what it did, and in which tab.
fn browser_write_schemas(verb: &str) -> (Value, Value) {
    let element =
        json!({ "type": "string", "description": "A ref from browser_snapshot, such as e3." });
    let mut done = json!({
        "type": "object",
        "properties": {
            "did": { "type": "string", "description": "What the tool did, as the Agent chip says it." },
            "tab": { "type": "string" },
            "url": { "type": "string" },
            "title": { "type": "string" },
            "dialog": {
                "type": "object",
                "description": "The dialog the action opened, which browser_dialog answers (#756).",
                "properties": {
                    "kind": { "type": "string", "enum": ["alert", "confirm", "prompt", "beforeunload"] },
                    "message": { "type": "string" }
                }
            }
        },
        "required": ["did", "tab"]
    });
    if verb == "annotate"
        && let Some(properties) = done.get_mut("properties").and_then(Value::as_object_mut)
    {
        properties.extend([
            (
                "id".to_string(),
                json!({ "type": "integer", "description": "The annotation's id; none for `clear`." }),
            ),
            ("box".to_string(), page_box_schema()),
        ]);
    }
    let arguments = match verb {
        "navigate" => browser_arguments(
            json!({
                "url": { "type": "string", "description": "An http or https URL." },
                "new_tab": { "type": "boolean", "description": "Open the page in a new tab, which leaves the user's focus where it is." }
            }),
            &["url"],
        ),
        "click" => browser_arguments(
            json!({
                "ref": element,
                "x": { "type": "number", "description": "A point of the viewport, in CSS pixels, with y." },
                "y": { "type": "number" },
                "button": { "type": "string", "enum": ["left", "right", "middle"] },
                "count": { "type": "integer", "minimum": 1, "maximum": 3 }
            }),
            &[],
        ),
        "type" => browser_arguments(
            json!({
                "text": { "type": "string" },
                "ref": element,
                "submit": { "type": "boolean", "description": "Press Enter after the text." }
            }),
            &["text"],
        ),
        "press" => browser_arguments(
            json!({
                "key": { "type": "string", "description": "Enter, Tab, Ctrl+A, Shift+ArrowLeft." }
            }),
            &["key"],
        ),
        "scroll" => browser_arguments(
            json!({
                "dy": { "type": "number", "description": "Pixels down; negative is up." },
                "dx": { "type": "number", "description": "Pixels right; negative is left." },
                "ref": element
            }),
            &[],
        ),
        "annotate" => browser_arguments(
            json!({
                "ref": element,
                "x": { "type": "number", "description": "An area of the viewport, in CSS pixels, with y, width and height." },
                "y": { "type": "number" },
                "width": { "type": "number" },
                "height": { "type": "number" },
                "note": { "type": "string", "description": "What the box says." },
                "clear": { "type": "boolean", "description": "Remove the agent's own boxes from the page instead." }
            }),
            &[],
        ),
        "hover" | "select" | "drag" | "upload" | "dialog" | "resize" | "color_scheme" => {
            acting_arguments(verb, &element)
        }
        // `back`.
        _ => browser_arguments(json!({}), &[]),
    };
    (arguments, done)
}

/// The arguments of the acting tools #756 added: `hover`, `select`, `drag`, `upload`, `dialog`,
/// `resize` and `color_scheme`.
fn acting_arguments(verb: &str, element: &Value) -> Value {
    match verb {
        "hover" => browser_arguments(
            json!({
                "ref": element.clone(),
                "x": { "type": "number", "description": "A point of the viewport, in CSS pixels, with y." },
                "y": { "type": "number" }
            }),
            &[],
        ),
        "select" => browser_arguments(
            json!({
                "ref": element.clone(),
                "values": { "type": "array", "items": { "type": "string" }, "description": "Options by their value." },
                "labels": { "type": "array", "items": { "type": "string" }, "description": "Options by the text shown." }
            }),
            &["ref"],
        ),
        "drag" => browser_arguments(
            json!({
                "from_ref": element.clone(),
                "from_x": { "type": "number", "description": "The start, in CSS pixels, with from_y." },
                "from_y": { "type": "number" },
                "to_ref": element.clone(),
                "to_x": { "type": "number", "description": "The end, in CSS pixels, with to_y." },
                "to_y": { "type": "number" }
            }),
            &[],
        ),
        "upload" => browser_arguments(
            json!({
                "ref": element.clone(),
                "paths": { "type": "array", "items": { "type": "string" }, "description": "Absolute paths of files in an open project." }
            }),
            &["paths"],
        ),
        "dialog" => browser_arguments(
            json!({
                "accept": { "type": "boolean", "description": "OK; false is Cancel." },
                "text": { "type": "string", "description": "What a prompt dialog answers." }
            }),
            &["accept"],
        ),
        "resize" => browser_arguments(
            json!({
                "preset": { "type": "string", "description": "A device: iPhone SE, iPhone 15 Pro, iPhone 15 Pro Max, Pixel 8, Galaxy S24, iPad Mini, iPad Air, iPad Pro 13, Surface Pro 7, Laptop, Desktop." },
                "width": { "type": "integer", "minimum": 320, "maximum": 3840 },
                "height": { "type": "integer", "minimum": 240, "maximum": 2160 },
                "fill": { "type": "boolean", "description": "Back at the tab's own size." }
            }),
            &[],
        ),
        "color_scheme" => browser_arguments(
            json!({
                "scheme": { "type": "string", "enum": ["dark", "light", "system"] }
            }),
            &["scheme"],
        ),
        _ => browser_arguments(json!({}), &[]),
    }
}

/// The input and output schemas of the terminal family's tools (#491).
fn terminal_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "blocks" => terminal_blocks_schemas(),
        "read" => terminal_read_schemas(),
        "find" => terminal_find_schemas(),
        "screen" => terminal_screen_schemas(),
        "type" => terminal_type_schemas(),
        "run" => terminal_run_schemas(),
        _ => terminal_list_schemas(),
    }
}

/// The schema of the `terminal` argument, a terminal's id.
fn terminal_argument_schema() -> Value {
    json!({
        "type": "integer",
        "description": "The terminal's id, from terminal_list; the terminal this call comes from when left out."
    })
}

/// `terminal_list`: no arguments; each terminal.
/// `ports_list` (#521): no arguments; each listener, never its command line, which can carry a
/// token.
fn ports_list_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "ports": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "project": { "type": "string" },
                            "folder": {
                                "type": "string",
                                "description": "The project folder that holds the process's working directory."
                            },
                            "address": { "type": "string" },
                            "port": { "type": "integer" },
                            "url": { "type": "string" },
                            "pid": { "type": "integer" },
                            "name": { "type": "string", "description": "The process's name." },
                            "cwd": { "type": "string" }
                        },
                        "required": ["project", "folder", "address", "port", "url", "pid", "name", "cwd"]
                    }
                }
            },
            "required": ["ports"]
        }),
    )
}

/// `docs_search` and `docs_read` (#681): a query, or a page and a heading; sections, or a page of
/// text.
fn docs_schemas(verb: &str) -> (Value, Value) {
    if verb == "search" {
        return (
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string", "minLength": 1, "description": "What to look for, in words." }
                },
                "required": ["query"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "query": { "type": "string" },
                    "matched": { "type": "integer", "description": "How many sections hold a word of the query." },
                    "results": {
                        "type": "array",
                        "items": {
                            "type": "object",
                            "properties": {
                                "page": { "type": "string", "description": "What docs_read takes as `page`." },
                                "heading": { "type": "string", "description": "What docs_read takes as `heading`." },
                                "snippet": { "type": "string" },
                                "words": { "type": "integer", "description": "How many of the query's words the section holds." }
                            },
                            "required": ["page", "heading", "snippet", "words"]
                        }
                    }
                },
                "required": ["query", "matched", "results"]
            }),
        );
    }
    (
        json!({
            "type": "object",
            "properties": {
                "page": { "type": "string", "description": "A page from docs_search: `zed/<path>.md` or `marley/guide.md`." },
                "heading": { "type": "string", "description": "One section of the page, by its heading; the whole page when left out." },
                "after": { "type": "integer", "minimum": 1, "description": "Read from the line after this one: a page's `next`." }
            },
            "required": ["page"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "page": { "type": "string" },
                "heading": { "type": ["string", "null"] },
                "headings": { "type": "array", "items": { "type": "string" }, "description": "The page's headings, for reading a section." },
                "text": { "type": "string" },
                "first_line": { "type": "integer" },
                "last_line": { "type": "integer" },
                "total_lines": { "type": "integer" },
                "next": { "type": ["integer", "null"], "description": "The `after` that reads the rest; null at the end." }
            },
            "required": ["page", "text", "first_line", "last_line", "total_lines", "next"]
        }),
    )
}

/// `settings_schema` and `settings_read` (#681): a key path; what the setting is, or what each
/// settings file sets it to.
fn settings_schemas(verb: &str) -> (Value, Value) {
    if verb == "change" {
        return settings_change_schemas();
    }
    let input = json!({
        "type": "object",
        "properties": {
            "key": { "type": "string", "description": "The key path in settings.json, dots between keys: `terminal.font_size`." }
        },
        "required": ["key"],
        "additionalProperties": false
    });
    if verb == "schema" {
        return (
            input,
            json!({
                "type": "object",
                "properties": {
                    "key": { "type": "string" },
                    "type": { "type": ["string", "null"] },
                    "description": { "type": ["string", "null"] },
                    "values": { "type": "array", "description": "The values it allows, when it lists them." },
                    "default": { "description": "The default from Marley's default settings; null when none." },
                    "keys": {
                        "type": "array",
                        "description": "For an object, its keys, each with the first line of its description.",
                        "items": { "type": "object" }
                    }
                },
                "required": ["key"]
            }),
        );
    }
    (
        input,
        json!({
            "type": "object",
            "properties": {
                "key": { "type": "string" },
                "effective": { "description": "The value in effect: the winning file's, or for an object the merged one." },
                "global": { "description": "The value where no project file sets the key: the user's settings over the defaults." },
                "set_in": { "type": ["string", "null"], "description": "The file whose value wins." },
                "values": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "file": { "type": "string" },
                            "value": {}
                        },
                        "required": ["file", "value"]
                    }
                }
            },
            "required": ["key", "effective", "set_in", "values"]
        }),
    )
}

/// `settings_change` (#682): a key path and a value; what was written.
fn settings_change_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "key": { "type": "string", "description": "The key path in settings.json, dots between keys: `terminal.font_size`." },
                "value": { "description": "The JSON value to set." }
            },
            "required": ["key", "value"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "result": { "type": "string", "enum": ["applied", "unchanged"] },
                "key": { "type": "string" },
                "file": { "type": "string" },
                "before": { "description": "The value before; null when the file did not set it." },
                "after": {}
            },
            "required": ["result", "key", "file", "before", "after"]
        }),
    )
}

/// `keymap_change` (#686): keystrokes, an action, a context and arguments; the binding added.
fn keymap_change_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "keystrokes": { "type": "string", "description": "As Zed's keymap writes them: `ctrl-alt-m`, or a sequence `cmd-k cmd-s`." },
                "action": { "type": "string", "description": "The action's name, from actions_list: `workspace::Save`." },
                "context": { "type": "string", "description": "Where the binding applies: `Workspace`, `Editor`; every context when left out." },
                "arguments": { "description": "The action's arguments, for an action that takes them." }
            },
            "required": ["keystrokes", "action"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "result": { "type": "string", "enum": ["applied"] },
                "keystrokes": { "type": "string" },
                "action": { "type": "string" },
                "context": { "type": ["string", "null"] },
                "file": { "type": "string" }
            },
            "required": ["result", "keystrokes", "action", "file"]
        }),
    )
}

/// The seat family: `seat_add` (#692), and `seat_stop` and `seat_remove` (#710): a seat's name; the
/// harness's answer, with `result`.
fn seat_schemas(verb: &str) -> (Value, Value) {
    if verb == "add" {
        return seat_add_schemas();
    }
    (
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "minLength": 1, "description": "The seat's name, as `seat add` took it." }
            },
            "required": ["name"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "result": { "type": "string", "enum": ["stopped", "removed"] },
                "seat": { "type": "string" },
                "stopped": { "type": "array", "items": { "type": "object" } },
                "supervision_ended": { "type": "array", "items": { "type": "string" } },
                "removed": { "type": "string", "description": "The deleted profile's path." }
            },
            "required": ["result", "seat"]
        }),
    )
}

/// `seat_add` (#692): a seat's name, agent, folder and role; the seat added and starting.
fn seat_add_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "name": { "type": "string", "description": "The seat's name: lowercase letters, digits, `-` and `_`." },
                "agent": { "type": "string", "enum": ["claude", "codex"], "description": "Claude Code or Codex." },
                "cwd": { "type": "string", "description": "The folder the seat works in, on the harness's machine." },
                "role": { "type": "string", "description": "`manager`, or any label; none when left out." }
            },
            "required": ["name", "agent", "cwd"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "result": { "type": "string", "enum": ["starting"] },
                "seat": { "type": "string" },
                "profile": { "type": "string" },
                "kind": { "type": "string" }
            },
            "required": ["result", "seat"]
        }),
    )
}

/// `actions_list` (#681): a query; the actions that hold it, with their keys.
fn actions_list_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "query": { "type": "string", "minLength": 1, "description": "Words the action's name or documentation holds." }
            },
            "required": ["query"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "matched": { "type": "integer" },
                "actions": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "name": { "type": "string" },
                            "palette": { "type": "string", "description": "Its name in the command palette." },
                            "documentation": { "type": ["string", "null"] },
                            "keys": {
                                "type": "array",
                                "items": {
                                    "type": "object",
                                    "properties": {
                                        "keystrokes": { "type": "string" },
                                        "context": { "type": ["string", "null"] }
                                    },
                                    "required": ["keystrokes"]
                                }
                            }
                        },
                        "required": ["name", "palette", "keys"]
                    }
                }
            },
            "required": ["matched", "actions"]
        }),
    )
}

fn terminal_list_schemas() -> (Value, Value) {
    (
        json!({ "type": "object", "properties": {}, "additionalProperties": false }),
        json!({
            "type": "object",
            "properties": {
                "terminals": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "id": { "type": "integer" },
                            "terminal_id": {
                                "type": ["string", "null"],
                                "description": "The MARLEY_TERMINAL_ID the terminal's programs see."
                            },
                            "self": {
                                "type": "boolean",
                                "description": "Whether this call comes from this terminal."
                            },
                            "title": { "type": "string" },
                            "project": { "type": ["string", "null"] },
                            "cwd": { "type": ["string", "null"] },
                            "running": {
                                "type": ["string", "null"],
                                "description": "The command running in the terminal, if any."
                            },
                            "blocks": { "type": "integer" }
                        },
                        "required": ["id", "title", "blocks"]
                    }
                }
            },
            "required": ["terminals"]
        }),
    )
}

/// `terminal_blocks`: a terminal and how many of its newest blocks; each block.
fn terminal_blocks_schemas() -> (Value, Value) {
    let terminal = terminal_argument_schema();
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal,
                "last": {
                    "type": "integer",
                    "minimum": 1,
                    "maximum": 500,
                    "description": "How many of the newest blocks to list; 50 when left out."
                }
            },
            "required": [],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "total": { "type": "integer", "description": "How many blocks the terminal holds." },
                "redacted": {
                    "type": "integer",
                    "description": "How many secrets the listed commands had hidden."
                },
                "blocks": {
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "index": { "type": "integer" },
                            "command": { "type": "string" },
                            "verified": {
                                "type": "boolean",
                                "description": "Whether the shell's own hook reported the command, not output that imitates one."
                            },
                            "running": { "type": "boolean" },
                            "exit_code": { "type": ["integer", "null"] },
                            "cwd": { "type": ["string", "null"] },
                            "started_at_ms": {
                                "type": ["integer", "null"],
                                "description": "When the command started, in Unix milliseconds."
                            },
                            "duration_ms": { "type": ["integer", "null"] },
                            "host": {
                                "type": ["string", "null"],
                                "description": "The ssh host the block's shell ran on (#526); null for the terminal's own shell."
                            },
                            "output_kept": {
                                "type": "boolean",
                                "description": "Whether the output is still in the terminal's scrollback."
                            },
                            "restored": {
                                "type": "boolean",
                                "description": "Whether the block ran before Marley restarted and was drawn again from the terminal's kept output (#764)."
                            }
                        },
                        "required": ["index", "command", "verified", "running", "output_kept"]
                    }
                }
            },
            "required": ["terminal", "total", "blocks"]
        }),
    )
}

/// `terminal_read`: a terminal and a block; the block's output.
fn terminal_read_schemas() -> (Value, Value) {
    let terminal = terminal_argument_schema();
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal,
                "block": { "type": "integer", "description": "The block's index, from terminal_blocks." },
                "before": {
                    "type": "integer",
                    "minimum": 2,
                    "description": "Read the page that ends just before this line: a page's `previous`. Left out, the newest page."
                }
            },
            "required": ["block"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "block": { "type": "integer" },
                "command": { "type": "string" },
                "running": { "type": "boolean" },
                "output": { "type": "string", "description": "The page's lines." },
                "first_line": { "type": "integer", "description": "The page's first line, counted from 1." },
                "last_line": { "type": "integer", "description": "The page's last line." },
                "total_lines": { "type": "integer", "description": "The block's lines so far." },
                "previous": {
                    "type": ["integer", "null"],
                    "description": "The `before` that reads the page before this one; null at the block's first line."
                },
                "line_cut": {
                    "type": "boolean",
                    "description": "Whether the page is one line longer than a page, its start left out."
                },
                "truncated": {
                    "type": "boolean",
                    "description": "Whether the page leaves part of the output out: earlier lines, or the start of a cut line."
                },
                "redacted": {
                    "type": "integer",
                    "description": "How many secrets the command and the whole output had hidden, the pages not shown included."
                }
            },
            "required": [
                "terminal", "block", "command", "running", "output", "first_line", "last_line",
                "total_lines", "previous", "line_cut", "truncated"
            ]
        }),
    )
}

/// `terminal_screen` (#525): a terminal; its screen and who controls it.
fn terminal_screen_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": { "terminal": terminal_argument_schema() },
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "rows": { "type": "array", "items": { "type": "string" } },
                "cursor": {
                    "type": "object",
                    "properties": {
                        "row": { "type": "integer" },
                        "column": { "type": "integer" }
                    }
                },
                "columns": { "type": "integer" },
                "lines": { "type": "integer" },
                "alternate_screen": { "type": "boolean" },
                "scrolled": {
                    "type": "boolean",
                    "description": "Whether the user scrolled back, so the rows are not the bottom of the screen."
                },
                "program": { "type": ["string", "null"] },
                "generation": { "type": "integer" },
                "taken_over": { "type": "boolean" },
                "approval": { "type": "string", "enum": ["ask_first_write", "ask_every_write", "never_ask"] },
                "approved": { "type": "boolean" },
                "redacted": { "type": "integer" }
            },
            "required": ["terminal", "rows", "generation", "taken_over", "approval", "approved"]
        }),
    )
}

/// `terminal_type` (#525): a terminal, its generation, and what to type; what was typed.
fn terminal_type_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal_argument_schema(),
                "generation": {
                    "type": "integer",
                    "description": "The terminal's generation, from terminal_screen."
                },
                "text": { "type": "string", "description": "Text to type, as a paste." },
                "keys": {
                    "type": "array",
                    "items": { "type": "string" },
                    "description": "Keys by name after the text: escape, ctrl-c, up, tab."
                },
                "submit": { "type": "boolean", "description": "Press Enter last." }
            },
            "required": ["generation"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "written": { "type": "integer" },
                "program": { "type": "string" },
                "generation": { "type": "integer" }
            },
            "required": ["written", "program", "generation"]
        }),
    )
}

/// `terminal_run` (#556): a terminal, a command and how long to wait; the block it ran.
/// `editor_list`, `editor_read` and `editor_open` (#704): no arguments, the editors out; an id or a
/// path and the first line in, a page of the text out; a path, a line and a column in, the editor
/// opened out. `editor_edit` and `editor_save` go to [`editor_change_schemas`].
fn editor_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "list" => (
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
            json!({
                "type": "object",
                "properties": { "editors": { "type": "array", "items": { "type": "object" } } },
                "required": ["editors"]
            }),
        ),
        "read" => (
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "The editor's id, from editor_list." },
                    "path": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 4096,
                        "description": "The open file's absolute path, when no id is given."
                    },
                    "start_line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "The first line of the page; 1 when left out."
                    }
                },
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "path": { "type": "string" },
                    "text": { "type": "string" },
                    "first_line": { "type": "integer" },
                    "last_line": { "type": "integer" },
                    "total_lines": { "type": "integer" },
                    "next_line": { "type": ["integer", "null"] },
                    "dirty": { "type": "boolean" },
                    "redacted": { "type": "integer" }
                },
                "required": ["id", "path", "text", "first_line", "last_line", "total_lines"]
            }),
        ),
        "edit" | "save" => editor_change_schemas(verb),
        "diagnostics" | "definition" | "references" | "hover" => editor_lsp_schemas(verb),
        _ => (
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 4096,
                        "description": "The file's absolute path, inside an open project's folders."
                    },
                    "line": { "type": "integer", "minimum": 1, "description": "1 when left out." },
                    "column": { "type": "integer", "minimum": 1, "description": "1 when left out." }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "path": { "type": "string" },
                    "line": { "type": "integer" },
                    "column": { "type": "integer" }
                },
                "required": ["id", "path"]
            }),
        ),
    }
}

/// `editor_edit` and `editor_save` (#705): an editor and the text to replace in, what changed out;
/// an editor in, whether it saved out.
/// The language server tools' schemas (#712): a position, a page start, and their answers.
fn editor_lsp_schemas(verb: &str) -> (Value, Value) {
    let mut position = position_properties();
    let start = page_start();
    let location = json!({ "type": "object" });
    match verb {
        "diagnostics" => editor_diagnostics_schemas(),
        "references" => {
            if let Some(properties) = position.as_object_mut() {
                let _start = properties.insert("start".to_string(), start);
            }
            (
                json!({
                    "type": "object",
                    "properties": position,
                    "required": ["path", "line"],
                    "additionalProperties": false
                }),
                json!({
                    "type": "object",
                    "properties": {
                        "references": { "type": "array", "items": location },
                        "total": { "type": "integer" },
                        "start": { "type": "integer" },
                        "next": { "type": ["integer", "null"] },
                        "redacted": { "type": "integer" }
                    },
                    "required": ["references", "total", "start", "next"]
                }),
            )
        }
        "hover" => (
            json!({
                "type": "object",
                "properties": position,
                "required": ["path", "line"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "text": { "type": "string" },
                    "range": { "type": ["object", "null"] },
                    "cut": { "type": "boolean" },
                    "redacted": { "type": "integer" }
                },
                "required": ["text"]
            }),
        ),
        _ => (
            json!({
                "type": "object",
                "properties": position,
                "required": ["path", "line"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "definitions": { "type": "array", "items": location },
                    "redacted": { "type": "integer" }
                },
                "required": ["definitions"]
            }),
        ),
    }
}

/// A position's properties (#712): the file, the 1-based line, and a symbol on it or a column.
fn position_properties() -> Value {
    let path = json!({
        "type": "string",
        "minLength": 1,
        "maxLength": 4096,
        "description": "The file's absolute path, inside an open project's folders."
    });
    json!({
        "path": path,
        "line": { "type": "integer", "minimum": 1, "description": "The 1-based line." },
        "symbol": {
            "type": "string",
            "minLength": 1,
            "maxLength": 256,
            "description": "A name on that line; its first occurrence is the position."
        },
        "column": {
            "type": "integer",
            "minimum": 1,
            "description": "The 1-based column, as editor_list counts it, when no symbol is given."
        }
    })
}

/// A page's `start` property (#712).
fn page_start() -> Value {
    json!({
        "type": "integer",
        "minimum": 0,
        "description": "The first entry of the page, from the last answer's next; 0 when left out."
    })
}

/// `editor_diagnostics`' schemas (#712).
fn editor_diagnostics_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "path": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 4096,
                    "description": "One file's absolute path; left out, a project's files with errors or warnings."
                },
                "project": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 256,
                    "description": "The project to list when no path is given: its name or a folder of it; the caller's project when left out."
                },
                "start": page_start()
            },
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "diagnostics": { "type": "array", "items": { "type": "object" } },
                "total": { "type": "integer" },
                "start": { "type": "integer" },
                "next": { "type": ["integer", "null"] },
                "redacted": { "type": "integer" },
                "hidden_secret_files": { "type": "integer" }
            },
            "required": ["diagnostics", "total", "start", "next"]
        }),
    )
}

fn editor_change_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "edit" => (
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "The editor's id, from editor_list." },
                    "path": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 4096,
                        "description": "The open file's absolute path, when no id is given."
                    },
                    "old_text": {
                        "type": "string",
                        "minLength": 1,
                        "description": "The text to replace, exactly as the editor holds it now."
                    },
                    "new_text": { "type": "string", "description": "What replaces it." },
                    "replace_all": {
                        "type": "boolean",
                        "description": "Replace every occurrence; false when left out."
                    }
                },
                "required": ["old_text", "new_text"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "path": { "type": "string" },
                    "replaced": { "type": "integer" },
                    "first_line": { "type": "integer" },
                    "dirty": { "type": "boolean" }
                },
                "required": ["id", "path", "replaced", "first_line", "dirty"]
            }),
        ),
        _ => (
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer", "description": "The editor's id, from editor_list." },
                    "path": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 4096,
                        "description": "The open file's absolute path, when no id is given."
                    }
                },
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "id": { "type": "integer" },
                    "path": { "type": "string" },
                    "saved": { "type": "boolean" }
                },
                "required": ["id", "path", "saved"]
            }),
        ),
    }
}

/// `thread_list`, `thread_read`, `thread_post` and `thread_answer` (#706): no arguments, the threads
/// out; a thread and the first line in, a page of its Markdown out; a thread and a message in,
/// whether it was sent out; a thread and allow or reject in, the answer given out.
fn thread_schemas(verb: &str) -> (Value, Value) {
    let thread = json!({ "type": "string", "minLength": 1, "description": "The thread's id, from thread_list." });
    match verb {
        "list" => (
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
            json!({
                "type": "object",
                "properties": { "threads": { "type": "array", "items": { "type": "object" } } },
                "required": ["threads"]
            }),
        ),
        "read" => (
            json!({
                "type": "object",
                "properties": {
                    "thread": thread,
                    "start_line": {
                        "type": "integer",
                        "minimum": 1,
                        "description": "The first line of the page; 1 when left out."
                    }
                },
                "required": ["thread"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "thread": { "type": "string" },
                    "text": { "type": "string" },
                    "first_line": { "type": "integer" },
                    "last_line": { "type": "integer" },
                    "total_lines": { "type": "integer" },
                    "next_line": { "type": ["integer", "null"] }
                },
                "required": ["thread", "text", "first_line", "last_line", "total_lines"]
            }),
        ),
        "post" => (
            json!({
                "type": "object",
                "properties": {
                    "thread": thread,
                    "message": { "type": "string", "minLength": 1, "description": "What to send." }
                },
                "required": ["thread", "message"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": { "thread": { "type": "string" }, "sent": { "type": "boolean" } },
                "required": ["thread", "sent"]
            }),
        ),
        _ => (
            json!({
                "type": "object",
                "properties": {
                    "thread": thread,
                    "allow": {
                        "type": "boolean",
                        "description": "true allows the call once; false rejects it."
                    }
                },
                "required": ["thread", "allow"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "thread": { "type": "string" },
                    "tool": { "type": "string" },
                    "allowed": { "type": "boolean" }
                },
                "required": ["thread", "tool", "allowed"]
            }),
        ),
    }
}

/// `action_list` and `action_run` (#707): no arguments, the actions an agent may run out; an
/// action's name, its arguments and a project in, whether it was dispatched out.
fn action_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "list" => (
            json!({ "type": "object", "properties": {}, "additionalProperties": false }),
            json!({
                "type": "object",
                "properties": {
                    "actions": { "type": "array", "items": { "type": "object" } },
                    "unknown": { "type": "array", "items": { "type": "string" } }
                },
                "required": ["actions", "unknown"]
            }),
        ),
        _ => (
            json!({
                "type": "object",
                "properties": {
                    "name": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 200,
                        "description": "The action's name, such as workspace::ToggleLeftDock."
                    },
                    "arguments": {
                        "type": "object",
                        "description": "The action's arguments, for an action that takes some."
                    },
                    "project": {
                        "type": "string",
                        "description": "The project to run it in, by folder name or path; the active window when left out."
                    }
                },
                "required": ["name"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "name": { "type": "string" },
                    "dispatched": { "type": "boolean" }
                },
                "required": ["name", "dispatched"]
            }),
        ),
    }
}

/// `prompt_open` and `prompt_wait` (#649): a file's absolute path in, the edit's id out; the id
/// and a wait in, whether the edit ended out.
fn prompt_schemas(verb: &str) -> (Value, Value) {
    match verb {
        "open" => (
            json!({
                "type": "object",
                "properties": {
                    "path": {
                        "type": "string",
                        "minLength": 1,
                        "maxLength": 4096,
                        "description": "The file's absolute path."
                    }
                },
                "required": ["path"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": {
                    "edit": { "type": "integer" },
                    "file": { "type": "string" }
                },
                "required": ["edit", "file"]
            }),
        ),
        _ => (
            json!({
                "type": "object",
                "properties": {
                    "edit": { "type": "integer", "minimum": 0 },
                    "wait_seconds": {
                        "type": "integer",
                        "minimum": 1,
                        "maximum": 20,
                        "description": "How long to wait for the tab to close; 20 when left out."
                    }
                },
                "required": ["edit"],
                "additionalProperties": false
            }),
            json!({
                "type": "object",
                "properties": { "closed": { "type": "boolean" } },
                "required": ["closed"]
            }),
        ),
    }
}

fn terminal_run_schemas() -> (Value, Value) {
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal_argument_schema(),
                "command": {
                    "type": "string",
                    "minLength": 1,
                    "maxLength": 4096,
                    "description": "The command, one line, as the user would type it."
                },
                "wait_seconds": {
                    "type": "integer",
                    "minimum": 0,
                    "maximum": 20,
                    "description": "How long to wait for the block to end; 20 when left out."
                }
            },
            "required": ["command"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": {
                "terminal": { "type": "integer" },
                "block": { "type": "integer" },
                "command": { "type": "string" },
                "running": { "type": "boolean" },
                "exit_code": { "type": ["integer", "null"] },
                "duration_ms": { "type": ["integer", "null"] },
                "output": { "type": "string", "description": "The output's newest page." },
                "first_line": { "type": "integer" },
                "last_line": { "type": "integer" },
                "total_lines": { "type": "integer" },
                "previous": {
                    "type": ["integer", "null"],
                    "description": "terminal_read's `before` for the page before this one."
                },
                "line_cut": { "type": "boolean" },
                "truncated": { "type": "boolean" },
                "redacted": { "type": "integer" }
            },
            "required": [
                "terminal", "block", "command", "running", "output", "first_line", "last_line",
                "total_lines", "previous", "line_cut", "truncated", "redacted"
            ]
        }),
    )
}

/// `terminal_find` (#567): a terminal, a block and a query; the matching line, and up to three
/// candidates.
fn terminal_find_schemas() -> (Value, Value) {
    let mut output = found_properties();
    if let Some(properties) = output.as_object_mut() {
        properties.extend([
            ("terminal".to_string(), json!({ "type": "integer" })),
            ("block".to_string(), json!({ "type": "integer" })),
            (
                "line".to_string(),
                json!({
                    "type": ["integer", "null"],
                    "description": "The line's number in the block's output, from 1, when the answer is sure."
                }),
            ),
            (
                "candidates".to_string(),
                json!({
                    "type": "array",
                    "items": {
                        "type": "object",
                        "properties": {
                            "line": { "type": "integer" },
                            "text": { "type": "string" },
                            "probability": { "type": ["number", "null"] }
                        },
                        "required": ["line", "text"]
                    }
                }),
            ),
            (
                "cut".to_string(),
                json!({
                    "type": "boolean",
                    "description": "Whether the model saw less than the block: its newest lines, up to about 24,000 tokens."
                }),
            ),
        ]);
    }
    (
        json!({
            "type": "object",
            "properties": {
                "terminal": terminal_argument_schema(),
                "block": { "type": "integer", "description": "The block's index, from terminal_blocks." },
                "query": query_schema("where the server refused the connection")
            },
            "required": ["block", "query"],
            "additionalProperties": false
        }),
        json!({
            "type": "object",
            "properties": output,
            "required": ["terminal", "block", "query", "source", "sure", "candidates", "cut"]
        }),
    )
}

/// The JSON Schema for a `Receipt<T>` (the #367 tagged `Accepted`/`Refused`).
fn receipt_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "result": { "type": "string", "enum": ["accepted", "refused"] },
            "value": {},
            "reason": { "type": "string" }
        },
        "required": ["result"]
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tool_name_composes_family_underscore_verb() {
        assert_eq!(tool_name(Family::Fleet, "snapshot"), "fleet_snapshot");
        assert_eq!(
            tool_name(Family::Session, "surface_to_human"),
            "session_surface_to_human"
        );
    }

    #[test]
    fn registry_is_exactly_the_l1_set_with_correct_tiers() {
        let names: Vec<String> = registry().iter().map(ToolSpec::name).collect();
        // The set grew past L1's five: the terminal tools (#491, #525, #556), the browser tools
        // (#492 to #586), the finds (#567), `ports_list` (#521), the editor tools (#649) and the
        // docs, settings and actions tools (#681).
        assert_eq!(
            names,
            [
                "fleet_snapshot",
                "session_surface_to_human",
                "terminal_list",
                "terminal_blocks",
                "terminal_read",
                "terminal_find",
                "terminal_screen",
                "terminal_type",
                "terminal_run",
                "browser_tabs",
                "browser_look",
                "browser_snapshot",
                "browser_find",
                "browser_console",
                "browser_network",
                "browser_picks",
                "browser_pick",
                "browser_recordings",
                "browser_recording",
                "browser_draft_test",
                "browser_annotations",
                "browser_annotate",
                "browser_check_pick",
                "browser_navigate",
                "browser_open_url",
                "browser_back",
                "browser_click",
                "browser_type",
                "browser_press",
                "browser_scroll",
                "browser_hover",
                "browser_select",
                "browser_drag",
                "browser_upload",
                "browser_dialog",
                "browser_resize",
                "browser_color_scheme",
                "browser_wait",
                "ports_list",
                "prompt_open",
                "prompt_wait",
                "editor_list",
                "editor_read",
                "editor_open",
                "editor_edit",
                "editor_save",
                "editor_diagnostics",
                "editor_definition",
                "editor_references",
                "editor_hover",
                "thread_list",
                "thread_read",
                "thread_post",
                "thread_answer",
                "action_list",
                "action_run",
                "docs_search",
                "docs_read",
                "settings_schema",
                "settings_read",
                "settings_change",
                "keymap_change",
                "seat_add",
                "seat_stop",
                "seat_remove",
                "draft_message",
                "review_notes_add",
                "actions_list"
            ]
        );
        assert_eq!(registry().len(), 68);
        assert_eq!(
            lookup("fleet_snapshot").expect("read tool").tier,
            Tier::Read
        );
        let write = lookup("session_surface_to_human").expect("write tool");
        assert_eq!(write.tier, Tier::Write);
        assert_eq!(write.grant_class, "session.write");
        assert!(lookup("nope.nope").is_none());
    }

    #[test]
    fn tools_list_derives_from_registry_with_input_and_output_schemas() {
        let list = tools_list();
        let tools = list["tools"].as_array().expect("tools array");
        // REQ-001 / F2: the wire list is EXACTLY the served registry names, in order (no drift).
        let listed: Vec<&str> = tools.iter().map(|t| t["name"].as_str().unwrap()).collect();
        // A conditional tool is listed only while it is enabled, and none is here (#567).
        let registered: Vec<String> = registry()
            .iter()
            .filter(|spec| spec.family.is_served())
            .map(ToolSpec::name)
            .filter(|name| !CONDITIONAL_TOOLS.contains(&name.as_str()))
            .collect();
        assert_eq!(listed, registered);
        // REQ-001: each advertises an inputSchema AND an outputSchema.
        for tool in tools {
            assert!(tool["inputSchema"].is_object());
            assert!(tool["outputSchema"].is_object());
        }
        // the blocks tool's input takes a `terminal`, the calling terminal when left out (#520).
        let blocks = tools
            .iter()
            .find(|t| t["name"] == "terminal_blocks")
            .expect("blocks tool");
        assert!(blocks["inputSchema"]["properties"]["terminal"].is_object());
    }
}
