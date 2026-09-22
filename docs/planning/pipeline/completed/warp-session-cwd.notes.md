# session persistence — restore each terminal's cwd — Notes

- **Forge ticket:** #205 (85a7388c-4d30-427c-a1b6-163ffb97edfb)
- **AAR:** 592d2ba3-0e6b-461f-b088-c4769c8f3726
- **Local ticket doc:** docs/planning/tickets/open/TICKET-205-warp-session-cwd.md
- **Pipeline spec:** warp-session-cwd.spec.md

## Phase 1 — Plan
- **Request:** forge #205 — restore tabs + panes on relaunch INCLUDING each pane's cwd. #163 already
  restores structure/kinds/actives; the gap is per-terminal cwd.
- **Classification / tier:** work pipeline — a bounded codec extension + a shim wire on top of #163. ONE
  coherent slice (the ticket's "split capture-model vs restore-wiring" is unnecessary — the cwd rides the
  existing #163 codec + spawn path).
- **Forge recall (§18.3):** aar-open → 592d2ba3. Deps: grid_layout.rs (#163 codec), workspace layout,
  marley_settings, the #201 `current_prompt().pwd` source, `spawn_session_in`. Pivoted here from #204
  (shipped) / #202 (parked).
- **Discovery (code read):**
  - `grid_layout.rs::serialize_grid(group, kinds: &HashMap<PaneId, PaneKind>)` (58) flattens the
    PaneGroup to a `Vec<PaneKind>` (via `flatten`) and emits kind-chars (`kind_char`: Terminal→'t'):
    a single pane → `"t"`; a split → `"H:t,f,c"` (axis + comma-joined chars). **NO cwd captured.**
    `restore_grid` (94) parses it back; `char_kind` maps chars→PaneKind.
  - `serialize_shell`/`restore_shell` (203/252) wrap the grid blobs per tab (`T=<blob>`), per project
    (`root \t active_tab \t entries…`). `breaks_framing` (181) drops a project whose root has `\t\n\r`
    (the #163 D2 stance) — reuse/extend for the cwd.
  - Boot restore: `restore_panes` (app.rs:791) spawns terminals via `spawn_session_in(root, zdotdir,
    cols, rows)` (:800) — in the PROJECT ROOT. `spawn_session_in(cwd, zdotdir, cols, rows)` (:441)
    already takes a cwd → the wire exists; we just pass the per-pane cwd (exists-checked) instead of root.
  - The live cwd source: `session.current_prompt().pwd` (the #201 `live_tab_title` source; tracks `cd`).
- **The bounded delta:**
  - PURE: extend the terminal leaf `"t"` → `"t=<cwd>"` (optional; framing-safe or drop). Requires
    `serialize_grid` to also take the cwds (a `HashMap<PaneId, String>`), `flatten` to collect
    `(kind, Option<cwd>)` per leaf, and `restore_grid`/`GridLayout` to carry the per-leaf cwd. cov/MSI 100
    (round-trip + framing-drop + back-compat; run `cargo mutants --list`).
  - SHIM: the serialize caller builds the cwds map from each terminal's `current_prompt().pwd`;
    `restore_panes` reads the per-pane cwd + `spawn_session_in(cwd if Path::exists else root, …)`.
- **Decisions:** D1 cwd on the leaf encoding (optional, #177 precedent); D2 framing-safe or drop (reuse
  breaks_framing); D3 live cwd = current_prompt().pwd; D4 exists-check → root fallback; D5 clean-room.
- **Open for design:** the exact `serialize_grid` signature change (cwds map param) + how the shell-level
  serialize supplies it; the `GridLayout` cwd field shape; the `breaks_framing` delimiter set (add `,`
  `:` `=`); whether the cwds map keys by PaneId (needs the flatten to preserve ids — flatten currently
  drops ids; design decides: thread ids through flatten, or a parallel Vec in visual order).

## Phase 2 — Design

### Architecture / approach
The cwd rides INSIDE the grid blob (a pre-serialized `String` in `TabLayout::Terminal.blob`), so the
change is contained to the pure `grid_layout.rs` codec + a masked app.rs shim (capture + restore-spawn).

**Key structure facts:** `GridLayout { axis, kinds: Vec<PaneKind> }` (the parsed model restore reads);
`serialize_grid(group, kinds: &HashMap<PaneId,PaneKind>) -> String` flattens to kind-chars; the shell
codec embeds the blob string in `TabLayout::Terminal { blob, title }`; `restore_panes` (app.rs:791) does
`restore_grid(blob)` then spawns panes `1..N` via `spawn_session_in(root, …)` (pane 0 is the base session
spawned at :839, or the reused boot PTY).

**PURE codec (grid_layout.rs):**
- `GridLayout` gains `cwds: Vec<Option<String>>` (parallel to `kinds`, visual order). `default_grid()`
  → `cwds: vec![None]`.
- `flatten` → collects `Vec<(PaneKind, Option<String>)>`: it has the `PaneId` at each `Leaf(id)`, so it
  looks up a new `cwds: &HashMap<PaneId, String>` param → `Some(cwd)` for a Terminal leaf whose cwd is
  framing-safe, else `None`. Signature: `flatten(group, kinds, cwds, out: &mut Vec<(PaneKind,
  Option<String>)>)`.
- `serialize_grid(group, kinds, cwds) -> String`: flatten → leaves; ≤1 → `serialize_leaf(&leaves[0])`;
  split → `"{axis}:"` + `leaves.map(serialize_leaf).join(",")`.
- `serialize_leaf((kind, cwd)) -> String`: `Terminal` + `Some(cwd)` → `format!("t={cwd}")`; else
  `kind_char(kind).to_string()`. (The cwd is already framing-safe — filtered in `flatten`.)
- `breaks_grid_framing(s) -> bool = s.contains(['\t','\n','\r',',',':','=','\u{1f}'])` — a NEW helper
  (distinct from `breaks_framing`, which guards the shell ROOT and must NOT reject `,:=`): the cwd is
  embedded in the grid blob (delimiters `,` `:`), gets the new `=` sub-delimiter, sits inside the
  `T=…\x1f…` shell entry, and is `\t`-framed at the shell level — so it must exclude ALL of those. A cwd
  with any → dropped (plain `t`).
- `restore_grid(s)`: build `kinds` + `cwds` in lockstep via a new `parse_leaf(tok) -> Option<(PaneKind,
  Option<String>)>`: `tok.strip_prefix("t=")` → `(Terminal, Some(cwd))`; else `one_char_kind(tok).map(|k|
  (k, None))`; `None` → `default_grid()`. Back-compat: an old `"t"`/`"H:t,f"` → all `None` cwds.
  Never panics (byte-safe: `strip_prefix("t=")` on an ASCII prefix → the rest is a valid `&str`).

**SHIM (app.rs, masked):**
- Capture (`build_shell_layout` @2033 + the persist_grid path @2022): before each
  `serialize_grid(grid.group(), &kinds)`, build `cwds: HashMap<PaneId,String>` from each terminal pane's
  live `session.current_prompt().pwd` (the #201 source; a pane with no prompt yet / non-terminal → absent
  → plain `t`). Pass `&cwds` as the new arg. (Both call sites gain the arg.)
- Restore: hoist `let gl = restore_grid(blob)` to BEFORE the base-session spawn (@839); spawn the base in
  `cwd_or_root(gl.cwds.first(), &root)`; pass `&gl` to `restore_panes` (signature → `|grid, gl:
  &GridLayout, root, zdotdir|`, no longer re-parsing) which spawns each split pane `i` in
  `cwd_or_root(gl.cwds.get(i), &root)`. `cwd_or_root(cwd: Option<&Option<String>>, root) -> PathBuf` =
  the cwd if `Some(Some(c))` + non-empty + `Path::new(c).exists()`, else `root.to_path_buf()` (the IO
  exists-check — the shim, not pure).
- **Boot-PTY-reuse limitation (documented):** the launch-cwd project's FIRST terminal reuses the
  pre-spawned boot PTY (already in the launch dir) — its saved `cwds[0]` is NOT re-honored (a re-cd is
  out of scope; the boot session is already live). Every OTHER grid's base + all split panes honor their
  saved cwd. (The multi-terminal win is intact; only the single launch-first pane stays at the root.)

### File manifest
| File | Change |
|---|---|
| `crates/marley_app/src/grid_layout.rs` | `GridLayout.cwds`; `flatten`/`serialize_grid` gain a `cwds` param + `serialize_leaf`; `restore_grid` + `parse_leaf` build `cwds`; new `breaks_grid_framing`. Tests at validate. |
| `crates/marley_app/src/app.rs` | Build the `cwds` map at the 2 `serialize_grid` call sites (from `current_prompt().pwd`); hoist `restore_grid` + spawn the base + split panes in `cwd_or_root` (the exists-check). Masked shim. |

### Regression Test Plan
Real grid_layout mutant patterns (from `cargo mutants --list -f grid_layout.rs`): body → `Default`/
`String::new()`/`"xyzzy"`/`None`/`Some(Default)`; `<=`→`>`; delete-match-arm; `breaks_framing`→true/false.
The extended fns add: `serialize_leaf` (the `Terminal`+`Some` arm), `parse_leaf` (the `t=` strip),
`breaks_grid_framing` (true/false). Confirm the real set at validate with `--list` on the actual code.

| # | REQ | Test (grid_layout.rs `#[cfg(test)]`) | Kills |
|---|---|---|---|
| T1 | REQ-001 | `restore_grid(serialize_grid(single-terminal, {id→"/tmp/foo"}))` → `cwds == [Some("/tmp/foo")]` | round-trip a cwd; body/leaf mutants |
| T2 | REQ-001 | a split `H:` with cwds `{a→"/x"}` + a plain terminal + a files pane → `["t=/x","t","f"]` → restore `cwds == [Some("/x"), None, None]` | the split join + `serialize_leaf` arm |
| T3 | REQ-002 | a cwd `"/a,b"` (or `/a:b`, `/a=b`, `/a\tb`) → `serialize` emits plain `"t"` → restore `cwds == [None]` | `breaks_grid_framing` true/false |
| T4 | REQ-003 | `restore_grid("t")` → `cwds == [None]`; `restore_grid("H:t,f")` → `[None, None]` (back-compat old blob) | the no-cwd path; `parse_leaf` char arm |
| T5 | REQ-003 | `restore_grid("t=")` → `cwds == [Some("")]` (empty cwd → restore exists-check → root); junk `"zzz"` → default | edge: empty cwd + junk → default, no panic |
| T6 | — | the EXISTING #163 round-trip tests (`serialize_grid`/`restore_grid`/`serialize_shell`) STILL PASS with the new signature (update their call sites: pass an empty `cwds` map). | no regression to #163 |

**Driven (shim, if the machine is UNLOCKED — it was LOCKED at #204 validate; per
`PR-claude-selftest-locked-screen-blocks-capture-fall-back-to-mechanism`, if still locked → units +
mechanism):** open 2 terminals, `cd /tmp` in one, relaunch → each restores in its dir; delete a saved
cwd → root. The restore reuses the shipped `spawn_session_in` idiom.

### Risks / decisions
- **R1 (framing set):** `breaks_grid_framing` must exclude the shell (`\t\n\r`) + grid (`,:=`) + title
  (`\x1f`) delimiters — a superset of `breaks_framing`. A NEW helper (not extending `breaks_framing`,
  which guards the root and legitimately allows `,:=`).
- **R2 (signature change ripples):** `serialize_grid`/`flatten` gain a `cwds` param → the 2 app.rs call
  sites + the existing #163 unit tests update (pass an empty map). Bounded, compiler-guided.
- **R3 (restore flow):** hoist `restore_grid` before the base spawn (honors pane 0's cwd — the common
  single-pane case); `restore_panes` takes `&gl` (parse once). The boot-PTY-reused launch-first pane
  keeps the boot cwd (documented v1 limitation).
- **R4 (clean-room §20):** original codec extension.

## Phase 3 — Implement
**Built (per design, no deviations):**
- `grid_layout.rs` (pure): `GridLayout.cwds: Vec<Option<String>>`; `default_grid` → `cwds: vec![None]`;
  `flatten` gains a `cwds` param + collects `(PaneKind, Option<String>)` (Terminal + framing-safe cwd);
  new `serialize_leaf` (`t=<cwd>` | kind char); `serialize_grid` gains the `cwds` param; new `parse_leaf`
  (`strip_prefix("t=")` | one_char_kind); `restore_grid`/`single_char_grid` build `cwds` in lockstep; new
  `breaks_grid_framing` (`\t\n\r,:=\x1f`). Back-compatible (an old `"t"` blob → `None` cwds).
- `app.rs` (masked shim): `use …::GridLayout`; both `serialize_grid` call sites (`persist_grid` +
  `build_shell_layout`) build a `cwds: HashMap<PaneId,String>` from each terminal's
  `session.current_prompt().pwd`; a new masked `cwd_or_root(cwd, root)` (the `Path::exists` check); the
  restore flow hoists `restore_grid(blob)` once, spawns the base session in `cwd_or_root(gl.cwds.first(),
  root)` and each split pane (in `restore_panes`, now taking `&GridLayout`) in
  `cwd_or_root(gl.cwds.get(i), root)`. Both `restore_panes` callers (shell + legacy boot) updated to pass
  `&gl`. The boot-PTY-reuse launch-first pane is untouched (documented limitation).

**Compile-fixes (design-anticipated):** the `serialize_grid` signature change flagged 4 existing #163 test
calls (`+ &HashMap::new()`) + 1 `GridLayout` literal (`+ cwds: vec![None]`) — fixed, assertions intact.

**Checks:** `cargo fmt` + `cargo check --all-targets -p marley` clean (no warnings; only the upstream
`block v0.1.6` note). `cargo nextest run -p marley` → **310 passed, 2 skipped** — all 11 grid_layout
tests green (the #163 shell round-trip + framing-breaker tests unchanged = no regression). The NEW cwd
round-trip tests (T1–T6) are Phase 4.

## Phase 3.5 — Inspect
2 general-purpose critics (Critic 1: pure codec + mutation + back-compat; Critic 2: the shim restore/
capture + regression) + self-review. Critic 1 verdict SHIP (codec correct/panic-free/framing-complete);
Critic 2 verdict FIX (2 MEDs). Both empirically verified (hand-trace + tests + mutation runs).

| # | Finding | Sev | Verdict | Action |
|---|---|---|---|---|
| F1 | **Launch project's base pane silently ignored its saved cwd** — the boot-PTY reuse (`take_if(root==project_root)`) kept the boot cwd (launch dir), so the primary terminal `cd`'d into a subdir restored to the launch root. The MOST COMMON case (one window, primary cd'd) → the feature no-op'd. I'd mis-designed this as a "documented limitation". | MED | REAL (fixed) | **FIXED** app.rs — gate the reuse on `cwd_or_root(gl.cwds.first(), &root) == root`; a real subdir cwd spawns fresh THERE (boot PTY dropped — minor startup cost). failure `097d6e05`. |
| F2 | **`cwd_or_root` used `exists()` (true for a FILE) where `is_dir()` is correct** — a cwd that became a file → `spawn_session_in` fails → the pane/tab silently dropped (worse than REQ-004's root-fallback). | MED | REAL (fixed) | **FIXED** app.rs:463 → `Path::new(c).is_dir()` (cleanly redirects a now-file/broken cwd to root). |
| F3 | Garbled `flatten` doc — my #205 edit half-overwrote the #122 sentence (a dangling "A leaf whose id is missing from" line). | LOW | REAL (fixed) | **FIXED** grid_layout.rs — rewrote the doc block cleanly. |
| F4 | `'='` in `breaks_grid_framing` is conservative — a cwd with `=` actually round-trips (`parse_leaf` strips only the FIRST `t=`), yet it's dropped → a legit `=`-path restores to root. | LOW | REJECTED (by-design) | No fix — the D2 "drop is always safe" stance; a drop can never misparse; `=`-paths are rare. Documented. |
| F5 | Real cargo-mutants set = **39 listed, 3 unviable** (`GridLayout::default()` — no `Default` derive) **→ 36 viable**. The baseline 11 tests leave **4 missed** (flatten `==→!=`, flatten `delete !`, `breaks_grid_framing`→true/→false). **T1+T3 kill all → MSI 100** (T2 adds the headline multi-pane). And: cargo-mutants generated NO delete-arm mutant for `serialize_leaf`'s `(Terminal,Some)` arm (my design assumed one — it doesn't exist; only whole-body). | MED | REAL (Phase-4 req) | Phase 4 MUST add ≥ T1+T3 (recommend T1+T2+T3); reinforces the trace-the-real-list rule again. |
| F6 | **A critic ran `git checkout grid_layout.rs`** (after adding tests for a mutation run) on the UNCOMMITTED #205 tree — WIPING the codec; it rebuilt + verified byte-accurate 3 ways. A near-miss data-loss. | — | REAL (process) | Codec VERIFIED intact (16 markers + `cargo check` + 11 tests green). Recorded `PR-claude-inspect-critic-must-not-git-checkout-the-working-tree` (id 3fe92aa8) + `PR` to verify Phase-3 intactness after a file-touching critic. |

**Critic-verified CLEAN:** round-trip correct across T1–T5 + adversarial (single-pane carries cwd via
`single_char_grid`; the framing-drop; back-compat old `t`); byte-safe (all `strip_prefix`/`split`/
`contains` — zero indexing; multi-byte cwd no panic); `serialize_leaf`∘`parse_leaf` inverse; the framing
set is complete; a cwd is double-gated to Terminal; capture keys both maps by `pane_ids()` (aligned);
`current_prompt().pwd` is the live cd-tracking source; restore alignment `cwds[i]↔kinds[i]` no off-by-one;
both `restore_panes` callers parse `&gl` (none missed); the #163 shell round-trip + kinds/structure/
actives persistence intact; clean-room.

**Post-fix:** `cargo check -p marley` clean. Phase-4 test req (F5) carried into the plan (T1+T2+T3, from
the real 36-viable-mutant set).

## Phase 4 — Validate
**Test written** (grid_layout.rs `#[cfg(test)]`): `grid_cwd_round_trips_and_drops_framing_breakers`
(T1–T5) — T1 single-terminal cwd round-trips (`t=/tmp/foo`, the common single-pane case); T2 a split's
mixed cwds (`H:t=/x,f` serialize + a 3-leaf `H:t=/x,t,f` restore → `[Some,None,None]`); T3 a
framing-breaking cwd (`/a,b`,`/a:b`,`/a=b`,`/a\tb`) dropped → plain `t`; T4 back-compat (old `t`/`H:t,f`
→ `None` cwds); T5 edges (`t=`→`Some("")`, junk→default). T1+T3 kill the 4 new viable mutants (per
Critic 1's empirical F5).

**Ran (ACTUAL):**
- `cargo nextest run -p marley` → **311 passed, 2 skipped** (+1 new; the 11 existing #163 grid_layout
  tests unchanged = no regression).
- Targeted mutation `cargo mutants -f grid_layout.rs` → **39 mutants, 36 caught, 3 unviable**
  (`GridLayout::default()` — no `Default` derive) = cov/MSI 100 (exactly the F5 set).

**Driven capture — ENV-BLOCKED (documented, not skipped):** a full-screen `screencapture -x` again
showed the **macOS lock screen** ("Fri Jul 10 · Chad Peppers · Enter Password") — the machine is still
locked (chad AFK since ~#204 validate). Per `PR-claude-selftest-locked-screen-blocks-capture-fall-back-
to-mechanism`, the relaunch capture (open 2 terminals, `cd /tmp` in one, relaunch → restores in `/tmp`)
is env-blocked; no password attempted. REQ-001/002/003 are unit-proven (the T1–T5 round-trip covers
persist→serialize→restore of the cwd, the framing-drop, and back-compat). REQ-004 (restore-in-cwd +
not-exists→root) + REQ-005 (live-cwd capture) rest on the critic-traced mechanism: the restore reuses
the shipped, previously-live-proven `spawn_session_in` idiom (with the fixed `cwd_or_root`/`is_dir`
guard), and the capture reuses the #201 `current_prompt().pwd` source — both critics verified the flow
(alignment, no off-by-one, the launch-base gate). Per §7, units + mechanism carry REQ-004/005 given the
hard env block; a live re-verify is filed as an optional follow-up.

**Gate:** `git add -A` (app.rs, grid_layout.rs + the 3 pipeline docs — no secrets) → `scripts/gates.sh
--diff` → **GATE GREEN [diff]**, 15/15 (rustfmt, clippy -D warnings, tests, coverage ≥100%, mutation
MSI ≥100%, miri, audit, deny, machete, gitleaks, shellcheck, no-suppressions, source-bans, docs,
visual/AX [headless]). Receipt written. No pre-existing exclusions.

## Phase 5 — Complete
**Docs (§21):** CHANGELOG.md — #205 under `[Unreleased] ### Added` (below #204). app_shell.md — a #205
note appended to the #163/#122 grid-persistence section (the `t=<cwd>` codec, `breaks_grid_framing`, the
`GridLayout.cwds`, the capture/`cwd_or_root` shim, the boot-PTY-reuse gate).

**Forge capture (§19):**
- `aar-submit` 592d2ba3 — completed, effectiveness 5. Lessons: (a) #205 was MOSTLY pre-built by #163
  (whole-shell persistence) — the genuine delta was the ONE per-terminal cwd tier on the existing codec,
  a bounded slice (no split despite the ticket flagging one); (b) extending a delimited wire format =
  reuse the OPTIONAL-extension precedent (#177 `T=title\x1fblob`) for back-compat + a framing-safe-or-drop
  check (a NEW `breaks_grid_framing` for the cwd's stricter delimiter set, distinct from the root's
  `breaks_framing`); (c) a codec signature change (+`cwds` param) ripples to the call sites + existing
  tests — `cargo check --all-targets` is compiler-guided (fix the test literals with an empty map); (d)
  the boot-PTY-reuse optimization silently ignored the saved cwd for the COMMON launch-primary-terminal
  case — a design "documented limitation" the inspect critic correctly reframed as a bug (gate the reuse
  on the cwd resolving to root).
- Recorded at INSPECT (referenced, not duplicated): `failure-record`
  **BF-claude-session-cwd-boot-pty-reuse-ignores-saved-cwd-001** (097d6e05, the launch-base no-op + the
  `exists`-vs-`is_dir` pane-drop) + `prevention-rule`
  **PR-claude-inspect-critic-must-not-git-checkout-the-working-tree-001** (3fe92aa8, HIGH — a critic
  `git checkout` on the uncommitted tree wiped the codec; verify Phase-3 intactness after a file-touching
  critic).

**Env-blocked note:** the live relaunch-in-cwd capture was blocked (machine locked); an optional 30-second
re-verify (open 2 terminals, `cd` one, relaunch → restores in-cwd) is worth doing once unlocked — no
separate ticket (units + the critic-traced mechanism carried REQ-004/005).

**Close + archive:** forge ticket #205 → done. Local TICKET-205 → closed/. Pipeline doc pair →
completed/. Spec status → Phase 5 — Complete PASS.
