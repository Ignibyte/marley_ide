# Clickable links — Notes

- **Forge ticket:** #196 (b76e42db-b8b2-4e44-8ecc-d2bc5e65c1ae)
- **AAR:** 189c5b14-1076-4161-b65f-5aa88cd632fe
- **Local ticket doc:** docs/planning/tickets/open/TICKET-196-clickable-links.md
- **Pipeline spec:** clickable-links.spec.md

## Phase 1 — Plan
- **Request:** URLs + file paths in terminal block output become clickable (URL → browser,
  file → code view). Front half of the M13 terminal↔editor wedge (#212).
- **Classification / tier:** work pipeline, medium. Systems: a new PURE `marley_app::links`
  scanner (cov/MSI 100) + an app.rs render/click shim + a small `marley_command` open-url
  adapter (spawn confined). Wedge dep for #212.
- **Forge recall (§19):** relied on in-repo seams + the #190 lesson (relative-path-vs-cwd:
  the `.app` launch has cwd=/, so a project-relative path must go through `resolve_under_root`
  before `open_file_in_viewer` reads it — the file-link open reuses that fixed path). Security
  memory: the #42 paste-injection lesson (sanitize before a process boundary) applies to the
  URL→opener spawn.
- **Discovery (confirmed in code):**
  - Output render: each output line is painted as a `flex_row` of styled runs — `for run in line { run_paint(run, &palette) → a colored div }` (app.rs:4277 main path; the block-loop output render at ~4451-4468). Links must overlay these runs.
  - `StyledRun` (terminal_blocks/src/styled.rs:15) = text + color/bold. **No hyperlink field** → **OSC 8 is NOT captured today** → deferred (a separate terminal_blocks grid→run change).
  - `open_file_in_viewer` (app.rs:1892) + `resolve_under_root` (imported app.rs:25) — reuse for the file-link open.
  - No existing URL/open adapter in `marley_command` → add a small one (platform opener, spawn confined).
  - `text_selection.rs` provides the block-row selection geometry the link render must not break.
- **Decisions:** D1 pure `scan_links` (textual, no fs stat); D2 OSC 8 deferred; D3 URL via a
  new marley_command adapter + file via open_file_in_viewer; D4 overlay links on run_paint w/o
  breaking selection; D5 autonomous, hold push.
- **Follow-up to file at complete:** OSC 8 explicit-hyperlink honoring (terminal_blocks captures
  the cell hyperlink → StyledRun carries it → the scanner/render prefer it over the text heuristic).

## Phase 2 — Design

**OSC 8 defer — CONFIRMED.** `terminal_blocks::StyledRun` (styled.rs:15) = `{ text, fg, bg, flags }`
(alacritty Color/Flags), coalesced by `(fg,bg,flags)` only; `grid_styled_rows`/`output_styled`
carry no hyperlink. OSC 8 needs a StyledRun hyperlink field + grid plumbing = a distinct
terminal_blocks change → deferred (filed at complete). This slice = text-scan only.

