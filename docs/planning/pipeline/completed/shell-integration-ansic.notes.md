# shell-integration AnsiCQuoted emit — Notes

- **Forge ticket:** #22 `02632a5b-cf30-48c7-a008-2cc1c0b9c5d7`
- **AAR:** `c88f9a9c-44b8-4020-8c02-b6a49c013244`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-022-ansic-emit.md
- **Pipeline spec:** shell-integration-ansic.spec.md

<!-- Working scratch for the pipeline. Each phase appends its entry. -->

## Phase 1 — Plan
- **Request:** forge #22 (M1.C Wired Cockpit seq-1, auto-approved run) — the M1.A shell
  integration emits Plain DCS frames; `;` in a command/pwd truncates the Block command and grows
  phantom fields. Switch the emit to AnsiCQuoted.
- **Classification / tier:** work pipeline, `bug` — one shippable slice (rc text + tests + spec
  clause). No UI surface; gate-15 rides existing baselines.
- **Forge recall (§18.3):** bulletins none; knowledge-search returned only opaque AD ids;
  docs-search cross-project noise. Load-bearing priors from the M1.A/M1.B corpus: the real-zsh
  `#[serial]` integration test precedent; write-first-not-research; `--diff` gate mode is
  commit-valid for a single-crate change; §21 CHANGELOG.
- **Discovery (current state, verified in-tree):**
  - Emit: `crates/marley_app/src/shell_integration.rs:19` — `marley_zsh_init()` emits
    `\ePp…\e\\` (Plain, selector `p`) frames: `init;id=1`, `bootstrapped;subshell=0`,
    `preexec;command=%s` ($1 raw), `precmd;exit=%d;pwd=%s` ($PWD raw).
  - Decode: `crates/terminal_blocks/src/dcs.rs` — `encoding_for_dcs_terminator`: `h`/`p`/`q`;
    `decode_hook(AnsiCQuoted, …)` → `c_unescape` supporting exactly `\n \t \r \\ \; \xHH`.
    The `name;key=value` split uses `split_once('=')` per part → a `=` INSIDE a value survives;
    the real corruption is `;` (field split) + raw ESC (scanner). Phantom fields arise from
    `;`-containing values.
  - Proof harness: `crates/marley_app/tests/integration.rs` — `#[serial]` real-zsh test asserts
    only output text today; `Block.command: String` + `prompt.pwd: Option<String>` are public →
    exact round-trip assertions are directly writable.
  - Spec gap: SPEC-app-shell has NO emit-side clause (SPEC-terminal-blocks owns decode only) →
    the emit contract gets its EARS clause(s) this pipeline (D3).
- **Decisions:** D1–D5 in the spec (emit-side only; in-rc zsh substitution chain; SPEC-app-shell
  amendment; exact round-trip proof; keep `$1`).
- **Open questions for Design:** uniform `q` for the static init/bootstrapped frames vs mixed
  selectors; whether the additive `ansi_c_quote` encode helper earns its place in `dcs.rs`
  (test-support value) or tests hand-compute expected payloads; zsh substitution ORDER (backslash
  first) + which control bytes beyond ESC/`\n`/`\t`/`\r` get chains; the torture-string vector.
- **AAR id:** `c88f9a9c-44b8-4020-8c02-b6a49c013244` (inspect→failure-record, complete→aar-submit).

## Phase 2 — Design

### Architecture / approach
Emit-side only, inside `marley_app` (the rc is marley_app's artifact; `marley_terminal` owns the
codec and is NOT touched — see D-2.3). The rc's two dynamic frames switch selector `p` → `q`
(AnsiCQuoted) and pass their values through an **in-function zsh parameter-substitution chain**
(no forks, no external commands in the hot path):

```zsh
__marley_preexec() {
  local c=$1
  c=${c//\\/\\\\}      # backslash FIRST (order load-bearing)
  c=${c//;/\\;}
  c=${c//$'\n'/\\n}
  c=${c//$'\t'/\\t}
  c=${c//$'\r'/\\r}
  c=${c//$'\e'/\\x1b}
  printf '\ePqpreexec;command=%s\e\\' "$c"
}
__marley_precmd() {
  local ec=$? p=$PWD    # capture $? before ANY expansion
  p=${p//\\/\\\\} … (same 6-step chain) …
  printf '\ePqprecmd;exit=%d;pwd=%s\e\\' "$ec" "$p"
}
```

