---
pipeline_id: 106e9e43-303b-4c8d-9e22-1c58ab115efe
ticket: forge#83 (e7b17bf2-3978-4ff4-b6f9-c711ee5e00ee) · local docs/planning/tickets/open/TICKET-083-ssh-target.md
aar_id: 2ef9bbd3-8c98-4eb7-9d74-26755c804a5a
status: Phase 5 — Complete PASS
title: parse an ssh target + build the ssh command
type: feature
milestone: M3.A
references:
  - crates/marley_remote/src/lib.rs (NEW crate — SshTarget + parse_ssh_target + ssh_command)
---

## Title
The pure foundation of the remote seam: parse a typed `[user@]host[:port]` into an `SshTarget` and build
the `ssh` argv from it. A new `marley_remote` domain crate. Marley spawns the user's ssh (argv, no shell,
no secret handling) — ssh owns all security.

## Scope
### In
- NEW crate `crates/marley_remote` (pure, gpui-free; workspace member; mirrors marley_agent).
- `SshTarget { user: Option<String>, host: String, port: Option<u16> }`.
- `parse_ssh_target(&str) -> Option<SshTarget>` — `[user@]host[:port]`, incl. bracketed IPv6 `[::1]:22`;
  reject empty / empty-user / empty-or-bad-or-overflow-port / empty-host / bare (unbracketed) IPv6.
- `ssh_command(&SshTarget) -> Vec<String>` — the argv `["ssh", ("-p", port)?, [user@]host]`.

### Out
- The SPAWN of the ssh pane (#84). The badge (#85), status (#86), hosts config (#87). ssh options beyond
  `-p` (identity files etc. — ssh's own config handles them). Password prompting (ssh does it in-pane).

## Reference (§20)
Not recorded: this gpui-era spec predates the §20 reference section. The section was
added on 2026-09-22 when the archive entered the fork's history; the notes file holds
what the pipeline read.

### Prior art
Not recorded: the prior-art sweep became a required step after this spec was written.

## Locked-In Decisions
- D1 — a NEW `marley_remote` crate (the remote domain, matching marley_agent/marley_forge_client), not a
  module in marley_command — keeps the pure ssh logic isolated + testable.
- D2 — the argv is a `Vec<String>` (never a shell string) → a host/user string can NOT inject flags or
  commands (the security invariant); `-p PORT` is separate argv items.
- D3 — bracketed IPv6 (`[::1]`, `[::1]:22`) IS supported (a clean bracket handler); a BARE unbracketed
  IPv6 (`::1`) is REJECTED (→ None) — use brackets (documented).
- D4 — Marley handles NO ssh secrets (keys, known_hosts, passwords) — ssh does; the crate has no secrets.

## Acceptance Criteria (EARS)
| # | EARS (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN `parse_ssh_target` gets `[user@]host[:port]`, it shall return the split fields (user/host/port), trimming outer whitespace. | unit |
| REQ-002 | WHEN the input is empty, has an empty user, an empty/non-numeric/overflowing port, an empty host, or a bare IPv6, `parse_ssh_target` shall return `None`. | unit |
| REQ-003 | WHEN `parse_ssh_target` gets bracketed IPv6 (`[::1]`, `[::1]:22`), it shall return the inner host (+ port). | unit |
| REQ-004 | WHEN `ssh_command` is called, it shall be `["ssh", "-p", port?, [user@]host]` (argv, no shell). | unit |
| REQ-005 | `scripts/gates.sh` GREEN, cov/MSI 100 on `marley_remote`. | gate |

## Phase Plan
- **P2** — the crate scaffold + parse_ssh_target (user split, port split incl. brackets, the rejects) +
  ssh_command; mutation targets; test plan.
- **P3** — create `marley_remote` (Cargo.toml + lib.rs + workspace member) + implement.
- **P3.5** — 1 critic (correctness + a security pass): the parse edge cases + MSI; the argv-not-shell
  no-injection invariant; no secrets.
- **P4** — parse_ssh_target + ssh_command unit tests (cov/MSI 100) + gate GREEN. (Library — no UI self-test.)
- **P5** — docs, AAR, archive, close #83.
