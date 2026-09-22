# marley_agent — the agent-run model — Notes

- **Forge ticket:** #61 `57ebab5f-adab-4d76-a202-0ab33ab3e56b`
- **AAR:** `0e27da39-7932-47a1-b222-f98ef51c28d6`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-061-agent-model.md

## Phase 1 — Plan
- **Request:** forge #61 (M2.B seq-3, auto-approved) — the pure agent-run model, the cockpit foundation.
- **Classification:** work pipeline, `feature`, a PURE new crate `marley_agent`. No UI.
- **Decisions:** D1 recognize claude+codex; D2 exited-first (mirror exit_status_kind #36); D3 no Default
  on AgentRun (per the #54/#55 unviable-mutant pattern); D4 new gpui-free std-only crate.
- **Coverage note:** `agent_kind_of` uses `split_whitespace().next()?` (None for empty → coverable via
  the `""` test) + `rsplit('/').next().unwrap_or(token)` (the basename; `unwrap_or`'s fallback is dead
  but it's ONE line → line-covered, gate:4 is LINE coverage). agent_status_from + the match arms give
  real viable mutants.
- **AAR id:** `0e27da39-7932-47a1-b222-f98ef51c28d6`.

## Phase 2 — Design

### PURE — `crates/marley_agent/src/lib.rs` (NEW crate)
```rust
/// A recognized agent CLI.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentKind { Claude, Codex }

/// Classify a command line as an agent CLI by its leading program (path-stripped); args are ignored.
pub fn agent_kind_of(command: &str) -> Option<AgentKind> {
    let token = command.split_whitespace().next()?;
    let program = token.rsplit('/').next().unwrap_or(token);
    match program {
        "claude" => Some(AgentKind::Claude),
        "codex" => Some(AgentKind::Codex),
        _ => None,
    }
}

/// An agent run's live status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentStatus { Idle, Working, Exited }

/// Project a pane's flags into an agent status: an EXITED agent is `Exited` (checked first, regardless
/// of a stale `active`); an active one is `Working`; otherwise `Idle`.
pub fn agent_status_from(exited: bool, active: bool) -> AgentStatus {
    if exited {
        AgentStatus::Exited
    } else if active {
        AgentStatus::Working
    } else {
        AgentStatus::Idle
    }
}

/// A running (or finished) agent terminal: what it is, a display label, and its status.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun {
    pub kind: AgentKind,
    pub label: String,
    pub status: AgentStatus,
}

impl AgentRun {
    /// A fresh agent run — the given kind + label, starting `Idle`.
    pub fn new(kind: AgentKind, label: String) -> AgentRun {
        AgentRun { kind, label, status: AgentStatus::Idle }
    }
}
```
`Cargo.toml`: minimal-crate pattern (`edition.workspace`/`license.workspace`); NO deps.

### File manifest
- NEW `crates/marley_agent/Cargo.toml`.
- NEW `crates/marley_agent/src/lib.rs` — the types/fns + tests.
- (workspace `members = ["crates/*"]` auto-includes it; `--workspace` cov/mutation covers it.)

### Mutation Targets
- `agent_kind_of`: `split_whitespace().next()?` (empty → None, `""` test), the basename, the `claude`/
  `codex`/`_` match arms. `agent_status_from`: `if exited` FIRST (a `(true,true)→Exited` test kills a
  read-active-first mutant), `else if active`, else. `AgentRun::new`: `status = Idle`.

### Regression Test Plan (all pure unit; no UI)
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `agent_kind_of_recognizes_and_rejects` — `claude`→Claude, `claude --resume`→Claude (args ignored), `/usr/bin/claude`→Claude (path-stripped), `codex`→Codex, `ls`→None, ``→None | unit |
| REQ-002 | `agent_status_from_exited_first` — `(true,true)`→Exited, `(true,false)`→Exited, `(false,true)`→Working, `(false,false)`→Idle | unit |
| REQ-003 | `agent_run_new_starts_idle` — `AgentRun::new(Claude, "x")` → kind Claude, label "x", status Idle | unit |
| REQ-004 | gate GREEN, cov/MSI 100 | gate |

Uncoverable: none (all pure). `unwrap_or`'s fallback is dead-but-line-covered (gate:4 = LINE cov).

### Risks / decisions
- D-2.1 exited-first (D2) — the `(true,true)→Exited` test is load-bearing (kills a branch-reorder mutant).
- D-2.2 no `Default` on AgentRun (D3) — the whole-body mutant stays unviable; the file has viable
  mutants from the match arms + the status branches, so not the 0-viable trap. D-2.3 crate is an orphan
  (no consumer yet) — `--workspace` still builds/covers it; #62 is the first consumer.

## Phase 3 — Implement
- **Built:** `crates/marley_agent/{Cargo.toml, src/lib.rs}` — `AgentKind`, `agent_kind_of`, `AgentStatus`,
  `agent_status_from`, `AgentRun` + `new` (verbatim from the design). Crate + all items documented (gate:14).
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley_agent` 0 err; clippy `-D warnings` OK; `cargo
  metadata` confirms it's in the workspace. Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (25-case probe + a clean `-j1` cargo-mutants + coverage measurement). Verdict: **PASS —
  code correct.** cov 100 (28/28 regions), MSI 100 (5 mutants / 3 viable / 3 caught / 2 unviable / 0
  survivors — the 3 planned tests reach it).
- **Findings (no code change — test-adequacy notes, hollow-MSI family):**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | MED | `agent_status_from` has ZERO viable mutants (cargo-mutants only makes the unviable `Default::default()` whole-body — no branch-swap/negation for bare `bool` ifs). So MSI 100 is vacuous for it; the EXITED-FIRST ordering is guarded ONLY by the REQ-002 assertion. | P4 writes REQ-002 with `(true,true)→Exited` (+ (t,f)/(f,t)/(f,f)) — the real guard. Same family as exit_status_kind/#53. |
  | F2 | LOW | `AgentRun::new` generates 0 mutants; `status=Idle` guarded only by REQ-003. | P4 REQ-003 asserts Idle. |
  | — | INFO | `unwrap_or(token)` fallback is dead (rsplit always Some) but LINE-covered + panic-free (preferable to unwrap). | Keep. |
  | — | INFO | cargo-mutants with a shared CARGO_TARGET_DIR + `-j>1` mis-attributes (build-dir race) — the real gate uses copy-mode, safe. | Noted. |
- **Verified (probe):** agent_kind_of ALL 25 cases (claude/args/path/./claude → Claude; codex → Codex;
  ls/``/whitespace/claude/foo/CLAUDE → None; case-sensitivity intended); agent_status_from exited-first
  correct; AgentRun::new → Idle; the Default mutants rustc-confirmed UNVIABLE (excluded, not missed);
  Cargo.toml matches siblings; orphan crate fine; no panic path; §20 trivial classifier.
- **No code change** — F1/F2 = the P4 behavioral tests (which the gate + the assertions guard).

## Phase 4 — Validate
- **Tests added** (`lib.rs`): `agent_kind_of_recognizes_and_rejects` (REQ-001 — claude/args/path/codex/
  ls/empty/whitespace), `agent_status_from_exited_first` (REQ-002 — `(t,t)→Exited` per F1 + the 3 others),
  `agent_run_new_starts_idle` (REQ-003).
- **Runs (actual):** `cargo nextest run -p marley_agent` → 3 passed.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, coverage 100%, MSI 100.0%. (First run hit
  the known env PTY-test hang under a stray mutants proc's load — killed strays, re-ran clean.)
- **UI:** N/A — pure library crate.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; crate-map node+row; new arch doc marley_agent.md.
- **Knowledge:** aar-submit completed (5). Reused the hollow-MSI family (agent_status_from/new() 0-viable → behavioral assertions are the guard). No new rule.
- **Ticket:** forge #61 → done; archived. **3/6 of M2.B** (halfway) — the cockpit model foundation is in.