The escape set is exactly the dangerous bytes under the shipped `c_unescape` grammar: `\` (would
misread the next byte as an escape → `UndecodablePayload`), `;` (field split), ESC (DCS scan), plus
`\n`/`\t`/`\r` for grammar-parity hygiene (raw forms survive but escaped forms round-trip
identically). Other control bytes pass raw — tolerated by scanner+codec+UTF-8. `%` in values is
safe (printf `%s` argument, not format). Static frames `init;id=1` / `bootstrapped;subshell=0`
switch to `q` too (**uniform-q**): payloads are escape-free so decode is byte-identical, and the
unit test can assert "zero `\ePp` frames remain".

**Decisions**
- D-2.1 Uniform `q` selector for all four frames (one mental model; a stronger unit assertion).
- D-2.2 Escape set = `\` `;` ESC `\n` `\t` `\r`; substitution order backslash-first.
- D-2.3 **No Rust `ansi_c_quote` helper**: the escaping runs in zsh at runtime; a Rust twin would
  prove the Rust fn, not the shell chain — the real-zsh integration test is the proof. Keeps
  `marley_terminal` fully untouched (strongest REQ-004).
- D-2.4 **Probe-then-encode**: zsh `${var//pat/repl}` backslash semantics in the replacement text
  are subtle. Implement runs a one-shot probe (`zsh -c 's="a;b\\c"; s=${s//\\/\\\\}; s=${s//;/\\;};
  printf %s "$s"` → expect `a\;b\\c`, adjusting escapes until exact) BEFORE encoding the chain into
  the rc; the rc-text unit tests then pin the PROVEN syntax.
- D-2.5 SPEC-app-shell gains R25 (AnsiCQuoted emit escaping, exact round-trip) + R26 (every frame's
  selector maps to a supported encoding and decodes clean) + AC rows + Test-Plan entries + a
  Mutation-Targets line (`marley_zsh_init` whole-fn `""`/`"xyzzy"` replacements are killed by the
  selector/chain containment tests). `visual_acceptance` untouched.
- D-2.6 pwd proof: precmd stages the prompt for the NEXT block (apply.rs R6), so the test `cd`s
  into a `;`-named dir and asserts the FOLLOWING block's `prompt.pwd` (ends_with — macOS
  `/var`→`/private/var` symlink forbids full-path equality).

### File manifest
- M `crates/marley_app/src/shell_integration.rs` — uniform-`q` frames; the 6-step escape chain in
  both hooks; module-doc limitation note → the AnsiCQuoted contract; unit tests re-pinned
  (selector, chain presence + order, zero `\ePp`).
- M `crates/marley_app/tests/integration.rs` — ADD `#[serial]` `torture_command_round_trips_exactly`
  (existing test untouched): tempdir cwd; run `echo 'a=b; c' "d=e;f" g\h` → exact `block.command`;
  `mkdir` + `cd 'weird;dir=x'`; run `echo done` → its block's `prompt.pwd` ends_with `weird;dir=x`;
  the `cd` block's command also round-trips (second `;`/`=` vector).
- M `docs/specs/SPEC-app-shell.spec.md` — R25/R26 + AC rows 25/26 + Test Plan + Mutation Targets.
- M `CHANGELOG.md` — Fixed entry (staged with the commit, §21).
- M `docs/marley_architecture/app_shell.md` — limitation paragraph updated (complete phase).
- NOT touched: `crates/terminal_blocks/**` (REQ-004), `app.rs`, baselines (gate-15 rides).

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `rc_uses_ansic_quoted_selector_uniformly` — rc contains the 4 `\ePq…` frames and zero `\ePp`; kills the whole-fn `""`/`"xyzzy"` mutants | unit (rc text) |
| REQ-001 | `rc_escape_chain_is_complete_and_backslash_first` — all 6 substitutions present for BOTH `$1` and `$PWD`; backslash substitution index < each other substitution's index | unit (rc text) |
| REQ-002 | `torture_command_round_trips_exactly` — exact `block.command` equality for a `;`/`=`/quotes/spaces/backslash command; `prompt.pwd` ends_with the `;`-named dir on the following block; no phantom truncation | integration, `#[serial]` real zsh |
| REQ-003 | `init_contains_the_dcs_hook_frames` (re-pinned) — `init;id=1` + `bootstrapped;subshell=0` present under `q` | unit (rc text) |
| REQ-003 | `echo_command_produces_a_block_with_its_output` — pre-existing, UNMODIFIED, stays green | integration |
| REQ-004 | diff review: zero changes under `crates/terminal_blocks/`; full workspace suite green | review + suite |
| REQ-005 | `scripts/gates.sh --diff` GREEN (cov 100 / MSI 100 on touched) + commit receipt | gate |

