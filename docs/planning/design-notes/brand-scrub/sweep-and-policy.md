# Brand-Scrub: Full-Tree Sweep + Keep-Clean Policy

Auditor sweep of the Marley **source tree** for Warp/Zed brand mentions, plus a
proposed lightweight gate to keep it clean going forward.

**Scope:** `crates/**/*.rs` (source). The reference transcriptions
`docs/warp_architecture/` and `docs/zed_architecture/` are EXEMPT and out of
scope — they describe the third-party source deliberately.

**Method:** whole-word, case-insensitive extended-regex grep —
`grep -rniwE 'warp|zed' crates --include='*.rs'` — the `-w` flag is the
substring guard (see §c).

---

## TL;DR

| Metric | Value |
| --- | --- |
| Files with a brand mention | **11** (exactly the known 11 — no extras) |
| Total whole-word mentions | **56** |
| `warp` vs `zed` | **56 Warp · 0 Zed** (zero whole-word "zed" anywhere in source) |
| Location of every hit | **doc/line comments only** — 0 in code, strings, or identifiers |
| Crate-level `CLAUDE.md` / README | **none exist**; `NOTES.md` + all `Cargo.toml` clean |
| `marley_util`, `marley_core`, all test files | **clean** (zero brand) |

**Lint recommendation:** add a `brand_scrub_g()` static gate to `scripts/gates.sh`
using the identical `grep -rniwE 'warp|zed' crates --include='*.rs'` (fail on any
hit). Scoping to `crates/` auto-excludes the reference docs; the `-w` flag
excludes the substring trap (`standardiZED`, `normaliZED`, `siZED`, …). Wire it
in **after** the 11 files are scrubbed, or it starts RED (56 current hits).

---

## (a) Full-tree reconciliation

Command run:

```
grep -rniwE 'warp|zed' crates --include='*.rs'
```

Result: **56 mentions across 11 files** — every one is the word **Warp**; there
is **zero** whole-word **Zed** in the source. The 11 files reconcile **exactly**
against the known list handed to the per-file agents — no misses, no extras.

