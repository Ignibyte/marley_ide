# marley_spec_provenance — Notes

- **Forge ticket:** #8 `e1d6c657-efed-4aa8-b793-715c212b0c8c` (feature, M0), claimed `dc7df9b5-…`.
- **AAR:** `e247239e-99c3-41a4-af2e-a9951b3ac8d0`.
- **Local ticket doc:** `docs/planning/tickets/open/TICKET-005-spec-provenance.md`.
- **Pipeline spec:** `marley-spec-provenance.spec.md` (pipeline_id `e973adb9-…`).
- **Branch:** `ticket-005-spec-provenance` (stacked on 004).

## Phase 1 — Plan

- **Request:** the Rust gate-16 (`marley_spec_provenance`) replacing the interim
  `scripts/spec-provenance.sh`, per SPEC-gate R1–R15.
- **Classification / tier:** work pipeline, **feature** (INVENT tooling crate).
- **Forge recall (§18.3):** apply PR-…-injected-cwd-seam (testable dir seams),
  PR-…-gate-defining-files-in-receipt (scripts/spec-provenance.sh is fingerprinted —
  editing it is consistent). Pure logic → no serial/state concerns.

### Confirmed pre-flight
- **§11.1** (`seam-contracts.md:412`) is a markdown table `| STRIP | USE | Spec |`;
  col-1 backtick ids → denylist, col-2 → allowlist (R11, DATA).
- **docs/warp_architecture/** = 87 `.md` files (R5/R13 harvest).
- The interim shell gate-16 is CLEAN over live specs (explicit-denylist + spec_source
  only; no R5 harvest).

### Carry to Design — the SELF-TEST is the crux (D3)
- The crate over LIVE `docs/specs/` must report **0** violations. R5's 87-doc harvest
  WILL false-positive on common type names. **Spike `harvest_fork_identifiers` over the
  87 docs + `check_specs` over live `docs/specs/` FIRST**, then tune:
  - Conservative harvest: only backtick-code-span identifiers that look fork-private
    (CamelCase / `::`-path), filtered by a **stop-set** (std/primitive/common: String,
    Vec, Result, Option, Path, PathBuf, bool, u64, usize, Range, Command, Config, Stdio,
    BTreeSet, Cursor, Read, …).
  - Allowlist = §11.1 col-2 + the live specs' own Marley-original public-surface names +
    permissive-reuse (gpui::Hsla, ropey::Rope, alacritty_terminal::*).
  - **Iterate to 0.** If not reachable conservatively → present a scope decision (narrow
    R5 to the §11.1 strip-column diff + explicit denylist; defer the broad harvest). Do
    NOT silently weaken (§0/§15).
- **extract_public_surface (R2)** — slice `## Public surface (the contract)` → next `## `
  (or EOF), with 1-based line numbers. The shell excludes blockquote `>` lines (R8 narrative);
  decide pulldown-cmark section-walk vs a line scanner that tracks line numbers (line
  numbers are easier with a line scanner; pulldown-cmark for robust heading detection).
- **R7 identifier boundary** — a regex like `(?:[A-Za-z_][A-Za-z0-9_]*)(?:::[A-Za-z_][A-Za-z0-9_]*)*`
  tokenization, whole-word (so `BufferDelta` doesn't match inside `MyBufferDelta`).
- **SPEC-gate.spec.md self-exception** — the shell skips SPEC-gate.spec.md (its surface
  names the denylist as string DATA). The Rust crate's R8 (only the public-surface
  section) + the allowlist must keep SPEC-gate.spec.md clean (its denylist names are in a
  bullet list inside the surface — confirm they're treated as DATA / allowlisted, or
  apply the same self-exception).
- **Gate wiring (D5):** `scripts/spec-provenance.sh` → `cargo run -q -p marley_spec_provenance -- --specs docs/specs --fork-docs docs/warp_architecture` (+ config). Keep shellcheck-clean.

### §21 reminder
`.rs` ticket → enforce-changelog needs a staged CHANGELOG entry; complete adds
`docs/marley_architecture/marley_spec_provenance.md`.

**Phase 1 status:** PASS (autonomous-through-commit per session goal). → Phase 2 Design.

## Phase 2 — Design → CANCELLED (gate-16 removed per chad's decision)

### Self-test spike (scratchpad/genprobe) — the decisive finding
- Curated distinctive denylist + SPEC-gate self-exception → **0 denylist violations** on the
  live tree (matches the working shell).
- **R5 as specified is unworkable.** Harvesting identifiers from the 87 `docs/warp_architecture/`
  docs = **2047 ids**; diffed against the live specs' public surfaces it flagged **553
  occurrences / 194 distinct tokens** — almost all **legitimate**: Marley's OWN API (`FileId`,
  `SessionId`, `CharOffset`, `StandardizedPath`, `FeatureFlag`, …), std traits (`Serialize`,
  `PartialEq`, `IntoIterator`, `FromStr`, …), and English words (`The`, `Every`, `Returns`,
  `Left`, …). The mandated golden self-test (live → 0) is impossible without a ~194-name
  allowlist that defeats the gate. Marley + Warp share Rust/terminal vocabulary, so a raw
  fork-doc identifier diff cannot distinguish a fork-private name from a legitimate one.

### Decision (chad, 2026-06-28): REMOVE gate-16 entirely; cancel this ticket.
Marley is a private, non-OSS rebuild; the clean-room risk is narrow (no obvious "stolen"
artifact) and is handled by manual review, not an automated identifier-diff gate. The
follow-up work — removing the gate-16 apparatus (gates.sh provenance_g, scripts/spec-provenance.sh,
the 16→15 gate-count docs, the crate stub + SPEC-gate.spec.md) — is its own ticket. See
[[gate-16-removed-private-rebuild]].

**Phase 2 status:** CANCELLED. No crate built. Pipeline archived as cancelled.