Uncoverable paths: none new (no shim change; `app.rs` untouched). The integration tests depend on
`/bin/zsh` existing — same precondition the shipped M1.A test already establishes.

### Risks
- zsh replacement-text backslash semantics (mitigated by D-2.4 probe + the integration test).
- Multi-line commands: preexec `$1` carries raw newlines → chain escapes to `\n` → decoder restores
  → exact round-trip (covered by the chain; not separately tested in M1.C).
- `local ec=$? p=$PWD` must capture `$?` first — pinned by a unit substring assertion on that line.
- REQ-004 wording: "pre-existing tests unmodified" applies to `marley_terminal` (decode) and the
  existing integration test; the rc-text unit tests in `shell_integration.rs` are the changed
  surface and are legitimately re-pinned.

## Phase 3 — Implement
- **Built:**
  - `crates/marley_app/src/shell_integration.rs` — the rc rewritten as one multi-line Rust raw
    string (zero double-escaping hazards vs the old `concat!`): uniform `q` selector on all four
    frames; a single `__marley_quote` zsh function (6 substitutions, backslash first) writing
    `__MARLEY_REPLY`; `__marley_preexec` quotes `$1`, `__marley_precmd` captures `local ec=$?`
    BEFORE calling the quote fn (a fn call resets `$?`), then quotes `$PWD`. Module + fn docs
    re-written to the new contract (R25/R26 cited).
  - `docs/specs/SPEC-app-shell.spec.md` — R25 (AnsiCQuoted escaping, backslash-first, exact
    round-trip) + R26 (uniform supported selector, clean decode); AC rows 25/26; Test-Plan
    entries (2 unit + 1 integration); Mutation-Targets line (`marley_zsh_init` whole-fn
    replacements).
  - `CHANGELOG.md` — new `### Fixed` section under [Unreleased] with the TICKET-022 entry (§21).
