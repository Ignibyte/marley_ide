# TICKET-004 — marley_command (Unix-only)

- **Forge ticket:** #7 `8ce39d6e-8632-4a44-a8fe-f5255c15f913` (feature, M0)
- **Owner:** `dc7df9b5-63b4-4f63-a30a-ee7d51bb56f7`
- **AAR:** `5cc28178-8f9c-4401-ab29-946f690c2d17`
- **Pipeline doc:** `../../pipeline/active/marley-command.spec.md`
- **Authoritative spec:** [SPEC-process-command.spec.md](../../../specs/SPEC-process-command.spec.md)
- **Branch:** `ticket-004-marley-command`
- **Windows half:** forge#6 / TICKET-004b (BLOCKED on a Windows CI runner)
- **Status:** closed — delivered Unix-only (forge #7 → done); GATE GREEN [full], cov 100 / MSI 100, 16 tests. Windows half = forge #6 / 004b.

## Summary

The non-PTY child-process spawn seam — `blocking::Command` + `r#async::Command`
builders, default stdio, env/cwd, spawn/status/output, re-exports, WSL detection, and
the `std::process::Command` workspace ban. **Scoped Unix-only**: the Windows
console-flash-suppression half (R6 `CREATE_NO_WINDOW`, R7 `JobObject`) is deferred to
forge#6 (needs a Windows CI runner — cargo-mutants surfaces unkillable mutants for
`#[cfg(windows)]` code on macOS-only CI).

## Acceptance

The IN requirements (R1–R5, R8–R14, R15-partial) + 100% coverage + mutation MSI 100
(no `#[cfg(windows)]`) + FULL gate green + a §21 CHANGELOG entry &
`docs/marley_architecture/marley_command.md`. Full criteria in the pipeline spec
(AC-A…AC-H).
