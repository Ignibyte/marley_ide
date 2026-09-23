# Marley Constitution

The binding rules for building **Marley**, Ignibyte's agentic dev cockpit, built since
2026-09-18 as a fork of Zed (`Ignibyte/marley_ide`; upstream `zed-industries/zed`). The
pipeline commands (`.claude/commands/`) and the `.claude/hooks/` enforcement layer treat
this document as law. When a hook blocks you, it cites a section here. Sections are stable
anchors; hooks grep for `§N`, so the numbers never change even where a section retired.

What Marley is in this repo: Zed's editor, language intelligence, project model, workspace,
settings and agent panel, plus the three prongs of `docs/marley/three-prong-plan.md`: the
block terminal, the control plane (rustal-brain, the rustal-harness runtime, Rusty), and the
browser service. Marley-owned code lives in `crates/marley_*`. Changes inside Zed's own
crates are additive and small so upstream merges stay cheap. The gpui-era app that preceded
the fork is the design record under `docs/marley_architecture/`, `docs/specs/` and
`docs/marley/history/`; its knowledge ledgers and pipeline archive continue under
`docs/planning/` (§19).

---

## §0 — Quality Gates (binding)

The canonical gate is **`script/gates.sh`**, the single source of truth for "is this change
shippable?". It must pass green in the Test phase, in one of two commit-valid modes:
**`--full`** (the heavy gates over every Marley-owned crate; the periodic audit) or
**`--diff`** (the heavy gates only on what the change touched; the per-change loop). Both
write the receipt the commit hook requires (§15). `--fast` runs the static gates only, prints
`GATE GREEN [fast]`, writes no receipt, and can never satisfy a commit of Rust source. The
mode is always named: none, or an unknown one, is a usage error (exit 2) and runs no gate.

```
STATIC (always; --fast runs exactly these)
gate:1  rustfmt        cargo fmt --all --check                             (whole workspace)
gate:2  clippy         cargo clippy --all-targets --all-features -p <scope> -- -D warnings
gate:3  tests          cargo nextest run -p <scope>  +  cargo test --doc -p <marley crates>
gate:7  audit          cargo audit             (fork-point advisories listed, new ones fail)
gate:8  supply chain   cargo deny check licenses bans sources
gate:9  unused deps    cargo shear --locked --deny-warnings          (Zed's own tool)
gate:10 secrets        gitleaks: commits since the upstream fork point + Marley-owned dirs
gate:11 shell lint     shellcheck (.claude/hooks + script/gates.sh + script/mutation.sh)
gate:12 no-suppress    grep meta-gate (allow/expect must justify; blanket banned)
gate:13 source-bans    grep meta-gate (transmute, bare or through mem::; unsafe without a
                       // SAFETY: on its line or the line above)
gate:14 docs           rustdoc -D warnings on the Marley crates, and no `warning:` line printed;
                       no actionable TODO in the Marley crates' Rust source or Marley docs
gate:16 zed ledger     every changed path outside the Marley-owned set has its row in docs/marley/zed-touchpoints.md
gate:17 manifests      cargo sort --check + taplo fmt --check on the Marley manifests
gate:18 spelling       typos --config .config/typos.toml              (the repository, as Zed's CI)
gate:19 empty suites   cargo nextest list -p <marley crates>: every suite but a binary's has a test
gate:20 semgrep        semgrep 1.156.0 --config .semgrep.yml --error --strict on the Marley crates
gate:21 dylint         cargo dylint --all -- --all-targets -p <marley crates>: Zed's tooling/lints,
                       denied in each Marley crate root under the driver's dylint_lib cfg

HEAVY (--full + --diff; --fast skips; BLOCKED, not run, after a static red)
gate:4  coverage       cargo llvm-cov nextest -p <marley crates> --fail-under-lines 100
gate:6  miri           cargo +nightly miri  (conditional on unsafe in a Marley crate)
```

