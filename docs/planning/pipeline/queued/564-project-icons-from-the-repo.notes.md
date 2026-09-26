# Project icons from the repository's own files — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-564-project-icons-from-the-repo.md
- **Pipeline spec:** 564-project-icons-from-the-repo.spec.md

## Phase 1 — Plan
- **Request:** the Orca second pass of 2026-09-25, the five smaller details, item 5: "Project
  icons from the repository's own files, for the rail's project headers: a favicon or logo at
  the usual paths, or the icon `index.html` declares. Orca also sends the `package.json`
  homepage to Google's favicon service and fetches GitHub avatars, which Marley should not do."
  Chad decided on 2026-09-26 that every remaining Orca and Warp finding gets built.
- **Classification / tier:** feature, S. Marley crates only (`marley_workbench`); no Zed touch;
  three workspace dependencies added to one Marley manifest.
- **Recall (§18.3):**
  - L-claude-468-sample-the-capture-before-trusting-a-theme-token-001: the header's colours
    come from the capture; the icon sits in the same row and changes no token.
  - L-claude-498-a-new-toolbar-button-moves-older-scenarios-clicks-001: the icon sits between
    the disclosure and the name, so the name moves right by 20 px when a project has an icon.
    #500's scenario clicks the header's `+`, at the row's right end, and its repository has no
    icon; Test reruns it.
  - The fork-port memory: `script/clippy` runs cargo-shear over the repository, so `image`,
    `resvg` and `usvg` in `marley_workbench/Cargo.toml` must each be used in the same change.
  - The `blocking_io_on_foreground` lint (an error in Marley crates, `marley_workbench.rs:8-14`):
    every read runs inside `cx.background_spawn`.
  - #504 (queued): a page's favicon read through CDP, kept as `Arc<gpui::Image>` with
    `favicon::format(bytes)`; this ticket's decoder is the shared piece if #504 wants a resized
    `RenderImage` instead.
  - The ledgers hold nothing on favicons or project icons (grepped `favicon`, `repo icon`,
    `project icon`, 2026-09-26). Brain: not consulted in this drafting pass; promotion runs
    `brain_ask`.
