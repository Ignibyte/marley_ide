# Brand-scrub plan — remove Warp/Zed mentions from the Marley source

**Goal (chad):** nothing in the Marley *codebase* should mention "Warp" or "Zed" — so a future proprietary /
relicense pivot isn't shadowed by "derived-from" fear. The reference docs (`docs/warp_architecture/`,
`docs/zed_architecture/`) are the deliberate exception — they KEEP their mentions (that's their job). This scrubs
only `crates/**/*.rs`.

## The finding — it's small, and it's all comments

A 5-agent whole-word audit (`\bwarp\b` / `\bzed\b`, substring false-positives like "standardi**zed**" excluded):

| Scope | Files | Mentions | Notes |
|---|---|---|---|
| [app.rs](app-rs.md) | 1 | 36 | all comments; design-intent tags ("Warp-style", "Warp parity") |
| [UI/style cluster](ui-cluster.md) | 4 | 12 | all comments; 1 clean-room attestation flagged |
| [workspace/misc](workspace-misc.md) | 6 | 8 | all comments |
| [strings/config](strings-config.md) | — | **0** | **zero user-facing exposure** (see below) |
| [full-tree sweep + lint](sweep-and-policy.md) | reconcile | — | confirms 11 files, no extras + the keep-clean lint |
| **TOTAL** | **11** | **~56** | **all "Warp", zero "Zed", 100% code comments** |

**What is NOT affected (the reassuring part):**
- **Zero user-facing brand.** Themes are already `"Marley Light"` / `"Marley Dark"` (no "Warp Dark"); the default
  is `"Marley Dark"`. All palette labels, tab titles, and status messages are brand-neutral (the "Warp Workflows"
  analog uses the generic word "Workflow").
- **Zero identifiers / strings / test-data / config.** No fn/type/var is named after a brand; no `Cargo.toml`,
  asset, or `.stderr` fixture carries one. `marley_util`, `marley_core`, every other crate, and all test files
  are clean.
- **Whole-word "Zed" appears nowhere** in the source (every match was a substring).

→ The scrub is **pure comment-prose rewording**, so it **cannot** change behavior, mutation, or coverage — the
green gate stays green.

## The reword rule
Describe the behavior/value in Marley's own terms; keep the ticket number + the technical value; drop the brand.
- "the Warp-style bar at the very top" → "the top command bar"
- "#195 (Warp parity): match Warp's ~13pt terminal density" → "#195: the ~13pt terminal density"
- "the Warp arrangement" → "the terminal-first arrangement"
- "Warp-like gray" → "the panel gray"

## Items to eyeball on apply (per the agents)
- **`color.rs:9` — a clean-room ATTESTATION** ("not lifted from Warp or any AGPL source"). This mentions the
  brand *defensively* (a claim of originality). Reword to "not lifted from any AGPL-licensed source" — preserve
  the originality claim, don't just delete it. (Owner may prefer to keep it verbatim as a provenance note.)
- **The 4 `//!` module-doc headers** (grid_layout / workflows / nav / file_tree_view) — highest visibility
  (rustdoc-rendered). Reword carefully so gate:14 (rustdoc `-D warnings`) stays green.
- **`right_dock.rs:74`** — a verbatim chad quote ("icons like Warp"); **`workflows.rs:1`** — the "Warp Workflows
  analog" feature-name reference. Reword to the generic capability, keep the intent.

## Keep-clean lint (wire in AFTER the scrub)
Prevent new brand mentions from reappearing. Add to `scripts/gates.sh` (fold into `gate:14 docs_g`, or a
standalone descriptively-labelled static gate — NOT a new number; gate:16 was removed):
```sh
# no Warp/Zed brand names in the Marley source (the reference docs under docs/*_architecture/ are exempt)
brand=$(grep -rniwE 'warp|zed' crates --include='*.rs' 2>/dev/null || true)
[ -n "$brand" ] && { echo "brand mention in source:"; echo "$brand"; FAIL=1; }
```
Notes: `-w` (whole-word) is the substring guard — the tree has 80+ `zed`-substring words
(`StandardizedPath`×24, `initialized`, `normalized`, `serialized`…); do NOT use `\b` (BSD-`grep`-unsafe). Scoping
to `crates/` auto-excludes the exempt `docs/*_architecture/`. It rides `/commit` + the enforce-commit-gate hook
for free. ⚠️ Wire it in only *after* the 11 files are scrubbed, or it goes RED on the current 56.

## Status
**EXECUTED 2026-07-12 — TICKET-262 (sprint #29, M16).** All 56 rewords applied (3 documented grammar deviations
where a catalog line read broken in context — see the pipeline notes), the `color.rs` attestation reworded per
the owner's ticket text, and the keep-clean lint wired into gate:14 (`docs_g` third check) in the same commit,
with the `quality-bar.spec.md` gate-14 row updated to match. Detector at zero; negative smoke proven
(inject → gate:14 RED → revert → GREEN [diff] 15/15). This directory is now the historical audit record.