Retired numbers are not reused: gate:5 (mutation, below) and gate:15 (the gpui-era macOS
harness).

**Mutation testing is not a gate.** It was gate:5 until 2026-09-22, when Chad took it out of
the per-change loop because it was too slow to run on every change. `script/mutation.sh`
runs it over the Marley crates once, at the end of a sprint or before a release, and what it
finds is fixed at the source like any other red.

**The scope rule.** The Marley-owned surface is `crates/marley_*`. The static gates run over
those crates plus every crate the change touched (derived from `git status`, so a new
untracked crate counts). The heavy gates run over the Marley crates in FULL; in `--diff`
mode gate:4 covers the touched Marley crates. Upstream Zed code
is not held to the Marley floors: it is held to Zed's own bar (fmt, `./script/clippy`, its
own tests), and a change inside a Zed crate must leave that crate's tests green and add
driven tests for the behavior it adds (§7).

**No baselines. No suppressions. Source-fix only.** Any inline `#[allow(…)]` /
`#[expect(…)]` in Marley code must carry a real `//` justification (gate:12); blanket group
allows are banned outright. Every gate's verdict is the tool's **exit code**, never a grep of
its output. The gate runs ALL steps and reports each; a single red fails the gate.

**Process spawning is permitted.** Marley drives PTYs and child processes (the terminal, the
harness client, the agent hosts). What gate:13 bans is `mem::transmute` and `unsafe` without
a `// SAFETY:` justification. Keep spawns in adapter modules and validate inputs; no shelling
out unsanitized user input.

**Floors ratchet up, never down.** `RUST_COV_MIN` is baked into the gate at **100% lines**
over the Marley crates. Env may raise it; a value below the baked-in minimum is clamped back
up. Lowering a baked-in minimum is a charter amendment.

**ACCEPTED-UNTESTABLE is explicit, never silent.** Coverage runs with an explicit, documented
exclude list in the gate (today: the raw PTY shim `marley_terminal/src/pty_os.rs` and the
`std::net` transport `marley_mcp/src/transport.rs`). A new uncoverable path (FFI, GPU, a headed window, a bound port, a live service)
is added to that list with a reason, never hidden in a regex.

**Honest known-scope (the ratchet roadmap).** Recorded gaps, each a ratchet item:

- clippy runs rustal's lint table on the Marley crates (pedantic, nursery and cargo, with
  its deny list; §14) and Zed's workspace lints (`[workspace.lints]` in `Cargo.toml`) on
  Zed's crates. gate:21 runs Zed's dylint lints (`tooling/lints`) on the Marley crates, where
  each crate root makes them errors; on Zed's crates they stay at the library's warn level.
  A lint the library adds warns in the Marley crates until it joins the roots' lists.
- `gate:3` runs `--no-tests=warn` over the scope, so a Zed crate with no tests is a visible
  warning; gate:19 fails a Marley test suite with none. The binding "every behavior is
  tested" enforcement is gate:4 on the Marley crates plus the driven tests §7 requires for
  UI paths.
- `gate:7`: advisories already present at the upstream fork point belong to upstream's
  dependency tree. They are listed per id in `.cargo/audit.toml` with the fork commit they
  were inherited at, and the list is regenerated at every upstream merge. A NEW advisory
  fails the gate.
- `gate:8`: the license allowlist is Zed's accepted list (`script/licenses/zed-licenses.toml`
  is the reference); git sources are limited to the organizations upstream pins and any
  fork Marley carries (`deny.toml`). Zed's own crates are GPL-3.0-or-later or Apache-2.0;
  the Marley crates are `MIT OR Apache-2.0` (§20).
- `gate:10` scans the commits since the upstream fork point plus the working tree of the
  Marley-owned directories; upstream's history is upstream's.