- **Discovery (opened and checked, 2026-09-26):**
  - `crates/gpui/src/elements/img.rs`: `ImageSource` (42-51: `Resource`, `Render`, `Image`,
    `Custom`); the `From<&str>` path that treats a string as a URI or an embedded asset (63-87)
    and the `From<&Path>` forms (89-105); `img` (201); `Img::extensions` (212-218);
    `image_cache` (229-235); `StyledImage` (148-177); `use_data` (535-554) through the asset
    cache, cached by `(TypeId, hash(source))` for good; `ImageAssetLoader::load` (619-746:
    `std::fs::read` at 634, `guess_format` at 669, GIF and animated WebP arms, `decode_static_image`
    at 735, SVG at 739-743); `remove_asset` (577-585).
  - `crates/gpui/src/asset_cache.rs:71-78` (`Resource::Uri`, `Path`, `Embedded`);
    `crates/gpui/src/app.rs:2748` (`remove_asset`), 2772-2784 (`asset_entry`).
  - `crates/gpui/src/platform.rs`: `ImageFormat` (2826-2849, ICO among them), `Image` (2904),
    `Image::from_bytes` (2953), `to_image_data` (3001-3043), `decode_static_image` (2913-2938).
  - `crates/gpui/src/assets.rs`: `RenderImage` (43-49), `new` (61), `size` (79).
  - `crates/gpui/src/svg_renderer.rs`: `render_single_frame` (222-229) at
    `SMOOTH_SVG_SCALE_FACTOR` 2 (81); `render_alpha_mask` (231-262); the 8192 px cap (270-293).
    `crates/gpui/src/elements/svg.rs:150-182`: the mask needs a text colour.
  - `crates/gpui/src/window.rs:4879-4915`: `paint_image` uploads the frame to the atlas;
    `crates/gpui_wgpu/src/wgpu_atlas.rs:125`: a frame past the texture limit paints nothing.
  - Root `Cargo.toml:669-685` (`image` 0.25.1 with `bmp`, `gif`, `ico`, `jpeg`, `png`, `webp`,
    `tiff`, no AVIF), 828-833 (`resvg` 0.46 with `text`, `system-fonts`, `raster-images`), 928
    (`usvg` 0.46); `Cargo.lock`: `image` 0.25.10, `resvg` 0.46.0, `usvg` 0.46.0, `tiny-skia`
    0.11.4. `crates/gpui/Cargo.toml:65, 81-82`.
  - `crates/marley_browser/src/frame.rs:15-28`: the BGRA swap and `RenderImage::new`;
    `marley_browser/Cargo.toml:18`.
  - `crates/project/src/image_store.rs:175-185` (`fs.load_bytes`), 933-951 (`create_gpui_image`).
    `crates/fs/src/fs.rs:135` (`load_bytes`).
  - `crates/marley_workbench/src/rail.rs`: `Rail` (72) and its subscription maps (117);
    `sync_subscriptions` (440); `follow_folders` as the project subscription (469-471);
    `render_project_row` (1067-1162: the frame 1089-1092, the `Disclosure` 1094-1106, `row_label`
    1107-1112, the trailing slot 1113-1131); `build_snapshot` (1805-1915: `project_groups` at
    1814, `group_names` at 1819, the `GroupEntry` push at 1880); `group_names` (2065-2079);
    `row_frame` (1953); `row_label` (1926).
  - `crates/marley_rail/src/marley_rail.rs`: gpui-free (1-7); `ProjectSnapshot` (30-42),
    `ProjectRow` (198-212).
  - `crates/project/src/project.rs:338` (`Event`), 374 (`WorktreeUpdatedEntries(WorktreeId,
    UpdatedEntriesSet)`), 3994-3998 (emitted from the worktree store's event); `ProjectGroupKey`
    (6590) with `path_list` (6623). `crates/util/src/path_list.rs:79` (`paths`).
  - Zed's own project icons: none (`project_panel.rs:6861-6903` gives the root the generic
    folder icon; `title_bar.rs:797-876` is text; `sidebar.rs:2282-2304` draws a host icon only).
  - Orca (MIT, read at `1c2cf120e3`): the files and lines in the spec's prior art; the 13
    `repo-icon*` files total about 1,190 lines with their tests; no doc page of its own
    (`docs/site/content/docs/settings.mdx:146` mentions the icon choices).
- **Decisions:** D1 to D7 in the spec.

