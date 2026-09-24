# E2E visualization tests replace unit tests — Notes

- **Local ticket doc:** docs/planning/tickets/open/TICKET-483-e2e-visualization-tests.md
- **Pipeline spec:** 483-e2e-visualization-tests.spec.md

## Phase 1 — Plan (2026-09-23)
- **Request:** Chad, mid-#480: "NEW UPDATE. We are removing unit tests from the workflow
  entirely with instead doing e2e visualization tests only. Please update the workflow
  accordingly. No more unit tests". Asked about the tests in the tree, he chose "Keep, stop
  running" (they stay, clippy still builds them, no gate runs them); asked what the e2e test
  is, "Scripted live captures". Then: "Rustal is changing its code now as well" (rustal is
  making the same move; nothing of rustal's is touched here).
- **Classification:** chore; a gate-is-test change plus a new tool, so its verification is
  negative smokes and the runner's own e2e run (§7).
- **Recall.** The brain (consultation c574e6c7d8584198a1f3b9dde6152597) returned only unrelated
  follow-ups. The ledger: L-claude-467 (shoot one window by its toplevel), L-claude-437 and
  L-claude-438 (the headless drive; no input into Chad's session). rustal still gates on unit
  tests and Playwright; there is no precedent to mirror yet.
- **Discovery.** Test runs live in `script/gates.sh` (gate:3 `tests_g`, gate:4 `rust_cov`,
  gate:6 `miri_g`, gate:19 `empty_suites_g`, the floor knobs, the mutation-mask half of gate:12),
  `script/mutation.sh`, `justfile` (`test`, `gate-full`), `enforce-tests-ran.sh`, CONSTITUTION
  §0/§3/§7, `.claude/commands/pipeline/{plan,code,test}.md`, `.claude/commands/spec.md`, both
  pipeline templates, `docs/marley/README.md`, `docs/marley/three-prong-plan.md`,
  `docs/marley/zed-touchpoints.md` (the owned-path prose) and `vendor/README.md`.
  `lib-hook-helpers.sh` holds the owned-path set (`marley_owned_path`) and the receipt's
  fingerprint (`gate_state_hash`).

### Design
- **The gate.** The four steps and their helpers go; `--diff` and `--fast` stay (the receipt or
  none) and `--full` is refused with a message, since its meaning, the heavy gates over every
  Marley crate, is gone. gate:12 keeps the suppression checks and drops the mutation mask.
  gate:11 shellchecks `script/e2e.sh` and `script/e2e/*.sh` in place of the two retired scripts.
- **The runner, `script/e2e.sh <scenario>`**, from `live-shot.sh`'s launch and cleanup: refuses
  when a Marley window is open; copies the profile; makes a scratch folder; sources the
  scenario; runs its `setup` (E2E_PROFILE, E2E_WORK; it may set OPEN and prepend to PATH);
  launches; waits for the window; runs `steps`; kills Marley, reloads Hyprland's config (the
  window rule goes) and removes the copy and the folder. Steps: `settle <s>`,
  `press <mods> <key>` (down, 50 ms, up, to `address:<Marley>`), `type_text <text>` (a key per
  character, shifted where needed) and `shot <name>` (`grim -T <stableId>` into SHOT_DIR, the
  path printed). It prints whether the user's active window and workspace moved during the run.
- **`script/e2e/shot.sh`** is the one-shot scenario `just shot` runs (NAME, SEED, SETTLE),
  replacing `live-shot.sh`.
- **The hook** counts `script/e2e.sh`, `just e2e` and `just shot` as the Test phase's run.
- **Files:** `script/gates.sh`, `script/e2e.sh`, `script/e2e/shot.sh`,
  `script/e2e/483-e2e-runner.sh`, `script/mutation.sh` and `script/live-shot.sh` (removed),
  `justfile`, `.claude/hooks/enforce-tests-ran.sh`, `.claude/hooks/enforce-quality.sh` (a
  comment), `.claude/hooks/enforce-commit-gate.sh` (a comment), `.claude/hooks/lib-hook-helpers.sh`,
  `CONSTITUTION.md`, `.claude/commands/pipeline/*.md`, `.claude/commands/spec.md`, the two
  templates, `docs/marley/README.md`, `docs/marley/three-prong-plan.md`,
  `docs/marley/zed-touchpoints.md`, `vendor/README.md`. No application code.

### E2E plan
| REQ | Proof |
|---|---|
| 001 | negative smokes: a test changed to fail leaves `just gate-fast` green; a test that does not compile turns gate:2 red; the summary lists no gate:3, 4, 6 or 19 |
| 002 | the runner's scenario: three PNGs in SHOT_DIR, no profile copy or scratch folder left |
| 003 | the same run: shot 2 without the trust prompt, shot 3 with the echoed text; the focus report says unchanged |
| 004 | the hook on a synthetic `/pipeline:test` transcript: blocked without an e2e run, allowed with one |
| 005 | review of §7 and the commands |
| 006 | `just gate-fast` green |

### Risks
- Hyprland may hand the keyboard to the target window for the instant of a sent key, so a key
  Chad types at that moment could land in Marley; the prototype found his active window and
  workspace unchanged afterwards. Accepted with the choice.
- `hyprctl reload` at cleanup reloads Chad's config, as `live-shot.sh` always did.

## Phase 2 — Code (2026-09-23)
- **Built.** `script/gates.sh` without gate:3, 4, 6 and 19, the coverage floor and its
  excludes, the mutation mask of gate:12 and `--full` (refused, with the reason); gate:2 also
  `cargo check --locked --all-targets` each `vendor/` copy, so the tests in the tree, those
  copies' included, keep building. `script/e2e.sh` (the runner: `setup` with `open_path` and
  `terminal_env`, `steps` with `settle`, `press`, `type_text` and `shot`, the focus report),
  `script/e2e/shot.sh` (the one-shot scenario, `just shot`), `script/e2e/483-e2e-runner.sh`.
  `script/mutation.sh` and `script/live-shot.sh` removed. `enforce-tests-ran.sh` counts an e2e
  run; the commit hook, `enforce-quality.sh` and `lib-hook-helpers.sh` lose the FULL and floor
  wording, the owned set gains `script/e2e.sh` and `script/e2e/`, and the fingerprint takes
  them in place of the nextest config. CONSTITUTION §0, §3, §7 (now "E2E Visualization
  Testing"), §14 and §15, and the amendment record; the four phase commands, `/spec`, both
  templates, the justfile, `docs/marley/README.md`, the plan's Gates bullet, the ledger's owned
  set and `.gitignore` row (its `mutants.out` lines went with the mutation run), and
  `vendor/README.md`. The workflow memory note follows the change.
- **Deviations.** `terminal_env` and `open_path` came from the runner's first runs: a
  scenario's typing depended on the user's shell (TICKET-485, below), and shellcheck reads a
  scenario's `OPEN=` as unused. gate:2 builds the vendor copies because §7 keeps their tests
  in the tree too.
- **Found on the way.** With the user's starship prompt on a 140-column path, text typed after
  the first character never shows on the line though bash gets it (Ctrl-L redraws it right);
  with a plain prompt it shows. Filed as TICKET-485 with the evidence.
- **Review.** The receipt still binds the gate-defining files, now the runner too; `--fast`
  still writes none. The runner removes its profile copy and scratch folder on every exit
  (the trap is set before `setup` runs), refuses a second Marley, and names the window by
  address for every key.

## Phase 3 — Test (2026-09-23)
- **E2E** (`SHOT_DIR=<scratchpad>/e2e script/e2e.sh script/e2e/483-e2e-runner.sh`), REQ-002 and
  REQ-003:
  - `483-01-trust-prompt`: Marley on the scratch repository, the "Unrecognized Project" prompt
    over a Restricted Mode window, the scenario's `$ ` prompt at the terminal's bottom.
  - `483-02-trusted`: after `press "" Return`, no prompt and no "Restricted Mode" in the title
    bar; the branch `e2e-runner` shows in the title bar.
  - `483-03-typed`: `$ echo 'Marley e2e: A-Z ok!'`, its output `Marley e2e: A-Z ok!`, a new
    prompt, and the finished block's gutter bar: capitals, digits, quotes, `:`, `-` and `!`
    all typed.
  - Focus report: "the user's window and workspace are as they were". After the run the
    shot directory held the three PNGs and the log, and no profile copy or scratch folder.
- **Probes on the way**, with the user's starship prompt (not the scenario's): only the first
  typed character showed while bash got every key (the command ran; Ctrl-L redrew the line
  right); `stty size` 65 × 321 matched the grid; a plain `$ ` prompt echoed every key. Filed as
  TICKET-485. The probes' scenarios stayed in the scratchpad.
