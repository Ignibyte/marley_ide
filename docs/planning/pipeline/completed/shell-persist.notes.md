# M10 — persist the workspace shell — Notes

- **Forge ticket:** #163 `657b8a5d-eee0-4c0e-a04b-c43147c33727` · **AAR:** `f2a6ffe7-388f-4632-b484-39b429f4a96d`

## Phase 1 — Plan / Phase 2 — Design (folded)
- Line/tab codec embedding the existing grid blobs; workspace.shell setting; persist piggybacked on
  persist_grid; the boot restores the full shape (guards: missing root/file skip, empty-project terminal,
  active clamps, fresh #167 blocks per grid) with the legacy single-grid fallback.
- **AAR id:** `f2a6ffe7-388f-4632-b484-39b429f4a96d`.

## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 3 — Implement
- grid_layout.rs: TabLayout/ProjectLayout/ShellLayout + serialize_shell/restore_shell (line-1 active idx; root\tactive\tT=blob|C=key|V=path per project; malformed skipped; C= reuses right_section_key/from_key).
- settings.rs: ShellLayoutSetting "" @ workspace.shell; AppliedSettings.shell; persist_shell; defaults + applied_from + 3 test constructors.
- app.rs: the boot restructured — restore_panes closure shared by both paths; a saved shell rebuilds every project/tab (missing roots skipped; the pre-spawned boot PTY seeds the FIRST restored terminal on block 0, later grids spawn_session_in(root) on fresh #167 blocks; cockpit tabs by section; code tabs re-read via the #154 guards; an emptied project gets terminal 1 or is skipped if even zsh fails; actives clamped; the Files tree re-synced to the restored active root); the legacy single-grid path unchanged as the fallback. build_shell_layout + the persist_grid piggyback (every persist site now saves the shell); ALL 6 switch sites (⌘]/⌘[/⌘N chords + the 3 rail click arms) now persist actives.
- fmt; 0 err; clippy OK.

## Inspect (Phase 3.5)
Method: 2 background critics (restore lifecycle; codec/round-trip integrity — incl. an offline toml-1.1.2
round-trip proof).

- **[HIGH → FIXED] the pane_blocks FIELD ignored the boot-restored blocks** (hard `0` in the constructor
  while the restore consumed 1..N) — the first post-boot ⌘T would re-mint block 1 = a restored grid's block
  → the #158 aliasing RETURNS + exact-cleanup strips the wrong tab's agents. Fixed: `pane_blocks,` seeds
  from the boot counter. (Caught by me mid-review from the critic charge; the critic confirmed with the
  full damage trace.)
- **[medium → FIXED] D2 was a lie:** a \t/\n in a root/path was silently MISPARSED (a \t root truncates
  into a wrong-but-plausible project; a \n path FORGES a project line — even a relative root like `src`).
  Fixed at WRITE time: serialize_shell drops projects with framing-breaking roots + code tabs with such
  paths (breaks_framing: \t \n \r); doc + a codec test (shell_codec_drops_framing_breakers_on_serialize).
- **[low → FIXED] REQ-002 was untested** — settings_round_trip_survives_reload now persists + reloads a
  \n/\t-bearing shell blob byte-exact (kills the persist_shell delete-mutant; the critic proved toml 1.1.2
  escapes correctly).
- **[low → FIXED] the boot-PTY cwd straddle** — the first restored terminal reused the launch-cwd PTY even
  for a DIFFERENT project root (one grid, two cwds). Fixed: `take_if(root == project_root)` + an off-thread
  reap of the unused session when no restored project matches.
- **[low → FIXED] whitespace-only line mutant gap** — added a `" "` line to the malformed test.
- **[doc → FIXED] C=unknown falls back to Details** (the #95 tolerant pattern) — documented in the
  restore_shell doc rather than pretending it's skipped.
- **[accepted, documented] active_tab off-by-shift when tabs drop on restore** (missing code file): the
  clamp keeps it valid; a remap is over-engineering for an FS-changed-between-runs case. **[accepted]**
  terminal-N naming regenerates by grid count (titles aren't persisted — no rename surface exists).
  **[signed off] all-roots-missing → legacy fallback overwrites the blob on the first mutation** — spec'd
  "missing → dropped"; the unmounted-volume caveat noted here.
- Critic-verified clean: no PTY leak on any path (the legacy expect is dead-defensive); no persist during
  boot; 256/256 nextest; clippy clean.

Lenses: id-block continuity, PTY lifecycle, codec framing/injection, TOML escaping, round-trip identity,
mutation-killability, index-shift honesty.

## Phase 4 — Validate
- **Tests:** shell_codec_round_trips (2 projects, all 3 variants + a GOLDEN wire assert); shell_codec_empty_defaults; shell_codec_malformed_pieces_skipped (bad idx ×2, empty-root, blank + whitespace-only lines, unknown tag, tabs-less); shell_codec_drops_framing_breakers_on_serialize (\t root → project gone; \n path → tab gone); settings round-trip extended with a \n\t shell blob (byte-exact through TOML). 6 targeted + the crate suite.
- **Self-test:** sp_before.png — the driven shape (⌘T + the 🤖 cockpit tab + ⌘1 + right-click→Enter split): rail = terminal 1 [pane 1, pane 2] + terminal 2 + Agents, active t1, the split RENDERED. sp_after.png — after kill+relaunch the rail is IDENTICAL (the split panes, the cockpit tab, actives all restored). REQ-003 pixel-proven.
- **Gate:** GREEN [diff] 15/15 first try (codec + setting cov/MSI 100).

## Phase 5 — Complete
- CHANGELOG + app_shell #163 note; forge #163 → done; failure-record boot-local-shadows-field (3f7407b6). **THE /work 161-170 GOAL IS COMPLETE: 10/10.** LESSONS: embed codecs (framing-safe alphabets); enforce D2 at write time; piggyback persists but audit SWITCH sites; golden-wire asserts beat identity-only round-trips.