- **Deviations from design (with reason):**
  - D-3.1 — ONE `__marley_quote` helper using the `__MARLEY_REPLY` no-fork idiom instead of the
    design sketch's inline chain duplicated in both hooks: single source of truth (no drift
    between the two copies), still fork-free; `__MARLEY_REPLY` (not zsh's `REPLY`) avoids
    clobbering user widgets' `REPLY`. The R25 unit test asserts both hooks route through the fn.
  - D-3.2 — rc authored as a Rust raw string rather than `concat!` lines: byte-identical output,
    removes the `\\\\\\\\`-class escaping hazard the probe was guarding against.
- **Verification at this phase:** D-2.4 probe PASSED (chain output `a\;b\\c\nd\te\rf\x1bg`; frame
  `ESC P q … ESC \`); `cargo check --workspace` green; `cargo fmt --check` clean; `zsh -n` on the
  extracted rc OK; a live `zsh -i` emits the `q` init/bootstrapped frames. The two existing
  rc-frame unit tests still compile; the selector assertions are re-pinned (Ppinit→Pqinit) — the
  full planned test additions (R25 chain test, R26 uniformity test, the torture integration test)
  are Phase 4 per the test plan.

## Phase 3.5 — Inspect
- **Critics run:** 2 parallel general-purpose critics — (A) zsh-semantics prober (100 quote/frame
  probes + 15 real-decoder cross-checks + 12 interactive-zsh probes, 2 locales, against the
  extracted rc AND the real `decode_hook` via a scratch path-dep crate); (B) diff reviewer
  (AC/security/provenance/state-integrity/simplification, verified against the real decoder too).
- **Findings table:**
  | # | Sev | Finding | Verdict | Fix |
  |---|---|---|---|---|
  | F1 | HIGH | The headline `;` fix CANNOT work emit-side alone: `decode_hook` un-escapes the whole payload BEFORE the `;`/`=` split (dcs.rs), so the rc's `\;` collapses to a bare `;` pre-split — `ls; pwd` still truncates to `ls`; a `;`-named dir injects phantom fields (`…\;git=evil` → `git_branch: Some("evil")`). Both critics proved it against the REAL decoder. | REAL (design premise D1 refuted) | R24 decode-order fix in `dcs.rs`: `split_unescaped`/`find_unescaped` raw-field scanners (backslash-consumes-next), split first, `c_unescape` per piece; Hex/Plain byte-identical. Verified: the 4 critic payloads now round-trip (`ls; pwd` ✓, phantom-injection dead ✓, `a\\;` boundary ✓, `\x3b` ✓); all 69 pre-existing marley_terminal tests pass with assertions unmodified. Pipeline spec D1/REQ-004/Scope amended. |
  | F2 | MED | SPEC-app-shell R25's tail ("decoded fields equal the original exactly") was unsatisfiable under the old premise. | REAL | Resolved by landing F1's decode fix in this pipeline — R25 now holds end to end. |
  | F3 | MED | Invalid-UTF-8 command bytes (raw paste) → `UndecodablePayload` → hook dropped. | REAL but OUT OF SCOPE (pre-existing decoder property, not introduced here; commands from `write_command` are always UTF-8) | Documented: dcs.rs module doc + R26 reworded to scope the no-decode-error guarantee to UTF-8 values. Backlog candidate for M2 shell-integration hardening. |
  | F4 | LOW | Raw C0 bytes (BEL/CAN) transit un-escaped; Marley's scanner tolerates them but standards-strict DCS parsers (tmux/ssh transit) would abort the frame. | REAL, future hardening | Deferred (M2 remote/multiplexer scope — noted; Marley-local pipeline is safe, verified). |
  | F5 | LOW | CHANGELOG overclaimed `=`/quotes as newly fixed (`=` in a value always survived `split_once('=')`). | REAL | Entry reworded: claims `;`/`\`/ESC + phantom-field injection; explicitly notes `=` was never at risk; now covers both crates. |
  | F6 | LOW | R26 absolute wording falsified by non-UTF-8 input. | REAL | R26 scoped to UTF-8 values with the limitation noted. |
- **Rejected findings:** none — every raised finding was empirically verified by the critics
  (their probes ran the real decoder); no false positives to reject.
- **Confirmed-clean lenses:** REQ-001 mechanics (q selector ×4, zero `\ePp`, chain order, zsh
  replacement semantics incl. the `\;`→`\\\;` compound); REQ-003 statics decode; re-pinned unit
  assertions match the rc exactly; provenance (structurally unlike iTerm2 OSC 133/1337, VSCode
  OSC 633, Warp; Marley-original DCS grammar); no secrets; no spawn-input change; fn-doc claims
  true (`$?` capture order verified live); decoder↔rc field contract aligned (`exit`/`pwd`).
- **Post-fix verification:** `cargo fmt --check` clean; `cargo clippy -p marley_terminal
  --all-targets -- -D warnings` clean; marley_terminal 64 unit + 5 integration pass; marley lib
  29 pass; the 4 adversarial payloads round-trip via the probe crate (P1–P4 OK).

## Phase 4 — Validate
- **Tests added (per the Phase 2 plan + inspect additions):**
  - `marley_app/src/shell_integration.rs` — `rc_uses_ansic_quoted_selector_uniformly` (R26: 4×
    `\ePq`, zero `\ePp`; kills the whole-fn mutants) + `rc_escape_chain_is_complete_and_backslash_first`
    (R25: 6 substitutions, backslash-first ordering by index, both hooks route through
    `__marley_quote`, `$?` captured pre-call).
  - `marley_app/tests/integration.rs` — `torture_command_round_trips_exactly` (`#[serial]` real
    zsh): `echo 'a=b; c' "d=e;f" g\h` → EXACT `Block.command`; `cd 'weird;dir=x'` command
    round-trips; the following block's `prompt.pwd` ends with the `;`-named dir.
  - `terminal_blocks/src/dcs.rs` — `decode_ansic_escaped_separators_stay_in_value` (R24: `\;`,
    `\x3b`, pwd phantom-injection dead, `a\\;` boundary, `\x3d`) +
    `decode_ansic_field_grammar_edges` (separator-less segment skipped, duplicate-key first-wins,
    empty payload → UnknownHook, bad escape in a KEY → UndecodablePayload, non-UTF-8 in
    name/key/value → UndecodablePayload, Plain legacy split pinned) +
    `split_unescaped_boundaries` + `find_unescaped_positions` (the scanner mutant-killers).
- **Runs (actual):** `cargo nextest run --workspace` → **438 passed, 5 skipped** (all 7 new tests
  listed PASS); `cargo test --workspace --doc` → green (no doctests, 0 failures).
- **Gate journey (fix-at-source, §0):**
  1. Run 1 RED (12/15): gate:4 coverage, gate:7 audit, gate:8 deny.
  2. gate:7/8 — TWO NEW RUSTSEC advisories published 2026-06-29 (mid-sprint): RUSTSEC-2026-0194 +
     -0195 on quick-xml 0.30/0.39, pulled ONLY by wayland-scanner/xcb (Linux-only gpui windowing,
     compiled out of the shipped macOS target — `cargo tree -i quick-xml` empty on host; bundled
     protocol XML, no untrusted input; fix needs quick-xml ≥0.41 upstream). Per-id ignores with
     justification: deny.toml + NEW `.cargo/audit.toml` (vulnerability-class needs the audit-side
     ignore too) — the TICKET-007 chad-approved pattern. CHANGELOG `### Security` entry.
  3. gate:4 — the three AnsiCQuoted `String::from_utf8` error arms were uncovered → added the
     non-UTF-8 name/key/value asserts. Run 2 RED (14/15): ONE line left —
     `shell_integration.rs` 98.65%: the `unwrap_or_else(|| panic!())` closure in the NEW test
     never executes (a dead test closure = a missed function+line) → replaced with
     `assert!(contains) + expect`. (Learning: my interim standalone coverage "pass" was a false
     read — piping through `grep|head` masked the exit code; only the gate log is truth, §15.)
  4. Run 3 **GREEN [diff]: 15/15**. Coverage 100% lines on the testable surface; mutation
     **35 caught / 0 missed → MSI 100.0%** (the diff-scoped mutants over dcs.rs +
     shell_integration.rs); receipt written (`.git/ignibyte-gate-receipt`
     `0e8b996f4b0c4802a208a2be9ba64b7dd928b409`).
- **Visual (gate:15):** PASS — no new UI surface; rides the existing `shell_dark` baseline +
  headless harness suite (per the plan, `visual_acceptance: N/A` for this change).
- **Pre-existing failures:** none. (dcs.rs/session.rs REGION misses at 100% LINE coverage are the
  pre-existing measurement shape, not regressions — the line gate is the floor.)

## Phase 5 — Complete
- **Docs (§21):** CHANGELOG `### Fixed` (the two-halves entry) + `### Security` (the quick-xml
  per-id ignores) staged; `docs/marley_architecture/app_shell.md` shell-integration section
  rewritten to the AnsiCQuoted contract (R25/R26, remaining non-UTF-8 limitation);
  `docs/marley_architecture/terminal_blocks.md` dcs.rs bullet gains the R24 split-before-unescape
  record + the BF code. SPEC-app-shell R25/R26 + SPEC-terminal-blocks R24 landed at
  implement/inspect.
- **AAR capture:** failure `BF-claude-decoder-unescape-before-split-nullifies-emit-escaping-001`
  (HIGH, validation) + prevention rule
  `PR-claude-verify-decoder-parse-order-before-scoping-encode-fix-001` recorded at inspect;
  aar-submit with outcome `completed`. Lessons: (1) the inspect critics running the REAL decoder
  (not reviewing the diff) is what caught the premise-refuting bug — keep instructing critics to
  execute, not read; (2) piping a gate command through `grep|head` masks its exit code — only the
  gate's own log/exit is truth (§15); (3) a `unwrap_or_else(|| panic!)` in TEST code is a dead
  closure under a 100%-line floor — prefer `assert!` + `expect`; (4) two RUSTSEC advisories
  published MID-SPRINT hit gate:7/8 — vulnerability-class ignores need `.cargo/audit.toml` in
  addition to deny.toml (new repo fact).
- **Ticket:** forge #22 → done; local doc → `docs/planning/tickets/closed/TICKET-022-ansic-emit.md`
  (status closed). Pipeline pair archived to `docs/planning/pipeline/completed/`.