| # | File | Hits | Comment kind | Reconciles? |
| --- | --- | ---: | --- | --- |
| 1 | `crates/marley_app/src/app.rs` | 36 | `//` inline ("#NNN (Warp parity)" ticket notes) | ✅ known |
| 2 | `crates/marley_app/src/typography.rs` | 7 | `//` inline + block ("Warp parity/-calibrated/-density") | ✅ known |
| 3 | `crates/ui_components/src/lib.rs` | 3 | `//` inline + a test-assert trailing comment ("Warp-matched/-like") | ✅ known |
| 4 | `crates/marley_app/src/workspace.rs` | 2 | `///` item doc ("Warp typed-pane grid", "Warp-layout enabler") | ✅ known |
| 5 | `crates/marley_app/src/grid_layout.rs` | 2 | **`//!` module-doc header** ("Warp layout", "Warp arrangement") | ✅ known |
| 6 | `crates/ui_components/src/render/keyboard_shortcut.rs` | 1 | `///` item doc ("#222 Warp parity") | ✅ known |
| 7 | `crates/marley_app/src/workflows.rs` | 1 | **`//!` module-doc header** ("Warp Workflows analog") | ✅ known |
| 8 | `crates/marley_app/src/right_dock.rs` | 1 | `///` item doc (chad's "icons like Warp") | ✅ known |
| 9 | `crates/marley_app/src/nav.rs` | 1 | **`//!` module-doc header** ("Warp-style folding") | ✅ known |
| 10 | `crates/marley_app/src/file_tree_view.rs` | 1 | **`//!` module-doc header** ("Warp explorer look") | ✅ known |
| 11 | `crates/marley_app/src/color.rs` | 1 | `//` clean-room attestation ("not lifted from Warp or any AGPL source") | ✅ known |

**Confirmed clean (no brand mention):**

- `crates/marley_util/**` and `crates/marley_core/**` — zero hits.
- All test files / `#[cfg(test)]` lines / `tests/` dirs — zero hits.
- Every other crate (`editor`, `marley_agent`, `marley_command`,
  `marley_project`, `marley_remote`, `marley_search_core`, `marley_settings`,
  `marley_text_offsets`, `marley_visual_harness`, `terminal_blocks`) — zero hits.

### Two flavors worth calling out for the scrubbers

1. **`//!` module-doc headers (4 files:** `grid_layout.rs`, `workflows.rs`,
   `nav.rs`, `file_tree_view.rs`**).** These are the **highest-visibility**
   mentions — they render as the module's summary line in `cargo doc` output
   (gate:14). Prioritize these.
2. **`color.rs:9` is a clean-room ATTESTATION**, not a styling note:
   `"clean-room — not lifted from Warp or any AGPL source"`. Do not just delete
   the brand — the sentence's job is to assert originality. Reword to keep the
   claim, e.g. `"clean-room — not lifted from any external / AGPL source"`.
   (Flag only; this auditor does not edit code.)

---

## (b) Doc-comment / CLAUDE.md hits

- **No crate-level `CLAUDE.md` or `README` exists** anywhere under `crates/`.
  The only crate-level doc file is `crates/marley_app/assets/icons/NOTES.md` —
  **clean** (no brand). No `Cargo.toml` description/keywords carry a brand.
  No non-`.rs` crate file (`*.json`, `*.txt`, `*.toml`, `*.md`) carries a brand.
- **Every one of the 56 hits is inside a doc/line comment** — verified none fall
  in code, string literals, or identifiers. Breakdown by comment type:
  - **`//!` module-doc headers:** 5 lines across 4 files (grid_layout ×2,
    workflows, nav, file_tree_view) — the rustdoc-visible set.
  - **`///` item doc-comments:** workspace ×2, keyboard_shortcut, right_dock.
  - **`//` line comments:** the remaining ~45 (app.rs ×36, typography ×7,
    ui_components/lib ×3 incl. one trailing a test `assert_eq!`, color ×1).

So the scrub is purely a **comment-prose** rewrite (drop "Warp"/reword to a
neutral descriptor like "the terminal-block layout" / "the modern-terminal
density") — no code, no public API, no string, no test fixture changes.

---

## (c) Keep-clean lint proposal

### The check

A presence test — the gate fails if **any** whole-word `warp`/`zed` appears in
`crates/**/*.rs`:

```sh
# ── brand-scrub: no Warp/Zed brand mentions in Marley source ──────────────────
# The reference transcriptions docs/warp_architecture/ + docs/zed_architecture/
# are EXEMPT — scoping the scan to crates/ excludes docs/ by path, so no
# per-directory exclude is needed. Whole-word (-w) is the substring guard:
# it excludes standardiZED / normaliZED / seriaLIZED / siZED / recogniZED /
# StandardiZEDPath etc. (real substring hits in this tree: 80+ lines) while
# still catching the prose brand "Warp" / "Zed".
brand_scrub_g() {
  local hits
  hits=$(grep -rniwE 'warp|zed' crates --include='*.rs' 2>/dev/null || true)
  [ -z "$hits" ] || {
    echo "Warp/Zed brand mention(s) in Marley source (docs/*_architecture/ is the only allowed home):"
    echo "$hits"
    return 1
  }
  return 0
}
```

### Why `-w` and not `\b` (the substring trap)

Whole-word matching is mandatory: without it, `zed` is a substring of a **flood**
of ordinary identifiers/words in this tree —

| Substring word | occurrences | | word | occurrences |
| --- | ---: | --- | --- | ---: |
| `StandardizedPath` | 24 | | `serialized` | 7 |
| `initialized` | 16 | | `sized`/`oversized`/`Sized` | ~9 |
| `normalized` | 14 | | `recognized`/`sanitized`/… | ~7 |
| `standardized` | 9 | | | |

A naive `grep 'zed'` would drown in ~80+ false positives. `warp` has **no**
common-word substring collisions (verified: zero non-whole-word `warp` hits), so
the trap is a `zed`-only hazard — but the matcher must guard both.

**Use `grep -w`, not the `\b` word-boundary escape.** `scripts/gates.sh` is
explicitly *"Bash 3.2 + BSD-safe (… POSIX ERE, no `-P`/`\K`/`\s`)"*. `\b` is a
GNU extension; although the current macOS `grep` happens to honor it, `-w` is the
POSIX-portable flag and is exactly what the sweep in §a used. Consistency +
portability → `-w`.

**Known limitation of `-w` (acceptable):** `-w` treats `_` as a word char, so an
underscore-joined identifier like `warp_layout` / `zed_theme` would slip through.
That is a theoretical gap — all 56 real mentions are natural-language prose in
comments ("Warp parity"), which `-w` catches. If you want to close the gap, swap
the pattern for a POSIX-ERE boundary regex that also catches identifiers while
still excluding the substring trap:

```sh
# stricter variant — also catches warp_layout / zed_theme, still excludes standardized/sized:
grep -rniE '(^|[^A-Za-z])(warp|zed)([^A-Za-z]|$)' crates --include='*.rs'
```

Recommendation: ship the simple `-w` form (matches the sweep, lowest risk); adopt
the boundary variant only if an underscore-identifier mention ever appears.

### Where it plugs into the gate

`scripts/gates.sh` already has the exact idiom — `no_suppr_g`, `source_bans_g`,
`docs_g` all do *"grep → collect → non-empty ⇒ echo + `return 1`"*. Two wiring
options, both using the identical grep:

- **Option A (recommended) — fold into `docs_g` (gate:14).** Brand hygiene is a
  doc-quality concern and `docs_g` already excludes the reference docs for its
  TODO scan (`grep -vE 'docs/(planning|warp_architecture)/'`), so the precedent
  is right there. Add the brand grep as a third check after the rustdoc build and
  the doc-todos check; a hit makes gate:14 red.

- **Option B — a standalone static gate.** Add `brand_scrub_g()` (above) and one
  registration line in the STATIC block, next to gate:12/13/14:

  ```sh
  run_gate "brand-scrub (no Warp/Zed in crates)" brand_scrub_g
  ```

  Use a **descriptive label, not a new number** — the spec fixes the canonical
  gates at 1–15 and gate:16 was deliberately removed (TICKET-006). `run_gate`
  takes a free-form label, so `"brand-scrub …"` reads honestly as a project-local
  hygiene gate.

Either way it lands **inside the STATIC group** (cheap grep, always run — no need
to gate it behind FULL/DIFF).

### What you get for free

- **Enforced by `/commit` and the `enforce-commit-gate.sh` hook automatically** —
  the gate runs inside `gates.sh`, whose green is what the commit receipt binds.
  No new git hook to write or install.
- **Self-validating** — gate:11 shellcheck already lints `scripts/*.sh`, so the
  new function is held to the same shell-quality bar.
- **No reference-doc exclude to maintain** — scoping to `crates/` means
  `docs/warp_architecture/` and `docs/zed_architecture/` can never trip it.

### Sequencing note

The tree currently has **56 hits**, so wiring this gate in **before** the 11
files are scrubbed makes it start RED (and would block the very commit that adds
it). Land the gate in the **same change that removes the last mention**, or
immediately after the per-file scrub PRs merge. Its job from then on is to keep
the count at zero.