- **REQ-001.** The gate's cargo commands are audit, check, clippy, deny, doc, dylint, fmt,
  metadata, shear and sort: none runs a test (`cargo doc` runs no doctests). A test made not to
  compile (`let broken: u32 = "…"` in `marley_dcs`'s `scan`) made `just clippy marley_dcs` exit
  101 with E0308, which gate:2 runs; the file was restored by sha256.
- **REQ-004.** `enforce-tests-ran.sh` on synthetic `/pipeline:test` transcripts: `cargo nextest
  run` and `just gate-diff` alone, a quoted `echo "script/e2e.sh …"` and `script/e2e.sh --help`
  block (exit 2); `SHOT_DIR=… timeout 200 script/e2e.sh …`, `cd … && just e2e …` and
  `OPEN=… just shot …` pass (exit 0).
- **REQ-005.** Review: §0, §3, §7, §14 and §15, the four phase commands, `/spec` and both
  templates now name the scenario and its shots; no unit or driven test is asked for anywhere.
- **REQ-006.** `just gate-fast`: the first run was red on gate:1 and gate:2 in #480's parked,
  uncommitted code (rustfmt in its tests; clippy's `missing_const_for_fn`,
  `too_long_first_doc_paragraph` and `needless_pass_by_ref_mut`), which is not #483's. With that
  code stashed (restored after #483's commits), `GATE GREEN [fast]`, 15 gates, gate:2 checking
  the vendored `alacritty_terminal` with its tests.

## Phase 4 — Complete (2026-09-23)
- **Docs.** CHANGELOG (Changed: every change proven by an e2e visualization test; Removed: the
  test gates, `--full`, `script/mutation.sh`, `script/live-shot.sh`); CONSTITUTION §0, §3, §7,
  §14, §15 and the amendment record; the phase commands, `/spec`, the templates, the justfile;
  `docs/marley/README.md`, the plan's Gates bullet, the ledger's owned set and its `.gitignore`
  row; `vendor/README.md`. The workflow memory note.
- **Knowledge.** AD-claude-483-e2e-visualization-tests-replace-unit-tests-001,
  L-claude-483-send-keys-to-one-hyprland-window-by-address-001,
  L-claude-483-a-scenario-brings-its-own-shell-001. The starship echo is TICKET-485, whose
  pipeline records its failure block once it is understood.
- **Tickets.** TICKET-484 (autosuggestions, Chad's request during this pipeline) and
  TICKET-485 queued; TICKET-475 moved to Deliberate (moot while no gate runs the tests);
  TICKET-480 parked in `queued/`, back on top of the Queue.
- **Brain.** Consultation c574e6c7d8584198a1f3b9dde6152597 closed with
  `marley-proves-every-change-with-e2e-visualization-tests`, follow-up by 2026-10-07.
- **Commits.** Two, as the amendment rule asks: the constitution with the gate and the hooks
  that enforce it, then the rest.