- `gate:14` scans the Marley crates' Rust source and Marley-authored docs only
  (`CONSTITUTION.md`, `docs/marley/`, `docs/marley_architecture/`, `docs/specs/`,
  `docs/decisions/`, `docs/zed_architecture/`, `docs/tickets/`, `.claude/`);
  `docs/planning/` is working scratch and
  `docs/warp_architecture/` transcribes Warp's own markers. The gpui-era brand scrub is
  retired: this repo is Zed.
- `gate:16` compares the work tree, the index and the untracked files with the upstream
  fork point (`upstream_base` in `.claude/hooks/lib-hook-helpers.sh`: the merge-base with a
  fetched `upstream/main`, else `MARLEY_UPSTREAM_BASE` when set, where a value that names no
  commit fails closed, else the recorded fork commit). It fails on a changed path outside the
  Marley-owned set that has no row in `docs/marley/zed-touchpoints.md`, on a row whose path
  no longer differs, on duplicate rows and rows for owned paths, and on any upstream file the
  owned set would claim. The owned set is `marley_owned_path` in the same file, shared with
  the write hook, and `enforce-commit-gate.sh` runs the same check at every `git commit`, Rust
  or not (§14).
- `gate:15` (the macOS accessibility and screenshot harness) is retired; UI proof is §7.
- not yet ported: architecture-layering / taint analysis.

A `--full` or `--diff` run removes the earlier receipt when it starts, and on a green writes
a worktree-bound receipt (`.git/ignibyte-gate-receipt`) that `enforce-commit-gate.sh`
validates at commit (§15). The receipt carries the fingerprint taken when the run started,
and the run fails instead of writing one when the gated files changed while it ran.

Tools: `cargo install cargo-audit cargo-deny cargo-shear cargo-llvm-cov cargo-nextest
cargo-sort taplo-cli typos-cli`, `rustup component add llvm-tools-preview`, semgrep 1.156.0
(`pipx install semgrep==1.156.0`), and `gitleaks shellcheck jq` from the distro; for gate:21,
`cargo install cargo-dylint dylint-link --locked` (6.0.4) and, from `tooling/lints`,
`rustup toolchain install` (its pinned nightly with `rustc-dev`, `rust-src` and
`llvm-tools-preview`);
`cargo-mutants` only for the end-of-sprint `script/mutation.sh`. Run the gate in the
Test phase; fix every red at the source. One cargo command at a time on this box: the target
directory is shared by every project on it.

---

## §3 — Phase Gates (binding)

Work flows through four phases, and that is all: **Plan → Code → Test → Complete**. **Every
phase has an entry gate: the previous phase must be `PASS` before the next begins**, and the
plan is presented for confirmation unless the user runs the work autonomously. The
`enforce-phase-gate.sh` PreToolUse hook blocks `Write`/`Edit` to application code until the
gate is satisfied.

```
/pipeline:plan        (Phase 1)  pick the item, pre-flight, recall; ticket + active spec/notes;
                                 the design and its test plan
  → /pipeline:code      (Phase 2)  the code, fmt- and clippy-clean, then a review of the diff
  → /pipeline:test      (Phase 3)  write + RUN the tests; the live drive; script/gates.sh --diff green
  → /pipeline:complete  (Phase 4)  docs (CHANGELOG + architecture, §21), knowledge, close the
                                 ticket, archive, commit
```

- **NEVER have two pipeline documents active** in `docs/planning/pipeline/active/`.
- A pipeline doc is a `<title>.spec.md` + `<title>.notes.md` pair. The `.spec.md` carries
  `pipeline_id:` (a real UUID) and `status:` frontmatter. Phase advance = setting
  `status: Phase N — <Title> PASS; ready for Phase M — <Title>`.
- Work not ready for a ticket lives in `docs/planning/intake/` (candidates, not active
  pipeline docs). `/spec` drafts a batch of Phase-1 specs into `docs/planning/pipeline/queued/`.
- Every work item gets a local ticket document in `docs/planning/tickets/{open,closed}/`, the
  canonical record, plus its row in `docs/planning/tickets/BACKLOG.md` while open and not yet
  promoted; promotion (`/pipeline:plan`) removes the row, completion closes the doc (§19).
