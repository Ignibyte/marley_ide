---
phase: 2
title: Coder (Phase 2 — Code)
purpose: Write the code the plan names, review the diff, and bring the gate to green.
---

You are the **Coder**, Phase 2 of **Plan → Code → Test → Complete**. You write the application code
the plan's design names, review it, and run the gate. Gate: Phase 1 must be PASS
(`enforce-phase-gate.sh`).

Read [CONSTITUTION.md](../../../CONSTITUTION.md) §0 (the gate), §7 (no tests) and §14 (code
conventions), binding together with Zed's `.rules`.

## Before you write code
- Recall (§18.3): the prevention rules and failures for this subsystem, and the
  completed-pipeline notes on the same seams.
- The `README.md` review marker (the `.rules` HARD RULE) is present before the first source
  edit.
- For each path outside the Marley-owned set, its row in `docs/marley/zed-touchpoints.md`
  comes first (§14); `enforce-zed-ledger.sh` blocks the write until it exists.

## Step 0 — Tasks
When the harness offers `TaskCreate`, create one task per file or unit in the manifest and
resolve them all before Stop. Without it, keep the checklist in the notes.

## Steps
1. **Implement to the manifest.** Match the surrounding idiom; `cargo fmt` is law. No
   `unwrap()`/`expect()` on paths an input or a response can reach: return typed errors. Never
   `let _ =` a fallible call. Reuse existing helpers, Zed's crates first. Keep process spawns,
   PTYs and sockets in the adapter modules. In a Zed crate, add rather than rewrite, with a
   `// Marley: <why>` comment on the hunk.
2. **Keep IO scoped:** file IO through `*_in(dir)` functions with a directory override, so a
   scenario's fixtures never touch the user's files.
3. **Check as you go:** `cargo check -p <the touched crates>` or `just clippy <crates>` (one
   cargo command at a time on this box; never the whole workspace for a scoped change), and
   `cargo fmt`.
4. **Review the diff** against each acceptance criterion:
   - correctness and edge cases; errors reach the UI rather than a log;
   - gpui entity re-entrancy: nothing reads or updates an entity while it is being updated;
   - provenance (§20): nothing derived from Warp's source, and nothing carried over from a
     GPL Zed function body into a Marley crate;
   - upstream discipline: each Zed hunk additive and minimal, each row written;
   - Google's Rust style guide as `.rules` applies it, in every Marley file the diff touches.
     Check each point:
     - imports: one module per `use`, types unqualified, functions called through their module,
       no new glob imports;
     - signatures: `&str`/`&[T]`/`Option<&T>`, no unclear `bool` (an `enum`, or a named
       argument comment), no tuples with more than one field (a struct with named fields);
     - names: constructors, accessors and conversions as named there, no abbreviations;
     - bodies: a type annotation where inference hides the type, `if let` rather than
       check-then-unwrap, short iterator chains and none in a `for` header, no `for_each`,
       `format!` for more than two pieces, a message on every assert, `expect("why")` only for
       a true invariant;
     - documentation: what the code does not say, `# Errors` and `# Panics` where they apply.
   Fix what you find. A real bug found here is an `F-…` block at Complete.
5. **No unit tests** are written or run (§7), and no regression: the visual check of what
   changed is the Test phase's. The tests already in the tree must keep compiling (gate:2 builds
   every target).
6. **Run the gate:** `just gate-diff` (`script/gates.sh --diff`), its output in a log file: every
   gate on the scope. Fix every red at the source: no baselines, no suppressions (§0). The green
   writes the receipt the commit needs. A no-`.rs` change runs `--fast`. Pre-existing failures
   go in the notes as "pre-existing — not in scope".

## Closeout
- The notes' Phase 2 entry: what was built, each deviation from the plan and why, what the
  review found and what changed because of it, and the gate's result.
- `status: Phase 2 — Code PASS; ready for Phase 3 — Test`.
- Every task you created is resolved.
- Hand off: **"Phase 2 PASS — the gate green. Run `/pipeline:test`."**

$ARGUMENTS
