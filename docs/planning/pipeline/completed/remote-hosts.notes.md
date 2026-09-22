# known-hosts config + a 'connect to…' palette action — Notes

- **Forge ticket:** #87 `4bdd76e9-296a-42af-a34f-7a9b125fa516` (BACKLOG — M3.A sprint pending)
- **AAR:** `08963f50-f4b5-4d26-be9a-d6fd53b43b20`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-087-remote-hosts.md

## Phase 1 — Plan
- **Request:** forge #87 (M3.A seq-5, auto-approved, LAST M3.A) — named ssh hosts in settings + a palette
  connect action.
- **Classification:** work pipeline, `feature`, PURE (marley_remote) + SETTINGS (define_setting!) + an
  app.rs SHIM. UI.
- **Pre-flight facts:** marley_settings `define_setting!(pub Name: Value = default, "path")`, Value = any
  serde type (blanket SettingsValue over Serialize+DeserializeOwned) — a `Vec<RemoteHost>` setting works;
  the 5 existing settings live in marley_app/src/settings.rs. The palette: `commands: Vec<Command{id:
  CommandId(u32),title,keywords,binding}>` from `cockpit_commands()` (ids 0,1,2,4,5); Enter →
  `activate() -> Option<CommandId>` `.and_then(action_for_command)` `.map(str::to_string)` →
  `dispatch_action`. action_for_command is a static id→&'static str map. serde is workspace-standard
  (marley_settings/marley_forge_client use `serde = { version = "1", features=["derive"] }`). The #84
  open-remote arm inlines spawn_remote_session + split_focused + remotes.insert(Connected) + flash.
- **Decisions:** D1 CONNECT_BASE=1000 id range + defensive .get; D2 extract open_remote_target (shared by
  cmd-shift-o + palette; cmd-shift-o still clears the compose prompt); D3 invalid target dropped; D4
  RemoteHost gains serde.