- Pipeline acceptance criteria use EARS: `shall`, one observable behavior per requirement, a
  verification method for each.
- **Application code** = `crates/*/src/**/*.rs` and `crates/*/examples/**/*.rs`, Zed's crates
  included. Docs, `.claude/`, `docs/planning/`, config, `crates/*/tests/**`, and the
  milestone ticket lists are not gated.
- The `README.md` review marker rule in `.rules` stands: before the first source change of a
  piece of work the two `> [!IMPORTANT]` lines are present at the top of `README.md`, and only
  the human removes them.

---

## §7 — Testing Standards (binding)

**Full testing is expected.** Every pipeline produces meaningful tests for the code it
writes. Tests are not optional and not skippable because a change "looks simple".

- **Pure and library code** (the Marley crates' cores, any pure module added to a Zed
  crate): Rust `#[cfg(test)]` **unit tests**, **each EARS clause maps to at least one test**,
  plus `trybuild` compile-fail cases for type-safety contracts and doctests for public
  examples. Mutation testing (`script/mutation.sh`) runs once at the end of a sprint, not per
  change (§0).
- **UI code** (gpui render and input paths, in a Marley crate or a Zed crate): a change
  adds or updates a **driven test** on gpui's `TestAppContext` / `VisualTestContext` (the
  repo skill `.agents/skills/gpui-test` documents the harness; prefer the executor's timers
  over `smol::Timer`, per `.rules`), and `/pipeline:test` also runs the real app
  (`cargo run`), exercises the behavior, and captures it (a screenshot through the
  `dev-box-desktop` skill on this box). A green unit test never proves a pane works; the
  driven test plus the live drive do. The macOS AX harness of the gpui era is retired.
- **NEVER mark a phase PASS if tests did not actually RUN.** Writing a test file is not
  testing. The `enforce-tests-ran.sh` Stop hook checks the transcript for a real
  `cargo nextest run` / `cargo test` / `script/gates.sh` invocation at `/pipeline:test`.
- **Pre-existing failures are not your problem, but document them.** Note them in the notes
  as "pre-existing" and move on; don't fix unrelated breakage unless asked. Zed's suite is
  large; run the touched crates' tests, not the world.
- **Genuinely uncoverable paths** (a live GUI runtime, a bound port, a live service, a
  daemon socket, ssh) get a documented skip with the reason recorded in the notes and an
  explicit exclude in the gate (§0), never a silent regex.
- **Gate-is-test changes** (config, tooling, docs with no `.rs`) are verified by the gate's
  own exit codes plus **negative smokes** (inject the drift → the gate goes red → revert →
  green), not by inventing unit tests.

---

## §14 — Code Conventions (binding)

**Rust** (`crates/`):

- Zed's `.rules` applies to every crate: no `unwrap()`/`expect()` on paths an input or an
  external response can reach, never `let _ =` on a fallible call, no `mod.rs`, a library
  root named after the crate, full words for names, the GPUI entity and task rules, executor
  timers in tests. `expect()` is acceptable only for a genuine invariant, with a message that
  states it.
- Matches the surrounding code's idiom; `cargo fmt` is law (gate:1). clippy clean at
  `-D warnings`, `--all-targets` (gate:2). An `#[allow]` needs a trailing `//` justification
  (gate:12).
- Each Marley crate carries rustal's lint table in its own manifest: cargo gives a crate
  either the workspace's table or its own, so the seven tables repeat one another, and Zed's
  crates keep Zed's. Test code may `unwrap()` and `expect()` (the `clippy.toml` test
  allowances); library code may not. A crate-specific allow sits in that crate's table with
  a comment saying why (the gpui crate allows `future_not_send` and `unused_results`).
