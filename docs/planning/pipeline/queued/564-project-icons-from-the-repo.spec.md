---
pipeline_id: 31691c29-1a1f-4b85-92e8-40abc218bdd1
ticket: docs/planning/tickets/open/TICKET-564-project-icons-from-the-repo.md
status: QUEUED — Phase 1 Plan drafted; ready to promote to active
title: "Project icons from the repository's own files"
type: feature
slice: workbench shell (the rail's project headers, after #468); the Orca second pass, smaller item 5
references: [docs/planning/design-notes/orca-second-pass-2026-09-25.md, docs/marley/workbench-shell.md, docs/planning/pipeline/completed/468-rail-rows-after-warps-tab-list.spec.md, docs/planning/pipeline/queued/504-browser-tabs-in-the-rail.spec.md]
---

## Title
The rail's project headers gain the repository's own icon: a favicon or logo at the paths
projects keep them, or the icon the project's `index.html` declares, read from disk once per
project and again when the file changes, drawn at 16 px before the name. Several projects and
their worktrees then tell apart at a glance. Nothing is fetched.

## Scope
### In
- **The search** (`crates/marley_workbench/src/project_icons.rs`, new): for a project's root,
  Orca's fifteen names in its order (`favicon`, `public/favicon`, `app/favicon`, `app/icon`,
  `src/favicon`, `src/app/icon`, `assets/favicon`, `assets/icon`, `static/favicon`, `logo`,
  `public/logo`, `public/icon`, `src-tauri/icons/icon`, `app-icon`, `icon`), each as `.png`,
  `.svg`, `.webp`, then `.ico`; then the icon declared by `index.html`, `public/index.html` or
  `src/index.html` (`<link rel="icon">` or `rel="shortcut icon"`, any attribute order and
  quoting; the `href` with its query and fragment dropped, resolved against the file's folder,
  then `public/`, then the root; an `href` with a scheme, a leading `//`, `..` or `data:` is
  skipped). The first file that reads and decodes wins; one that fails is skipped with a log
  line and the search goes on.
- **The read and decode**, off the main thread: `std::fs::read` inside
  `cx.background_spawn(futures::future::lazy(..))`, a file over 256 KiB skipped; rasters decoded
  with the `image` crate (`guess_format`: PNG, WebP, ICO, JPEG, GIF's first frame, BMP) and
  resized to 32 px on the long side; an SVG rendered at 32 px with `resvg` and `usvg`; the result
  a BGRA `gpui::RenderImage`, so the atlas gets 32 px whatever the file held.
- **The refresh**: the rail's project subscription (`follow_folders`) also handles
  `project::Event::WorktreeUpdatedEntries`; when a changed entry is a candidate path, an
  `index.html` of the three, or the file the last search chose, the search runs again for that
  project, at most once a second per project.
- **The drawing**: `render_project_row` draws the icon between the disclosure and the name
  (`img(ImageSource::Render(..))`, 16 px square, `rounded_sm`, `flex_none`); a project without
  one keeps today's header. The icons live in a map on the `Rail`, keyed by the group's first
  path, filled when a group first shows and dropped when it goes; the pure model in
  `marley_rail` is unchanged, and the row's icon is looked up by its group at render time.
- **Worktree rows** (#510) draw their project's icon when #510 has landed.
- `script/e2e/564-project-icons-from-the-repo.sh`.

### Out (explicitly deferred)
- Anything fetched: Orca's `package.json` homepage through Google's favicon service and the
  GitHub owner's avatar. Marley makes no request on a project's behalf.
- A user-chosen icon per project, and a setting that turns icons off: both are Zed settings
  touches, and whether a repository's favicon reads well at 16 px wants Chad's eye first.
- Orca's fallbacks (an emoji, a Lucide glyph, a letter): a header without an icon is unchanged.
- The rail switcher and Zed's title bar.
- Remote (SSH) projects: the files are on another machine (`Project::is_local` false, no search).
- Orca's TSX route files (`app/routes/__root.tsx` and three siblings) as declaring sources: the
  three HTML files cover the frameworks that write a favicon at all.
- Animated icons: a GIF's first frame only.

## Reference (§20)
Orca's repository icon auto-detection (the second pass, smaller item 5;
`src/main/repo-icon-autodetect.ts` with `repo-icon-file-detection.ts`, `repo-icon-source-href.ts`
and `repo-icon-href-candidates.ts`; the renderer `repo-icon.tsx`): the repository's own files
first, then the `package.json` homepage through Google's favicon service, then the GitHub
avatar, drawn at 16 px in the sidebar's section header with a folder glyph as the fallback.
Marley takes the first leg, adds SVG and ICO (Orca reads PNG and WebP only) and leaves the two
fetches out. Upstream Zed draws no per-project icon anywhere: the project panel's root gets the
generic folder icon (`file_icons::get_folder_icon`), the title bar's project name is text, and
the Threads Sidebar's project header carries a remote-host icon only (`sidebar.rs:2405-2408`).
The header's layout is #468's, after Warp's tab list, whose docs show no project icons; the
icon itself is Marley-specific.

### Prior art
- **Behavior maps and reports.** The second pass, item 5 (the usual paths, the declared icon,
  1,190 lines in Orca, "Orca also sends the `package.json` homepage to Google's favicon service
  and fetches GitHub avatars, which Marley should not do"). Report 05 §2.19 ("Icons": Orca's
  fixed Lucide table; Zed's icon themes are ahead) and §2.5 (the sidebar's cards carry the
  project icon). Orca's files, read at `1c2cf120e3`: the fifteen names tried as `.png` then
  `.webp`, probed six at a time, the first in list order winning
  (`repo-icon-file-detection.ts:12-37, 148-174`); the declaring files in order, `index.html`,
  `public/index.html`, four TSX route files, `src/index.html`, files over 256 KiB skipped
  (39-51, 187); PNG and WebP by magic bytes (53-95); `MAX_REPO_ICON_UPLOAD_BYTES` 256 KiB
  (`shared/repo-icon.ts:10`); the `<link rel="icon"|"shortcut icon">` regex and the TSX
  object-literal fallback (`repo-icon-source-href.ts:2-46`); `iconHrefCandidates` (schemes and
  `//` skipped, query and hash stripped, `..` refused; the file's folder, then `public/`, then the
  root; `repo-icon-href-candidates.ts:3-38`); the homepage favicon
  (`https://www.google.com/s2/favicons?domain=…&sz=64`, `repo-icon.ts:17-32`) and the avatar
  (`https://github.com/<owner>.png?size=64`, 51-60); detection once, when a repository is added,
  the icon kept on the repository record as a data URI; the renderer's `<img>` with
  `object-contain` and a Folder fallback (`repo-icon.tsx:130-186`), 16 px in
  `sidebar/worktree-list/rows/SectionHeader.tsx:319-324`.
- **Published material.** The HTML standard's link type `icon` (and `shortcut icon`, the legacy
  form browsers accept) and `favicon.ico` at a site's root as the browsers' fallback; the ICO
  container, one entry per size, which the `image` crate reads by taking the entry with the
  most bits per pixel and then the largest area.