- **Self-test:** a settings TOML with a localhost host → ⌘⇧P → 'connect: …' → the pane. **May be
  ENV-BLOCKED** (synthetic input degraded this session — #86); if so, unit-verify + state it.
- **AAR id:** `08963f50-f4b5-4d26-be9a-d6fd53b43b20`.

## Phase 2 — Design

### PURE — `marley_remote` (+ serde dep)
```rust
use serde::{Deserialize, Serialize};

/// A named, saved ssh host (#87) — a `[remote.hosts]` settings entry.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RemoteHost {
    pub name: String,
    pub target: String,
}

/// A command-palette "connect: …" action for a saved host (#87).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RemoteAction {
    pub label: String,
    pub target: SshTarget,
}

/// Build the palette connect actions from saved hosts (#87): each host with a VALID target yields a
/// `connect: {name} → {target}` action; a host with an invalid target is DROPPED (not a panic).
pub fn remote_palette_actions(hosts: &[RemoteHost]) -> Vec<RemoteAction> {
    hosts
        .iter()
        .filter_map(|host| {
            parse_ssh_target(&host.target).map(|target| RemoteAction {
                label: format!("connect: {} → {}", host.name, host.target),
                target,
            })
        })
        .collect()
}
```
- `marley_remote/Cargo.toml`: `serde = { version = "1", features = ["derive"] }`.

### SETTINGS — `marley_app/src/settings.rs`
- `use marley_remote::RemoteHost;`
- `define_setting!(pub RemoteHosts: Vec<RemoteHost> = Vec::new(), "remote.hosts");`
- `AppliedSettings` gains `pub remote_hosts: Vec<RemoteHost>` (drop the `Eq` derive → keep `PartialEq`
  [Vec<RemoteHost> is Eq since String/Vec are Eq, so `Eq` stays fine]).
- `applied_from`: `remote_hosts: manager.get::<RemoteHosts>()`.
- `applied_defaults`: `remote_hosts: RemoteHosts::default_value()`.

### SHIM — `app.rs` (mutants::skip + cov-excluded)
- import `RemoteAction, remote_palette_actions` (+ `RemoteHost` via settings); `const CONNECT_BASE: u32 = 1000;`
- `RootView.remote_actions: Vec<RemoteAction>`.
- boot: `let remote_actions = remote_palette_actions(&applied.remote_hosts);` → build `commands`:
```rust
let mut commands = cockpit_commands();
for (i, action) in remote_actions.iter().enumerate() {
    commands.push(Command {
        id: CommandId(CONNECT_BASE + i as u32),
        title: action.label.clone(),
        keywords: vec!["connect".into(), "ssh".into(), "remote".into()],
        binding: None,
    });
}
```
  store `commands` + `remote_actions` in RootView.
- extract `open_remote_target` (shared by cmd-shift-o + palette):
```rust
#[cfg_attr(test, mutants::skip)]
fn open_remote_target(&mut self, target: SshTarget) -> bool {
    let argv = ssh_command(&target);
    let host = target.host;
    if let Ok(pane_id) = self.workspace.split_focused(PaneAxis::Horizontal, SplitDirection::After, || {
        spawn_remote_session(&argv, &self.zdotdir, self.term_cols, self.term_rows)
    }) {
        self.status_flash = Some(Flash::new(format!("ssh {host}")));
        self.remotes.insert(pane_id, Remote { host, status: RemoteStatus::Connected });
        true
    } else { false }
}
```
- cmd-shift-o arm → `if let Some(t)=parse { if self.open_remote_target(t) { clear compose } }`.
- palette Enter → branch:
```rust
if let Some(id) = self.palette.activate(&filter_commands(&self.commands, self.palette.query())) {
    if let Some(action) = action_for_command(id) {
        self.dispatch_action(action);
    } else if let Some(target) = id.0.checked_sub(CONNECT_BASE)
        .and_then(|i| self.remote_actions.get(i as usize)).map(|a| a.target.clone()) {
        self.open_remote_target(target);
    }
}
self.palette_open = false;
```

### File manifest
- MODIFY `crates/marley_remote/src/lib.rs` — RemoteHost + RemoteAction + remote_palette_actions + tests.
- MODIFY `crates/marley_remote/Cargo.toml` — serde dep.
- MODIFY `crates/marley_app/src/settings.rs` — RemoteHosts setting + AppliedSettings.remote_hosts +
  applied_from/applied_defaults + a load test.
- MODIFY `crates/marley_app/src/app.rs` — import + CONNECT_BASE + remote_actions field + boot build +
  open_remote_target + cmd-shift-o + palette dispatch.

### Mutation Targets (pure)
- `remote_palette_actions`: the `filter_map` drop (a bad target excluded), the label `format!`.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `remote_palette_actions_labels` — 2 valid hosts → 2 actions, labels "connect: name → target", parsed targets | unit |
| REQ-002 | `remote_palette_actions_drops_invalid` — hosts w/ ""/"-x"/"::1" targets dropped; a mix keeps only valid | unit |
| REQ-003 | `remote_hosts_setting_loads` — a TOML `[[remote.hosts]] name=… target=…` round-trips via the manager; absent → empty; (malformed → default/empty) | unit (marley_app/settings.rs) |
| REQ-004 | a `localhost` host → ⌘⇧P → "connect: …" → opens the pane | self-test (may be env-blocked — #86) |
| REQ-005 | gate GREEN, cov/MSI 100 pure; shim masked | gate |

Uncoverable: the connect-command build + palette dispatch + open_remote_target — masked (gpui), REQ-004.

### Risks / decisions
- D-2.1 CONNECT_BASE=1000 (static ids are ≤5) — `checked_sub` + `.get` so a stale/out-of-range id can
  never panic or mis-dispatch. D-2.2 open_remote_target extraction must preserve #84 (spawn) / #85 (tag) /
  #86 (Connected) / #77 (flash) — the cmd-shift-o self-test path is unchanged, only refactored. D-2.3 the
  RemoteHosts `get` tolerance for REQ-003 (absent→default; malformed→?) is VERIFIED in the settings test.
  D-2.4 `remote_actions` + `commands` are built ONCE at boot (a settings change needs a reload — out of
  scope; the file is read-only in-app per M1.B).

## Phase 3 — Implement
- **Built (marley_remote):** `RemoteHost{name,target}` (serde derive) + `RemoteAction{label,target}` +
  `remote_palette_actions(hosts)` (filter_map: parse each target, drop invalid, label "connect: {name} →
  {target}"). + serde dep.
- **Built (marley_app/settings.rs):** `define_setting!(pub RemoteHosts: Vec<RemoteHost> = Vec::new(),
  "remote.hosts")`; `AppliedSettings.remote_hosts`; applied_from/applied_defaults resolve it.
- **Built (marley_app/app.rs):** import + `const CONNECT_BASE: u32 = 1000`; `RootView.remote_actions:
  Vec<RemoteAction>`; boot builds `remote_actions = remote_palette_actions(&applied.remote_hosts)` + appends
  a `Command{CommandId(CONNECT_BASE+i), title: label, keywords, binding: None}` per action to `commands`;
  extracted `open_remote_target(&mut self, SshTarget) -> bool` (the #84 spawn/split/tag/flash) — the
  cmd-shift-o arm now calls it + clears the compose prompt on success; the palette Enter branches
  `action_for_command(id)` else `id.0.checked_sub(CONNECT_BASE).and_then(get).map(clone) → open_remote_target`.
- **Deviations:** none.
- **Verification:** `cargo fmt`; `cargo check -p marley -p marley_remote` 0 err; clippy `-D warnings` OK;
  `cargo nextest -p marley_remote` 8 pass (the #87 tests are Phase 4). The palette dispatch/build + the
  spawn are masked; remote_palette_actions is the pure surface.

## Phase 3.5 — Inspect
- **Critic:** 1 (correctness + data-integrity + MSI; ran cargo-mutants + a live settings round-trip probe +
  read the SettingsManager::get impl). Verdict: **REQUEST CHANGES — 2 mechanical issues, no logic defect.**
- **Confirmations:** (a) remote_palette_actions labels/drops correct — "connect: prod → deploy@prod:22"
  (arrow byte-checked U+2192), a dropped host yields NO action (filter_map), mixed keeps order+only-valid.
  (b) **the settings round-trip + get-tolerance is SOLID (REQ-003):** `toml_path "remote.hosts"` resolves
  `[[remote.hosts]]` (array-of-tables) ↔ Vec<RemoteHost>; get is FULLY tolerant — absent key / missing
  file / wrong-type / missing-field / table-not-array ALL → the empty default, NO panic (`resolve_typed` =
  `from_file_value().unwrap_or_else(default)`, `from_file_value` = `try_into().ok()`); boot maps any load
  error to applied_defaults → **no config can crash boot.** (c) the connect-id dispatch can't panic/collide
  — checked_sub + .get are total; static ids ≤5, connect ids ≥1000, no overlap; clone-before-&mut, no
  double-borrow. (d) open_remote_target is BEHAVIOR-EQUIVALENT to the #84 inline arm (same ssh_command/
  split/flash/insert; only returns bool + the compose-clear moved to the caller, gated on the bool). (e)
  serde is workspace-standard (no new [[package]] beyond the serde edge); §20 original; no secrets.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | F1 | **HIGH** | The 3 `AppliedSettings{}` test fixtures (settings.rs) weren't updated for the new `remote_hosts` field → `cargo check --all-targets` fails (test build broken). My implement check omitted `--all-targets`. | **FIXED** — added `remote_hosts: Vec::new()` to all 3; `check --all-targets` + `clippy --all-targets` now clean. → PR recorded. |
  | F2 | HIGH | `remote_palette_actions` has no tests yet → the whole-body mutant survives (MSI 95.2%) + 0% cov. | **Phase 4** — the test plan's remote_palette_actions tests (valid/dropped/empty/mixed) close it (cargo-mutants emits only the whole-body mutant → one test set kills it + asserts labels/drops). |
  | F3 | LOW | A structurally-malformed `[[remote.hosts]]` entry drops ALL hosts (the Vec try_into is all-or-nothing), vs parse_ssh_target's per-entry drop. | Accept + DOCUMENT — tolerant (no crash, REQ-003 holds); inherent to the typed-Vec setting; a per-entry-tolerant deserializer is a future option. |
- **Fix applied (code):** F1 (the 3 fixtures). F2 → Phase 4. F3 documented.

## Phase 4 — Validate
- **Tests added:** `remote_palette_actions_labels` (REQ-001 — 2 valid hosts → 2 labeled actions + parsed
  targets), `remote_palette_actions_drops_invalid` (REQ-002 — ""/"-x"/"::1"/"h:bad" dropped; empty→empty;
  mixed keeps order+only-valid) [marley_remote]; `remote_hosts_setting_loads_and_tolerates` (REQ-003 — a
  `[[remote.hosts]]` round-trips; absent→empty; malformed string→empty, no panic) [settings.rs].
- **Runs (actual):** `cargo nextest -p marley_remote` → 10 passed (F2 closed — the whole-body mutant is now
  killed); `cargo nextest -p marley -E 'test(remote_hosts_setting)'` → 1 passed.
- **SELF-TEST (UI — REQ-004) — HARNESS ENV-BLOCKED (stated, per #86):** probed once more (fresh build +
  caffeinate wake) — synthetic input still did NOT land + the window capture failed (same TCC/display-state
  degradation as #86, ~7 attempts across #86/#87 now). Not retried further (anti-rabbit-hole). **REQ-004's
  chain is otherwise verified piece-by-piece:** (1) the connect actions + labels are pure-tested
  (remote_palette_actions, cov/MSI 100); (2) the settings load is tested (the round-trip); (3) the critic
  confirmed the connect-id palette dispatch → `open_remote_target`; (4) `open_remote_target` is
  byte-equivalent to the #84 open path, which was LIVE-PROVEN this session (#84 ssh child + #85 ⇄ badge).
  The only un-driven step is the palette-select click, on tested + live-proven code.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100%. remote_palette_actions + the settings load tested; the palette dispatch masked.
- **Pre-existing:** none.

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; marley_remote.md "## Saved hosts + the connect palette (#87)" + "## M3.A COMPLETE".
- **Knowledge:** aar-submit (4); PR-claude-implement-cargo-check-all-targets-catches-fixture-breaks-001 (adding a struct field breaks test-literal fixtures — check --all-targets in implement). Critic REQUEST-CHANGES → 2 mechanical fixes (fixtures + tests), no logic defect; the settings get-tolerance is solid (no config crashes boot). Self-test env-blocked (#86 harness).
- **Ticket:** forge #87 → done. **5/5 of M3.A — THE REMOTE SEAM IS COMPLETE.** (tickets in the forge backlog tagged M3.A — sprint-create still erroring, 10×.) Saved hosts + a connect palette action.