- `unreachable_pub` and `redundant_pub_crate` pull against each other in a private module.
  An item only its parent reaches is `pub(super)` in a module two or more levels deep; a
  module its siblings reach is declared `pub`; `#[allow(unreachable_pub)]` is never the
  answer.
- Newtypes own their invariants with **private** fields; cross-crate shared types have a
  single owner (`docs/specs/standards/seam-contracts.md` for the Marley crates) and are never
  re-declared downstream.
- Process spawn, PTY and sockets are core to the cockpit and permitted; keep them in adapter
  modules (`marley_terminal`, the harness and Rusty clients, `marley_mcp::transport`), use
  Zed's `util::command` / `smol` for non-PTY spawns, and validate inputs (no shelling out
  unsanitized user input).
- File IO is testable: route it through `*_in(dir)` functions with a directory override, so
  tests don't race on global state.
- `unsafe` carries a `// SAFETY:` note (gate:13) and is miri-clean (gate:6) or carries a
  spec-declared `miri-exempt` justification for FFI miri cannot model.
- No secrets in source; reuse existing helpers before adding new ones; comments explain
  *why*, not *what*.

**Upstream discipline** (the fork's own rule):

- A change to a Zed crate is the smallest diff that works, added behind a new module, a new
  event variant, a new panel or a setting; never a reformat, a rename sweep or a style pass
  over upstream code. Rebasing on upstream must stay cheap.
- Every change outside the Marley-owned paths gets its row in
  `docs/marley/zed-touchpoints.md` in the same change (what changed, why, what to do at a
  merge), and a code hunk carries a `// Marley: <why>` comment. `enforce-zed-ledger.sh`
  blocks a Write or Edit to such a path until the row exists, and the same check in
  `enforce-commit-gate.sh` refuses a commit that still lacks one, so a change made any other
  way is caught too (gate:16, §0).
- A change Marley needs in a dependency Zed forks (the alacritty fork) lives on a Marley
  branch of that fork, one file wide where possible, pinned by rev in `Cargo.toml`.
- Marley-owned crates keep the `marley_` prefix and `MIT OR Apache-2.0` (§20).

---

## §15 — Anti-Circumvention (binding)

**The transcript is the source of truth. If it didn't happen in the transcript, it didn't
happen.** Claiming "tests pass" without a visible test run is a violation. Claiming the gate
is green without running `script/gates.sh` is a violation. Hooks evaluate evidence (tool
calls, Bash commands, file state), not prose.

Do not weaken a gate, delete a test, lower a coverage floor, or add a blanket `#[allow]` to
get past a blocked Stop. Fix the cause.

**What the enforcement is, and isn't.** The hooks are a *discipline scaffold*, not a security
boundary. They reliably catch **omissions**: writing code before a phase is PASS, stopping a
phase with unresolved tasks or an un-advanced doc status, leaving `/pipeline:test` without
running tests, committing code without a green gate. They do **not** try to defeat
deliberate fabrication: the `status:` line and the test calls are self-reported. The one
hard, evidence-based gate is **`script/gates.sh` at commit**: `enforce-commit-gate.sh` blocks
a `git commit` that includes Rust source unless a `--full`/`--diff` gate run left a
*receipt* (`.git/ignibyte-gate-receipt`, a content fingerprint of every `crates/**/*.rs` and
every file under `crates/marley_*` in the tree) that still matches the worktree being
committed. Each such run removes the earlier receipt before its first gate, so a tree that
passed once and fails later cannot commit on the older green. The receipt is written only by a real
FULL/`--diff` green, so the verdict cannot be forged by printing or quoting `GATE GREEN`;
any edit after the green, by Write, Edit or a Bash heredoc, changes the fingerprint and
re-blocks. A **second** commit-time hook, `enforce-changelog.sh`, blocks a Rust-source commit
that lacks a `CHANGELOG.md` entry (§21). A change that touches **no** `.rs` is not blocked by
the receipt; its gate is enforced by pipeline discipline (the static gates at
`/pipeline:test`). The receipt fingerprint binds not just `crates/**/*.rs` but the
**gate-defining files** themselves (`script/gates.sh`, `.claude/hooks/**`, `clippy.toml`,
`rustfmt.toml`, `deny.toml`, `.gitleaks.toml`, `.semgrep.yml`, `.config/typos.toml`,
`.cargo/audit.toml`, the Cargo manifests and lockfile, the toolchain pin, the nextest
config, and `tooling/lints`, gate:21's library and its nightly pin), so weakening the gate
after a green invalidates the receipt.

---

## §18 — Review & Explore (binding)

**§18.1 — The review is part of Code.** There is no separate inspect phase since
2026-09-22. The Code phase ends with a review of its own diff: correctness against each
acceptance criterion, gpui entity re-entrancy, errors reaching the UI, provenance (§20) and
upstream discipline (§14). Independent critics are optional, for a change large enough to
want them.

**§18.2 — Delegate broad file-discovery to the Explore subagent.** Any lookup with more than
~3 candidate paths, or where the location isn't known a priori, goes through
`Agent(subagent_type=Explore)`. Inline grep walks don't substitute. In a workspace of 250
crates this matters more, not less.

**§18.3 — Local knowledge first (§19).** Before planning and before implementing, recall
prior knowledge locally: grep `docs/planning/knowledge/` (prevention rules, failures,
lessons, architecture decisions) and the completed-pipeline notes
(`docs/planning/pipeline/completed/`). At Complete, capture what you learned by APPENDING
ledger blocks: failures and prevention rules for the real bugs found, lessons, and
architecture decisions (§19 formats). The Rusty brain loop (`brain_ask` before a design
choice, `brain_decide` after) is the house-wide twin of this rule and applies here too.

---

## §19 — Local Knowledge & Tickets (binding)

**Tickets are local files.** `docs/planning/tickets/open/TICKET-<n>-<slug>.md` is the
canonical work item (closed → `tickets/closed/`). `tickets/BACKLOG.md` is the ordered queue:
`/pipeline:plan` with no argument takes the TOP row of its **Queue** section; **Deliberate** rows are
only picked explicitly. Ticket numbering = 1 + the max number across `open/` + `closed/`,
continuing from the gpui era (the archive reaches #435); never renumber, never reuse.

**Knowledge is a local ledger.** `docs/planning/knowledge/` holds `prevention-rules.md`,
`failures.md`, `lessons.md`, `architecture-decisions.md`: one `## <code>` block per entry,
`## <code>` + an italic meta line (severity / category / topic / status, as fits the file) +
the body. NEW appends use `PR-…`, `F-…`, `L-…`, `AD-…` codes; the exported corpus keeps its
sidecar-era prefixes (`BF-…`, `DL-…`, and a few odd historical codes). The ledger is
append-only at the tail; existing blocks are history. Recall = grep (§18.3); capture = the
phase-close appends. The ledgers, the ticket archive and the completed-pipeline archive were
carried into this fork on 2026-09-18 with their history intact; entries that describe the
gpui-era app (its `marley_app` shell, the React parity twin, the macOS harness) are read as
history, and their lessons about gpui, PTYs, alacritty and the pipeline itself still apply.

---

## §20 — Provenance (binding)

Marley exists to give Ignibyte a Warp-class cockpit it **owns outright**. The fork changes the
shape of the rule, not its purpose:

- **The Warp wall stands.** Terminal, block and cockpit behavior is implemented from the
  behavior specs in `docs/warp_architecture/` and observed captures in
  `docs/warp_architecture/observed/`, **never** by reading or translating the AGPL Warp
  source. A reworded translation of copyleft source is still a derivative work.
- **Zed is this codebase.** The fork is GPL-3.0-or-later where Zed is; reading, changing and
  extending Zed's crates is normal work, and their source is the first place to look.
  Marley-owned crates stay `MIT OR Apache-2.0` and distinct from upstream code, so the
  provenance of each line is a path, not a review.
- **The brain boundary stands.** The manager agent and any sold brain are **separate
  programs** speaking MCP across the seam Marley exposes (`marley_mcp`) and the seams it
  consumes (rustal-brain, the harness runtime, Rusty). They are never linked into this
  process; mere aggregation over a wire is not a derivative work, linking is.
- **Forced reference (`enforce-warp-reference.sh`).** Every pipeline spec carries a
  `## Reference (§20)` section naming the reference behavior: Warp for the terminal, blocks
  and cockpit (a `docs/warp_architecture/` citation or an observed capture), upstream Zed for
  editor and workspace behavior (cite the crate and the behavior kept), or
  `N/A — Marley-specific + why`. Plan fills it and its design confirms it; the commit hook
  blocks a staged spec whose section is empty. The hook gates PRESENCE, not correctness; the
  Code phase's review judges the match (§18.1).
- **Forced prior art (`### Prior art`, required at Plan).** The wall says what we may not
  read. It does not excuse reinventing what is already ours to take. Every spec records a
  sweep of three sources, and the hook blocks a spec that leaves it empty:
  1. **The behavior maps and observed captures**: `docs/warp_architecture/`,
     `docs/zed_architecture/`. Research, not source.
  2. **Published material**: docs, blogs, talks, the LSP, MCP, ACP and CDP specs.
  3. **The code we already ship**: Zed's own crates and every dependency in `Cargo.lock`.
     Reading them is adoption, and it is the highest-yield leg. Ask plainly: *does a crate we
     already build own this seam?* (gpui, `editor`, `terminal`, `task`, `context_server`,
     `agent_servers`, alacritty, vte, tree-sitter, `regex` …)
  If the sweep finds nothing, say so and say where you looked; silence is not a filled
  section.
- **The code-layer wall**: the Code phase's review (§18.1) includes a provenance check: no
  code structurally derived from Warp's source, and no Zed function body carried into a Marley
  crate.
- IP-counsel sign-off remains pending before commercializing (the gpui-era note in
  `docs/decisions/licensing-ownership-strategy.md` carries the reasoning).

---

## §21 — Documentation Phase (binding)

Context must never be lost. **Every pipeline's Phase 4 (Complete) shall, without exception,
do both:**

1. **Add a `CHANGELOG.md` entry** for the change (root `CHANGELOG.md`, Keep a Changelog
   format). This half is **machine-enforced**: `enforce-changelog.sh` blocks a `git commit`
   that includes Rust source (`crates/**/*.rs`) unless `CHANGELOG.md` is in the same
   changeset. A no-`.rs` change is exempt.
2. **Update the architecture docs**: the prong's section of `docs/marley/three-prong-plan.md`
   (slice status), the per-crate notes under `docs/marley_architecture/` for a Marley crate,
   and, for a change outside the Marley-owned paths, a check that its row in
   `docs/marley/zed-touchpoints.md` still describes what shipped (the row itself is written
   before the change, §14). This half is a **required** Phase-4 step.

Skipping either is a charter violation. The CHANGELOG keeps the *what/why* of every change;
the architecture docs keep the *shape* of the system.

---

## Amending this Constitution

These rules change deliberately, not mid-pipeline to dodge a gate. To amend: state the
section, the change, and the reason in a commit that touches only this file (and any hook
or gate that enforces the changed rule). Raising a floor or tightening a convention needs no
ceremony; loosening one needs a recorded reason. The 2026-09-18 port from the gpui-era repo
is the standing example: gate:15 retired, gate:9 moved to cargo-shear, gates 7, 8, 10 and 14
re-scoped to what a fork can honestly gate, each with its reason in §0. On 2026-09-22 Chad
loosened two rules, with the reason recorded in §0 and §3: mutation testing left the
per-change gate for `script/mutation.sh` at the end of a sprint, and the workflow became
four phases (`/work`, design, inspect and `/commit` folded into Plan, Code and Complete).