- **The code we already ship.** gpui's `img` (`crates/gpui/src/elements/img.rs:201`) takes
  `ImageSource::Render(Arc<RenderImage>)` (46) and `Image(Arc<Image>)` (48); a path
  (`Resource::Path`, `asset_cache.rs:71-78`) is cached by its string for good (`App::remove_asset`,
  `app.rs:2748`, is the only way out), which is why the icon comes from bytes Marley reads and
  hashes itself. Decoding runs `image::guess_format` (img.rs 669) then `decode_static_image`
  (`platform.rs:2913`); an SVG goes through `svg_renderer.render_single_frame` (739-743) at twice
  its own size, while `svg()` paints a one-colour mask (`svg.rs:150-182`), so a colour icon
  needs `img`. `paint_image` uploads the whole decoded frame (`window.rs:4879-4915`): a 1024 px
  favicon drawn at 16 px would cost a 1024 px texture, hence the resize before
  `RenderImage::new` (`crates/gpui/src/assets.rs:61`). `StyledImage::object_fit` (img.rs 159),
  `Contain` by default. The `image` crate 0.25.10 with `ico`, `png`, `webp`, `jpeg`, `gif` and
  `bmp` on (root `Cargo.toml:669-685`); `resvg` and `usvg` 0.46, workspace dependencies gpui
  builds (`crates/gpui/Cargo.toml:81-82`), `resvg` re-exporting `tiny_skia`
  (`resvg-0.46.0/src/lib.rs:17`). Marley's decoder pattern: `crates/marley_browser/src/frame.rs:15-28`
  (`image::load_from_memory_with_format`, RGBA to BGRA, `RenderImage::new`);
  `marley_browser/Cargo.toml:18` has `image`, `marley_workbench` has `gpui`, `ui`, `project`,
  `util` and `fs` and none of the three. Zed's `project::image_store::create_gpui_image`
  (`crates/project/src/image_store.rs:933-951`) maps `guess_format` to `gpui::ImageFormat`, ICO
  included. The rail: `render_project_row` (`rail.rs:1067`; the frame 1089, the disclosure
  1094-1106, the label 1107-1112), `build_snapshot` (1805) with `group_names` (2065) reading
  `group.key.path_list().paths()` (2068), `GroupEntry` (155), `sync_subscriptions` (440) and its
  project subscription `follow_folders` (469-471); `project::Event::WorktreeUpdatedEntries`
  (`crates/project/src/project.rs:374`, emitted at 3998). The house rule for file reads:
  `std::fs` inside `cx.background_spawn(futures::future::lazy(..))` (`autosuggest.rs:120-145`,
  `claude_plugin.rs:139-153`), since `blocking_io_on_foreground` is an error in Marley crates.
  #504's queued plan keeps a page's favicon as `Favicon { origin, image: Arc<gpui::Image> }`
  (its notes, 79) and a `favicon::format(bytes)` (88): this ticket's decoder can serve its rows.
  Does a crate we build own the seam? gpui owns decoding and drawing and `image` the resize;
  nothing owns "find a repository's icon", which is the one new module.

