---
pipeline_id: b2e70474-b684-489b-8e22-118d214982e0
ticket: docs/planning/tickets/open/TICKET-447-rustal-quality-gates.md
status: Phase 4 — Complete PASS
title: Rustal's Rust quality gates on the Marley crates
type: chore
slice: quality gates (outside the prongs; applies to every Marley crate)
references: [CONSTITUTION.md, script/gates.sh, /srv/stacks/rustal/bin/gate.sh, /srv/stacks/rustal/Cargo.toml]
---

## Title
Bring rustal's Rust quality gates to the seven `crates/marley_*` crates: its lint table, with
every hit fixed; the manifest, spelling, empty-suite, TODO, rustdoc-warning and semgrep checks;
and the sturdier receipt and explicit modes of its gate runner. Mutation stays out of the gate
(`script/mutation.sh`, run at the end of a sprint).

## Scope
### In
- **The lint table.** Each Marley manifest replaces `[lints] workspace = true` with rustal's
  table: Zed's own denies (`dbg_macro`, `todo`, `declare_interior_mutable_const`,
  `redundant_clone`, `disallowed_methods`), rustal's rust lints (`unsafe_code`,
  `missing_docs` and `missing_debug_implementations` deny; `unreachable_pub`,
  `unused_qualifications` and `unused_results` warn), and rustal's clippy table (`pedantic`,
  `nursery` and `cargo` warn; `missing_errors_doc`, `missing_panics_doc`, `missing_safety_doc`,
  `print_stdout`, `print_stderr`, `unimplemented`, `unwrap_used` and `expect_used` deny;
  `clone_on_ref_ptr` warn; `multiple_crate_versions` and `module_name_repetitions` allow). Zed's
  blanket `style = allow` is dropped for the Marley crates. Every hit is fixed; a crate-specific
  allow lives in its manifest with a comment saying why. Added in the Code phase:
  `let_underscore_must_use` deny (Zed's `.rules` bans `let _ =` on a fallible call); allows for
  `cargo_common_metadata`, `negative_feature_names` and `redundant_feature_names`, which read
  every manifest in the workspace and so report Zed's packages and Zed's `test-support`
  convention; and, in `marley_workbench` only, `unused_results` (gpui's `&mut App` chaining
  returns and `.log_err()`'s `Option`).
- **TICKET-444, folded in.** The new lint makes the eight `let _ =` discards #444 lists
  compile errors, so they are handled here: each propagates its error, logs it, or matches
  its one expected failure (ESRCH for the reap signal), and #444 closes with this ticket.
- **Test allowances.** `allow-unwrap-in-tests` and `allow-expect-in-tests` in the root
  `clippy.toml` (a Zed file; its ledger row comes first). Zed's crates enable neither lint, so
  the keys change nothing for them.
- **Gates.**
  - gate:13 accepts a `// SAFETY:` comment on the line above the `unsafe` and matches a bare
    `transmute(` as well as `mem::transmute`.
  - gate:14 fails when rustdoc prints a `warning:` line, and scans the Marley crates' Rust
    source for TODO, FIXME and XXX markers as it already scans their docs.
  - gate:17 (new, manifests): `cargo sort --check` and `taplo fmt --check` over the Marley
    manifests.
  - gate:18 (new, spelling): `typos` over the repository with Zed's `.config/typos.toml`, as
    Zed's own CI runs it.
  - gate:19 (new, empty suites): every Marley test suite except a binary's lists at least one
    test (`cargo nextest list`), rustal's rule.
  - gate:20 (new, semgrep 1.156.0): `.semgrep.yml` with rustal's two rules that clippy and
    gitleaks do not cover, `process-exit-in-library` and `command-injection-risk`, over
    `crates/marley_*`.
- **Runner.** Explicit modes (`--full`, `--diff`, `--fast`): no argument or an unknown one is a
  usage error. After a static red the heavy gates report BLOCKED instead of running. A FULL or
  DIFF run removes any earlier receipt when it starts and writes a new one only when the
  fingerprint at the end matches the one at the start. The fingerprint also covers
  `rustfmt.toml`, `.config/typos.toml`, `.semgrep.yml` and every file under `crates/marley_*`.
- **Doctests.** `doctest = false` leaves the seven Marley manifests, so gate:3's doctest half
  runs.
- **Docs.** CONSTITUTION §0's gate table and tools list, §14's lint rule, and the phase commands
  that name the gate modes.

### Out (explicitly deferred)
- Mutation testing: `script/mutation.sh`, at the end of a sprint (CONSTITUTION §0).
- Zed's own dylint lints (`tooling/lints`: `entity_update_in_render`, `notify_in_render`,
  `blocking_io_on_foreground`): they need `cargo-dylint` and the nightly toolchain the lints
  package pins; a follow-up ticket.
- `cargo +nightly udeps` (cargo-shear already gates unused dependencies), `cargo-semver-checks`
  (the crates are unpublished), `cargo-hack` (no Marley crate declares features).
- Rustal's DIFF global-input rule and the 90-day aging of advisory ignores.
- Lints on Zed's own crates: they keep Zed's bar (§0).

## Reference (§20)
N/A — Marley-specific tooling with no Warp or Zed behavior to match. The reference is Chad's own
rustal gate: `/srv/stacks/rustal/bin/gate.sh` (gates 2, 4, 10-15, 20, 22), its lint table in
`/srv/stacks/rustal/Cargo.toml:17-64`, `/srv/stacks/rustal/clippy.toml:5-6` and
`/srv/stacks/rustal/.semgrep.yml:106-178`, mapped against Marley's gate in this session.

### Prior art
- **Behavior maps:** none apply; this is gate tooling.
- **Published material:** the clippy lint list and `clippy.toml` options
  (`allow-unwrap-in-tests`, `allow-expect-in-tests`), the cargo `[lints]` table (a crate's table
  replaces the workspace's; the two cannot be combined), cargo-sort, taplo, typos and semgrep's
  rule syntax.
- **Code we already ship:** Zed's `script/clippy` already runs `cargo shear` and `typos` over the
  repository, with `.config/typos.toml`; Zed's CI runs both. Zed's root `clippy.toml` holds the
  `disallowed-methods` list the Marley crates keep through `disallowed_methods = "deny"`.
  `tooling/lints` holds Zed's dylint lints (out of scope, above). The tools are installed:
  cargo-sort, taplo 0.10.0, typos 1.50.1, semgrep 1.156.0 (rustal's pin), cargo-nextest 0.9.143.
  Rustal's `bin/gate.sh` owns every check adopted here; its commands are carried over, scoped
  to the Marley crates.

## UI proof
N/A — no UI delta: the lint fixes preserve behavior and the gate changes are tooling. The
existing driven tests (26 in `marley_workbench`) and unit tests prove the crates unchanged.

## Locked-In Decisions
- D1 — The lint table is per crate: cargo cannot combine `workspace = true` with overrides, and
  the table must not reach Zed's crates. The seven tables are identical apart from a
  crate-specific allow, which carries a comment.
- D2 — Test code may `unwrap()` and `expect()` (clippy's test allowances); library code may not.
  A genuine invariant in library code keeps its `expect` with an inline
  `#[allow(clippy::expect_used)] // <why>` (gate:12).
- D3 — `unreachable_pub` against `redundant_pub_crate` is resolved as rustal resolves it:
  `pub(super)` for items only the parent reaches, a `pub` module under `#[cfg(test)]` when other
  modules reach it, never an `#[allow(unreachable_pub)]`.
- D4 — New gates take new numbers (17 to 20); retired numbers (5, 15) are not reused.
- D5 — Semgrep carries only the two rules nothing else covers; `dbg!`, TODO markers and secrets
  are already gated by clippy, gate:14 and gitleaks.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | Each of the seven Marley crates shall carry rustal's lint table, and gate:2 (`cargo clippy -p <scope> --all-targets --all-features -- -D warnings`) shall pass over them | gate:2 |
| REQ-002 | An inline `#[allow]`/`#[expect]` in a Marley crate shall carry a `//` reason, and a manifest-level allow shall carry a comment | gate:12 + review |
| REQ-003 | WHEN a Marley crate has `unsafe` with its `// SAFETY:` comment on the line above, gate:13 shall pass; WHEN it calls a bare `transmute(`, gate:13 shall fail | negative smoke |
| REQ-004 | WHEN rustdoc prints a `warning:` line for a Marley crate, or a Marley crate's Rust source holds a TODO, FIXME or XXX marker, gate:14 shall fail | negative smokes |
| REQ-005 | WHEN a Marley manifest is not sorted per `cargo sort` or not formatted per `taplo fmt`, gate:17 shall fail | negative smoke |
| REQ-006 | WHEN `typos` finds a misspelling under `.config/typos.toml`, gate:18 shall fail | negative smoke |
| REQ-007 | WHEN a Marley crate's test suite other than a binary's lists no tests, gate:19 shall fail | negative smoke |
| REQ-008 | WHEN a Marley crate calls `std::process::exit` outside a binary, or builds a `std::process::Command` from a program name that is not a string literal, gate:20 shall fail | negative smoke |
| REQ-009 | WHEN `script/gates.sh` runs with no mode or an unknown one, it shall exit 2 with a usage line and run no gate | negative smoke |
| REQ-010 | WHEN a static gate is red in a FULL or DIFF run, the heavy gates shall report BLOCKED and not run | negative smoke |
| REQ-011 | WHEN a FULL or DIFF run starts, an earlier receipt shall be removed; WHEN the fingerprint at the end differs from the start, no receipt shall be written | negative smoke |
| REQ-012 | The receipt fingerprint shall change when `rustfmt.toml`, `.config/typos.toml`, `.semgrep.yml` or any file under `crates/marley_*` changes | smoke |
| REQ-013 | gate:3's `cargo test --doc` shall run each Marley crate's doctests, with no crate setting `doctest = false` | gate:3 + manifest check |
| REQ-014 | The existing tests of the seven crates shall stay green and gate:4 shall stay at 100% of lines | gate:3, gate:4 |
| REQ-015 | `script/gates.sh --diff` shall be green | the gate |
| REQ-016 | No Marley crate shall bind a `#[must_use]` value to `let _` (`let_underscore_must_use` deny, gate:2), and each of #444's eight discards shall propagate its error, log it, or match its one expected failure | gate:2 + review + the new pump test |

## Phase Plan
- **P1 Plan** — this spec, and the design and test plan in the notes.
- **P2 Code** — the tables and `clippy.toml`, then the fixes crate by crate (`cargo clippy --fix`
  for the machine-applicable ones first, then by lint), then the gate and fingerprint changes;
  fmt and clippy clean; a review of the diff for behavior changes hidden in lint fixes.
- **P3 Test** — the crates' tests and coverage, a negative smoke per new check, and
  `script/gates.sh --diff` green.
- **P4 Complete** — CHANGELOG, CONSTITUTION §0 and §14, the ledger row, lessons, close, archive,
  commit.