### Design
- **Approach.**
  - *`project_icons.rs`* (new; pure functions first, one adapter): `CANDIDATES` (the fifteen
    names by four extensions), `DECLARING_FILES` (the three HTML paths), `MAX_BYTES` (256 KiB),
    `ICON_PX` (32). `declared_href(html: &str) -> Option<String>` (a small scan for `<link`
    tags, `rel` of `icon` or `shortcut icon` in any order and quoting, the `href` cut at `?` and
    `#`); `href_candidates(href, file_dir) -> Vec<PathBuf>` (Orca's rule); `decode(bytes) ->
    Result<Arc<RenderImage>, String>`: `image::guess_format` then `load_from_memory_with_format`
    for a raster, `imageops::resize` to 32 px on the long side (`FilterType::Lanczos3`), RGBA to
    BGRA, `RenderImage::new`; for bytes no format claims, `usvg::Tree::from_data` with default
    options and `resvg::render` into a 32 px `tiny_skia::Pixmap` scaled to fit, then the same
    BGRA frame. `find_icon(root: &Path) -> Option<(PathBuf, Arc<RenderImage>)>`: the candidates
    in order, each read with `std::fs::read` after a `metadata` size check, a failure logged
    (`log::info!`, the file and the reason) and skipped.
  - *The `Rail`:* `icons: HashMap<PathBuf, IconEntry { chosen: Option<PathBuf>, image:
    Option<Arc<RenderImage>>, task: Option<Task<()>> }>` keyed by the group's first path.
    `build_snapshot` starts a search for a key it has not seen (`cx.background_spawn(lazy(move
    |_| find_icon(&root)))`, then an update on the main thread and `cx.notify()`), and drops the
    entries of groups that are gone. `follow_folders` gains an arm for `WorktreeUpdatedEntries`:
    when any changed path, relative to the worktree's root, is in `CANDIDATES`, in
    `DECLARING_FILES` or equals the entry's `chosen`, a one-second timer per key replaces any
    pending one and then runs the search again.
  - *`render_project_row`:* after the `Disclosure`, `.when_some(icon, |this, image|
    this.child(img(ImageSource::Render(image)).size(px(16.)).rounded_sm().flex_none()))`; the
    icon is read from the map by `group.key.path_list().paths().first()` before the call, so the
    function's signature grows by one `Option<Arc<RenderImage>>`.
  - *Worktree rows (#510):* the same lookup by the project's key, when the rows exist.
- **File manifest.** Marley crate: `crates/marley_workbench/src/project_icons.rs` (new),
  `crates/marley_workbench/src/rail.rs`, `crates/marley_workbench/src/marley_workbench.rs` (the
  module), `crates/marley_workbench/Cargo.toml` (`image`, `resvg`, `usvg`). Test phase:
  `script/e2e/564-project-icons-from-the-repo.sh`.
- **Ledger rows.** None: no path outside the Marley-owned set changes.
- **Knowledge at Complete (expected).** An AD for D1 and D3 (files only; bytes decoded and resized
  before the atlas); a lesson if the worktree events for a candidate path behave unexpectedly.

### E2E plan
Setup writes the fixtures with Python (`zlib`, `struct`: a PNG signature, `IHDR`, one `IDAT` of
raw RGB rows, `IEND`; the big one with `zlib.compress(level=0)` so 300 KiB stays 300 KiB) and
the SVG as text, makes the scratch repository with a commit, and opens it. Each step is typed in
the project's terminal, followed by `settle 3`.

| REQ | Scenario part | Shot or log |
|---|---|---|
| REQ-003 | the rail after Marley opens the repository | `564-01-none`: the header as today |
| REQ-001, REQ-004 | `cp $E2E_WORK/fixtures/mark.png favicon.png` | `564-02-png`: a red 16 px square before the name |
| REQ-002, REQ-004 | `rm favicon.png; mkdir assets; cp $E2E_WORK/fixtures/mark.svg assets/; printf '<!doctype html><link rel="icon" href="assets/mark.svg">' > index.html` | `564-03-svg`: a blue circle |
| REQ-005 | `cp $E2E_WORK/fixtures/junk.png favicon.png` | `564-04-junk`: the circle still; Marley.log names `favicon.png` and the decode failure |
| REQ-003, REQ-004, REQ-005 | `rm -r favicon.png index.html assets; cp $E2E_WORK/fixtures/big.png favicon.png` | `564-05-too-big`: no icon; Marley.log names the size |
| REQ-006, REQ-007 | review | the diff |

Not reachable by a scenario: a network request that never happens (the review reads the module);
an ICO with several entries, which the scenario could add by writing one by hand if Code wants
the extra proof; a project on a remote host.

### Risks
- A `logo.png` that is a wordmark reads as a smear at 16 px; D7 leaves the judgement to Chad
  after a look at real repositories, and a per-project choice is the follow-up if it bites.
- An SVG that needs a font renders with the system's through `resvg`'s `system-fonts`; one that
  embeds a raster works through `raster-images`; one that references an external file gets
  nothing from `usvg`'s default options, which is what we want.
- A worktree event under a gitignored folder may not arrive (Zed scans ignored folders lazily);
  the search itself reads the disk, so the icon is right at the next search, and every named
  candidate sits at the root or one folder down, outside the usual ignored trees.
- The `image` crate's default allocation limit (512 MiB) bounds a hostile PNG's decode; the
  256 KiB cap bounds the read; a decode still runs off the main thread.
- Two windows open on the same project share one search through the key; a window closing
  drops nothing the other still shows, since the entry lives on the `Rail` per window.
- The name shifts right by 20 px on a project with an icon; a later scenario measuring the
  header's text by coordinates should add an icon to its fixture or none, and say which.