## UI proof
UI-AFFECTING (the rail's project headers). `script/e2e/564-project-icons-from-the-repo.sh`
(Hyprland; keys only). Setup: a scratch repository with no icon file; under `$E2E_WORK/fixtures`,
written by a few lines of Python with `zlib` and `struct` (no image library assumed): `mark.png`,
a 32×32 red square; `mark.svg`, a blue circle; `junk.png`, 4 KiB of text; `big.png`, a valid
300 KiB PNG (stored, uncompressed blocks). Steps and shots, each command typed in the project's
terminal with a three-second settle: `564-01-none` (the header as today); `cp fixtures/mark.png
favicon.png` (`564-02-png`: a red square before the name); `rm favicon.png; mkdir assets; cp
fixtures/mark.svg assets/; printf '<!doctype html><link rel="icon" href="assets/mark.svg">' >
index.html` (`564-03-svg`: a blue circle); `cp fixtures/junk.png favicon.png` (`564-04-junk`: the
circle still, the named file skipped); `rm -r favicon.png index.html assets; cp fixtures/big.png
favicon.png` (`564-05-too-big`: no icon). The run log greps Marley.log for the two skip lines,
naming `favicon.png` and the reason each time.

## Locked-In Decisions
- D1: The repository's files only. Orca's two fetches are left out: Marley makes no request on
  a project's behalf, and a favicon service would be told every project's homepage.
- D2: Orca's fifteen names and its three HTML files, first hit wins, with SVG and ICO added:
  gpui decodes both, and `favicon.ico` is the commonest icon file there is. A file that fails to
  read or decode is skipped, with one log line, and the search goes on to the next candidate.
- D3: Read and decoded off the main thread from bytes, resized to 32 px before it becomes a
  `RenderImage`: `img(path)` would cache the path for good and upload the file's full size.
- D4: Orca's 256 KiB cap. An icon file is small; a 20 MB `logo.png` is not an icon.
- D5: Refresh on the worktree's entry events for candidate paths only, once a second per
  project at most: a favicon added while Marley runs shows without a relaunch, and a build
  changing thousands of files costs nothing unless one of them is a candidate.
- D6: The pure model stays gpui-free. The icon is looked up in `rail.rs` by the row's group at
  render time; no snapshot field carries an image.
- D7: No setting and no per-project choice in this slice: the header without an icon is
  unchanged, and the look of real repositories' favicons at 16 px is for Chad's eye first.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a project's root holds one of the fifteen names as PNG, SVG, WebP or ICO, the rail shall draw it at 16 px before the project's name. | Shot `564-02-png` |
| REQ-002 | WHERE no named file exists and an `index.html` of the three declares `<link rel="icon" href="…">`, the rail shall draw the file the href names, resolved against the file's folder, `public/` or the root. | Shot `564-03-svg` |
| REQ-003 | WHILE a project has no icon file, its header shall look as it does today. | Shots `564-01-none`, `564-05-too-big` |
| REQ-004 | WHEN an icon file appears, changes or goes while Marley runs, the header shall follow within five seconds. | Shots `564-02-png` to `564-05-too-big` |
| REQ-005 | IF a candidate is over 256 KiB or does not decode, THEN the rail shall skip it, go on to the next candidate, and log one line naming the file. | Shots `564-04-junk` (the SVG still drawn), `564-05-too-big`; Marley.log in the run log |
| REQ-006 | The system shall make no network request for an icon. | Review of the diff: the module reads the disk and nothing else |
| REQ-007 | WHEN an icon is drawn, the image uploaded shall be at most 32 px on its long side, whatever the file's size. | Review of the diff: the resize before `RenderImage::new` |

## Phase Plan
- **P1 Plan:** this spec; the design and the test plan in the notes.
- **P2 Code:** `project_icons.rs` (the search, the read, the decode), the map and the subscription
  arm in `rail.rs`, the drawing in `render_project_row`; `image`, `resvg` and `usvg` in
  `marley_workbench/Cargo.toml` (all workspace dependencies already; cargo-shear needs each used);
  fmt and clippy clean; a review of the diff.
- **P3 Test:** write and run the scenario and read every shot; rerun `500-browser-from-the-rail.sh`,
  whose repository has no icon, so its header must look as before; `script/gates.sh --diff` green.
- **P4 Complete:** CHANGELOG; `docs/marley/workbench-shell.md` (the project header) and
  `docs/marley_architecture/marley_workbench.md`; the ledger capture; close the ticket, archive,
  commit.