**Architecture / approach.** Two PURE fns (new `marley_app::links`) + a `marley_command` open-url
adapter + an app.rs render/click overlay. Output lines render as a `flex_row` of styled-run divs
(`for run in line { run_paint → colored div }`, block-output path app.rs ~4451-4468; there is NO
per-row click today). The line's text = the in-order concatenation of its `StyledRun.text`, so a
link's byte offsets (scanned over that text) align with the runs. The overlay does **link-aware run
splitting**: for each run, split it at link boundaries and paint each sub-segment with the run's
existing color + (if it's a link) an underline + a click handler. Character content is IDENTICAL to
today → the #31 colors are preserved and text-selection geometry (row/col over the line, `text_selection.rs`)
is unaffected; only div boundaries multiply.

**Pure model + fns (`crates/marley_app/src/links.rs`):**
```
pub enum LinkTarget { Url(String), File(PathBuf) }        // #212 later: File gains Option<(line,col)>
pub struct Link { pub range: Range<usize>, pub target: LinkTarget }  // byte range into the line
pub fn scan_links(line: &str) -> Vec<Link>                // ordered, non-overlapping
pub struct RunSegment { pub range: Range<usize>, pub link: Option<usize> }  // link = index into the line's links
pub fn split_run_by_links(run: Range<usize>, links: &[Link]) -> Vec<RunSegment>  // the render overlay primitive
```
- **URL matcher:** find `https?://` then extend to the first whitespace/control; TRIM trailing
  `.,;:!?` and a trailing `)]}>` UNLESS it balances an opener inside the match (the linkify
  balanced-paren rule). v1 schemes = http/https (others deferrable).
- **File-path heuristic:** a whitespace-delimited token that (a) contains `/` (abs or rel) OR (b)
  ends in a known code extension (`.rs .toml .json .md .sh .txt .lock .js .ts .py .yml .yaml .c .h .go` …);
  strip a trailing `:line` / `:line:col` from the token (the File target is the clean path; #212
  captures line/col); trim trailing punctuation. NO filesystem stat (the shim's `open_file_in_viewer`
  already guards existence). URLs are matched FIRST; paths are scanned only in the non-URL gaps → no
  overlap. Result sorted by `range.start`.
- `split_run_by_links`: intersect a run's byte span with the link ranges → segments each tagged
  `Some(link_idx)` or `None`, covering the run exactly, in order. Pure range geometry.

**Open-url adapter (`crates/marley_command`):**
- `pub fn is_openable_url(url: &str) -> bool` — PURE scheme allowlist (http/https only; reject
  `file:`/`javascript:`/`data:`/empty). cov/MSI 100.
- `pub fn open_url(url: &str) -> io::Result<()>` — guard with `is_openable_url` (Err on reject, NO
  spawn), else spawn the platform opener with the URL as a SINGLE argv (`open <url>` macOS /
  `xdg-open` linux) via `std::process::Command` directly — **no shell**, so nothing to inject into
  (#42 lesson: single arg, no shell; the scheme guard is defense-in-depth). Spawn is the masked shim.

**Render/click overlay (`app.rs`, block-output path ~4451-4468, shim):**
- Per output line: `let text = line.iter().map(|r| &r.text).collect(); let links = scan_links(&text);`
  then walk runs with a running byte cursor; for each run `split_run_by_links(cursor..cursor+run.len, &links)`;
  paint each segment (run color + bold; if `link.is_some()` add `.underline()` + hover + an
  `on_mouse_down` that dispatches open). A URL segment → `marley_command::open_url`; a File segment →
  `open_file_in_viewer(resolve_under_root(&root, &path))` (#190). Scope = BLOCK output only; alt-screen
  (4276-4286, a live program's screen — it may own the mouse) is deferred (note).

**File manifest.**
- ADD `crates/marley_app/src/links.rs` — `Link`/`LinkTarget`/`RunSegment` + `scan_links` + `split_run_by_links` (+ tests in Phase 4).
- `crates/marley_app/src/lib.rs` (or the module-decl site) — `mod links;`.
- `crates/marley_command/src/lib.rs` — `is_openable_url` (pure) + `open_url` (shim).
- `crates/marley_app/src/app.rs` — the block-output render overlay + the two click dispatches; `use crate::links::{scan_links, split_run_by_links, LinkTarget};` + `use marley_command::open_url;`.

**Regression Test Plan.**
| REQ | test |
|---|---|
| REQ-001 | `scan_links` URL fixtures: `"x https://a.io/p."` → Url span excludes the `.`; `"(https://a.io)"` → excludes the `)`; `"https://a.io/f(1)"` → keeps the balanced `)` |
| REQ-002 | `scan_links` path fixtures: `crates/app.rs` → File; `/etc/hosts` → File; `app.rs:42:10` → File("app.rs") (line:col stripped); a plain word → no link |
| REQ-003 | `scan_links` multi: `"a.rs https://x.io b.toml"` → [File(a.rs), Url(x.io), File(b.toml)] ordered, non-overlapping |
| REQ-004 | driven capture — echo a URL + a path into a block; click each → URL opens / file opens in the code view |
| REQ-005 | `split_run_by_links` fixtures (run fully inside a link, run crossing a link start/end, run with no link, a link fully inside one run, two links in a run) + driven (drag-select a line with a link still selects/copies) |
| REQ-006 | `is_openable_url` unit (http/https → true; file/javascript/data/empty → false) + review: `grep` shows spawn only in marley_command |
- Uncoverable-by-unit → driven capture: the render overlay + click dispatch + the actual spawn (REQ-004; the OS opener). `scan_links`/`split_run_by_links`/`is_openable_url` carry the logic at cov/MSI 100.

**Risks / decisions.**
- **R1 linkify boundaries** — the classic hard part; nailed by explicit trim + balanced-paren rules + many fixtures.
- **R2 selection composition** — link-aware splitting keeps identical characters, so `text_selection` row/col geometry is unchanged; extra div boundaries don't affect it. Driven-verify.
- **R3 security** — `open_url` passes the URL as a single argv via `Command` (no shell) + a scheme guard; the scanner only emits http/https anyway. Inspect will probe a crafted URL.
- **R4 scope** — block output only (not alt-screen; not the code view). `:line:col` precision + editor placement is #212. OSC 8 is the deferred follow-up.
- **R5 perf** — `scan_links` runs per visible output line per frame; linear over a short line, fine (memoize later if needed).

## Phase 3 — Implement
Built to the manifest, 4 files:
- **`marley_app/src/links.rs`** (new, PURE) — `LinkTarget::{Url,File}`, `Link`, `RunSegment`;
  `scan_links` (per-token: `url_link` first, else `path_link`) + `split_run_by_links` (range
  geometry); helpers `tokens`, `trim_url` (balanced-paren linkify rule), `path_link`/`trim_path_end`/`strip_line_col`/`looks_like_path`. No IO. `mod links;` added to lib.rs.
- **`marley_command/src/lib.rs`** — `is_openable_url` (pure http/https guard) + `open_url`
  (delegates to) `open_url_with(program, url)` (the TESTABLE seam — guard then spawn `program <url>`
  as a single argv via `blocking::Command`, no shell) + `default_opener()` (cfg! macos/linux). The
  injectable `program` lets tests spawn a harmless binary (e.g. `true`) instead of a browser → both
  branches coverable, NO mutants::skip needed.
- **`marley_app/src/app.rs`** — `use crate::links::{scan_links, split_run_by_links, LinkTarget};`;
  a shim `open_link_target(&LinkTarget)` (URL → `marley_command::open_url`; File →
  `open_file_in_viewer(resolve_under_root(root, path))`), inserted ABOVE `open_file_in_viewer`'s
  doc/skip (skip-detach-safe); the block-output render (~4487) now scans links per line + splits
  each run via `split_run_by_links`, painting link segments with `.underline()` + hover(accent) +
  an `on_mouse_down` → `open_link_target`. Non-link lines paint identically (one whole-run segment).
- **Deviations:** (1) added `open_url_with` + `default_opener` (not named in the manifest) — the §14
  injectable-IO pattern so the spawn is testable without opening a browser (no skip/exclude). (2)
  BLOCK output only (alt-screen deferred, per design).
- `cargo fmt` + `cargo check -p marley_command -p marley` clean (zero warnings).

## Phase 3.5 — Inspect
Three general-purpose critics over the diff (correctness/UTF-8 boundaries, security/spawn,
mutation/coverage). All three used COMPILED probes / ran `cargo mutants --list`.

| # | Finding | Verdict | Action |
|---|---|---|---|
| 1 | [MED, correctness] link `on_mouse_down` + `stop_propagation` opens on PRESS and swallows the drag-select anchor (can't start a selection on a link) | **Real** | **FIXED** — link span now uses `.id()` + `.on_click` (fires on release), no `stop_propagation`; a drag that starts on a link sets the pane anchor + selects. app.rs render. |
| 2 | [MED, security] `open_file_in_viewer` does `std::fs::read` (whole file) BEFORE the size guard → a clicked `/dev/zero`/fifo/huge file freezes/OOMs the UI thread; #196 makes it attacker-reachable from output | **Real (latent, #196 exposes)** | **FIXED** — `metadata` first: reject non-regular-file + oversize BEFORE reading; the read is then bounded. app.rs:1909. |
| 3 | [LOW, correctness] a malformed `https://` (empty host) falls through to a `File` link | **Real** | **FIXED** — scan_links: a `://` token never falls through to `path_link`. |
| 4 | [LOW, correctness] a bare `.rs` (no stem) linkifies as a file | **Real** | **FIXED** — `looks_like_path` requires a non-empty stem before the extension. |
| 5 | [LOW, security] absolute / `../` file paths from output are viewable (e.g. `/etc/passwd`) | **Accepted (reason)** | NOT confined — read-only, the user's own UID, on-screen only (no exfil sink); hard-confining would break legit absolute-path links (compiler errors → the #212 wedge). The #2 stat-guard neutralizes the dangerous cases (devices/fifos/huge). |
| — | [HIGH-mut] `split_run_by_links:82 >`→`>=` is a near-equivalent (only a degenerate zero-width link distinguishes it) | **Test-design** | Phase 4 fixture **S4** `split_run_by_links(0..10, &[Link{5..5}])` kills it via timeout (cargo-mutants counts timeouts as caught) — no source change / no exclusion. |

**Clean lenses (concrete):** UTF-8 char-alignment of every emitted byte range (compiled probe, emoji/CJK adjacent to links → 0 panics); `split_run_by_links` termination + exact coverage (proven); URL scheme-bypass (scan_links only emits http/https; `open_url` re-guards); arg-injection (single argv, no shell — `blocking::Command` is a transparent passthrough); text-selection composition (row_len is summed from the runs, independent of how many child spans a row has; the tint is on `line_row`); clean-room/secrets.

**Phase-4 MSI-100 fixture set (from the mutation critic — 55 mutants, 4 unviable `Default::default()`):**
- `scan_links` (14): F1 `http://x.com`→Url; F2 `http://x.com.`→trims `.`; F3 `http://x.com)`→trims unbalanced `)`; F4/F5/F6 `http://a/(x)|[x]|{x}`→keeps BALANCED bracket; F7 `see (http://x.co)`→Url @5..16 (pins start>0 + scheme>0); F8 `src/main.rs`→File; F9 `see [src/main.rs]`→File @5..16 (lead bracket); F10 `src/main.rs:10`→strips `:10`; F11 `src/main.rs:foo`→KEEPS `:foo`; F12 `main.rs`→File (ext-only); F13 `src/main`→File (slash-only); F14 `xyzzy`→[]; + my-fix negatives `http://`→[] and `.rs`→[].
- `split_run_by_links` (4): S1 `(0..5,[])`; S2 `(0..4,[2..4])` (leading gap); S3 `(0..10,[5..7])` (both sides); S4 `(0..10,[5..5])` (degenerate → timeout, the HIGH).
- `marley_command` (7): C1/C2/C3 is_openable_url http/https/reject; C4 `#[cfg(macos)] default_opener()=="open"`; C5 `open_url("file:///x").is_err()`; C6 `open_url_with("true","file:///x").is_err()`; C7 `open_url_with("true","http://example.com").is_ok()` (spawns harmless `true`, no browser).

Compile clean after all fixes. **Lessons to capture:** (a) guard file IO by `stat` BEFORE `read`, especially on an attacker-influenced path (a fifo/device blocks/OOMs a bare read); (b) an inline clickable element should open on `on_click` (release) not `on_mouse_down`, and must not `stop_propagation` if it overlaps a drag-select region.

## Phase 4 — Validate
**Tests added:** `links.rs` — `scan_links_urls_trim_and_balance` (F1-F7 + `http://x.com>` + `http://`→[]),
`scan_links_paths_and_negatives` (F8-F13 + `xyzzy`/`.rs`→[]), `scan_links_multiple_ordered` (REQ-003),
`split_run_by_links_segments` (S1-S4 incl. the S4 degenerate-link timeout-kill). `marley_command` —
`open_url_guards_scheme_then_spawns` (C1-C7; C7 spawns harmless `true`, no browser).

**Tests RUN:** `cargo nextest run -p marley -p marley_command` (targeted) → pass; `--workspace` → **779 passed, 5 skipped**.

**Live driven capture** (`196-2-echo.png`): `echo see https://example.com and src/app.rs` → the output line
renders **`https://example.com`** and **`src/app.rs` UNDERLINED** (clickable), "see"/"and" plain. The
styled render is intact; selection geometry is unaffected (proven by the critic + the run-independent row_len).

**Gate:** `scripts/gates.sh --diff` → **GATE GREEN [diff], 15/15.** gate:5 mutation MSI 100% (all scanner +
adapter mutants killed, incl. the near-equivalent `>=` via the S4 timeout), gate:4 coverage 100%, gate:2 clippy clean.

**Two mid-validate gate fixes (source, no suppressions):** (1) `trim_url` `loop`→`while let` (clippy);
(2) coverage: added the `http://x.com>` fixture (the `'>'`→`'<'` arm) + changed `default_opener` from `cfg!()`
to `#[cfg]` attributes so the `xdg-open` arm is compiled OUT on the macOS runner (dead-line-on-single-OS →
was uncounted; cross-platform-coverage lesson).

**Pre-existing:** none (only the upstream `block v0.1.6` dep warning).

## Phase 5 — Complete
- …
