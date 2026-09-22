# parse an ssh target + build the ssh command — Notes

- **Forge ticket:** #83 `e7b17bf2-3978-4ff4-b6f9-c711ee5e00ee` (in the BACKLOG — M3.A sprint pending
  forge sprint-create recovery)
- **AAR:** `2ef9bbd3-8c98-4eb7-9d74-26755c804a5a`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-083-ssh-target.md

## Phase 1 — Plan
- **Request:** forge #83 (M3.A seq-1, auto-approved) — the pure ssh target parse + command. First M3.A.
- **Classification:** work pipeline, `feature`, a NEW pure crate `marley_remote`. No UI.
- **Pre-flight facts:** the workspace domain crates are marley_agent/marley_forge_client (pattern to
  mirror); sessions spawn via `TerminalSession::spawn(SessionOptions{…})` (marley_command) at app.rs:178
  — #84 will feed `ssh_command` there. No marley_terminal (it's marley_command).
- **Decisions:** D1 new marley_remote crate; D2 argv not shell (no injection); D3 bracketed IPv6 in,
  bare IPv6 out; D4 no ssh secrets in Marley (ssh owns them).
- **AAR id:** `2ef9bbd3-8c98-4eb7-9d74-26755c804a5a`.

## Phase 2 — Design

### NEW crate `crates/marley_remote`
- `Cargo.toml` (mirror marley_agent — name marley_remote, edition/license workspace, NO deps). Auto-joins
  the workspace (`members = ["crates/*"]`, no root edit).
- `src/lib.rs`:
```rust
//! PURE — the remote (ssh) seam: parse a `[user@]host[:port]` target + build the `ssh` argv. Marley
//! spawns the user's ssh client (argv, no shell); ssh owns all security (keys, known_hosts, auth).

/// A parsed ssh destination.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SshTarget {
    pub user: Option<String>,
    pub host: String,
    pub port: Option<u16>,
}

/// Parse `[user@]host[:port]` (incl. bracketed IPv6 `[::1]:22`). `None` on empty / empty-user /
/// empty-or-bad-or-overflow port / empty-host / a bare (unbracketed) IPv6.
pub fn parse_ssh_target(s: &str) -> Option<SshTarget> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (user, rest) = match s.split_once('@') {
        Some((u, _)) if u.is_empty() => return None, // "@host"
        Some((u, r)) => (Some(u.to_string()), r),
        None => (None, s),
    };
    let (host, port) = parse_host_port(rest)?;
    if host.is_empty() {
        return None;
    }
    Some(SshTarget { user, host: host.to_string(), port })
}

/// Split `host[:port]`, honoring bracketed IPv6. A bare unbracketed IPv6 (extra `:`) → `None`.
fn parse_host_port(s: &str) -> Option<(&str, Option<u16>)> {
    if let Some(rest) = s.strip_prefix('[') {
        let (host, after) = rest.split_once(']')?; // [host] / [host]:port
        match after {
            "" => Some((host, None)),
            _ => {
                let port = after.strip_prefix(':')?;
                Some((host, Some(parse_port(port)?)))
            }
        }
    } else if let Some((host, port)) = s.rsplit_once(':') {
        if host.contains(':') {
            return None; // bare IPv6 — use brackets
        }
        Some((host, Some(parse_port(port)?)))
    } else {
        Some((s, None))
    }
}

/// A non-empty port that fits in a u16.
fn parse_port(s: &str) -> Option<u16> {
    if s.is_empty() {
        None
    } else {
        s.parse().ok()
    }
}

/// The `ssh` argv for `target` — `["ssh", ("-p", port)?, [user@]host]`. An argv (never a shell string),
/// so a host/user can't inject flags or commands.
pub fn ssh_command(target: &SshTarget) -> Vec<String> {
    let mut argv = vec!["ssh".to_string()];
    if let Some(port) = target.port {
        argv.push("-p".to_string());
        argv.push(port.to_string());
    }
    argv.push(match &target.user {
        Some(user) => format!("{user}@{}", target.host),
        None => target.host.clone(),
    });
    argv
}
```

### File manifest
- ADD `crates/marley_remote/Cargo.toml` — the crate manifest (no deps).
- ADD `crates/marley_remote/src/lib.rs` — SshTarget + parse_ssh_target + parse_host_port + parse_port +
  ssh_command + tests.

### Mutation Targets (pure)
- `parse_ssh_target`: the trim/empty, the `@` split, the empty-user reject, the empty-host reject.
- `parse_host_port`: the bracket branch (`[host]` vs `[host]:port` vs malformed), the rsplit branch, the
  bare-IPv6 `contains(':')` reject, the else (no-port).
- `parse_port`: the empty reject, the u16 parse (overflow).
- `ssh_command`: the `-p` emission (port present), the user@host vs host formatting.

### Regression Test Plan
| REQ | Test | Kind |
|---|---|---|
| REQ-001 | `parse_targets_ok` — "h"→{host}; "u@h"→{user,host}; "h:22"→{host,port}; "u@h:2222"→all; " u@h "→trimmed | unit |
| REQ-002 | `parse_targets_reject` — ""; "u@" (no host); "@h" (empty user); "h:" (empty port); "h:x" (bad); "h:99999" (overflow); "::1" (bare IPv6) → None | unit |
| REQ-003 | `parse_bracketed_ipv6` — "[::1]"→{host:"::1"}; "[::1]:22"→{host:"::1",port:22}; "u@[::1]:22"→all; "[::1]x"→None | unit |
| REQ-004 | `ssh_command_argv` — no-port→["ssh","u@h"]; with-port→["ssh","-p","22","u@h"]; no-user→["ssh","h"] | unit |
| REQ-005 | gate GREEN, cov/MSI 100 marley_remote | gate |

Uncoverable: none (fully pure). No UI self-test (library).

### Risks / decisions
- D-2.1 `parse_host_port` uses `rsplit_once(':')` + a `host.contains(':')` guard so `host:port` works and a
  bare IPv6 is rejected (→ use brackets). D-2.2 the bracket path handles `[host]` / `[host]:port` and
  rejects trailing junk (`[h]x`). D-2.3 the argv is a `Vec<String>` — the no-injection security invariant.
  D-2.4 `parse_port` factored out so the empty + overflow rejects are one tested unit.

## Phase 3 — Implement
- **Built (NEW crate `marley_remote`):** `Cargo.toml` (no deps; auto-joined the workspace); `src/lib.rs`
  — `SshTarget{user,host,port}`, `parse_ssh_target` (trim/empty; `@`-user split w/ empty-user reject;
  `parse_host_port`), `parse_host_port` (bracket IPv6 / rsplit host:port / bare-IPv6 reject / no-port),
  `parse_port` (empty + u16-overflow reject), `ssh_command` (the argv).
- **Deviations:** the empty-user check is a `Some(("", _))` PATTERN (not a `u.is_empty()` guard) — clippy
  redundant_guard.
- **Verification:** `cargo fmt`; `cargo check -p marley_remote` 0 err; clippy `-D warnings` OK; the crate
  builds + is in the workspace. Tests are Phase 4.

## Phase 3.5 — Inspect
- **Critic:** 1 (correctness + security; probe against real OpenSSH + cargo-mutants + hand-mutation of the
  method-call logic). Verdict: **found 1 HIGH (fixed) + 1 LOW test-gap; pure logic otherwise correct.**
- **Confirmations:** (a) parse correct across ALL cases; **MSI 100** (16 mutants: 15 caught, 1 unviable
  [no Default], 0 missed). cargo-mutants doesn't mutate `contains`/`is_empty`/`strip_prefix`/`rsplit_once`,
  so the critic HAND-mutated them — all killed by the REQ tests (the `host.contains(':')` flip+delete,
  the parse_port `is_empty` flip, the empty-user + empty-host guard drops). `rsplit_once`→`split_once` is
  an EQUIVALENT mutant (any multi-colon host is rejected by the contains(':') check → unobservable).
  (b) bracket-IPv6 in / bare-IPv6 out confirmed. (d) no secrets; zero deps; §20 original.
- **Findings + actions:**
  | # | Sev | Finding | Action |
  |---|---|---|---|
  | S1 | **HIGH** (security) | A host/user starting with `-` (e.g. `-oProxyCommand=evil`) was emitted as a leading-dash argv item that ssh RE-PARSES AS AN OPTION, not a destination — option-smuggling (CVE-2017-1000117 class). The crate's own "can't inject flags" doc was FALSE. Not a one-shot RCE today (one dest arg → ssh exits at usage), but a broken invariant + latent RCE the moment a 2nd arg is added. | **FIXED (both, defense-in-depth):** parse_ssh_target REJECTS a leading-`-` host OR user (→None); ssh_command inserts `"--"` before the destination (guards even a hand-built SshTarget). Verified against real ssh. Doc updated. |
  | S2 | LOW (test-gap) | `[::1]22` (numeric junk after `]`, no `:`) is correctly rejected by the code but not pinned by a planned test (a mutant dropping the `:` requirement would survive; cargo-mutants doesn't emit it). | P4: add `parse_ssh_target("[::1]22")==None`. |
  | N1 | NOTE | Inner whitespace preserved, port 0 accepted, multiple `@`, space-in-host — all single argv items → no injection (ssh just fails to resolve). | Accept. |
- **Fix applied (code):** S1 (leading-`-` reject + `--` argv). clippy OK, 0 err. → BF + PR recorded.

## Phase 4 — Validate
- **Tests added** (lib.rs): `parse_targets_ok` (REQ-001), `parse_targets_reject` (REQ-002 — 11 rejects),
  `parse_bracketed_ipv6` (REQ-003 + the S2 `[::1]22` gap), `parse_rejects_leading_dash` (S1 security —
  `-oProxyCommand=…` host/user → None), `ssh_command_argv` (REQ-004 — the `["ssh", -p?, "--", dest]` argv +
  a hand-built `-x` neutralized by `--`).
- **Runs (actual):** `cargo nextest run -p marley_remote` → 5 passed.
- **Gate:** `scripts/gates.sh --diff` → **GREEN [diff] 15/15**, cov 100%, MSI 100% on marley_remote.
- **Pre-existing:** none. (Library crate — no UI surface; the pure parse/command is fully unit-tested.)

## Phase 5 — Complete
- (pending)

## Phase 5 — Complete
- **Docs:** CHANGELOG `### Added`; NEW docs/marley_architecture/marley_remote.md.
- **Knowledge:** aar-submit (5); BF-claude-ssh-command-leading-dash-host-is-option-smuggling-injection-001 + PR-claude-subprocess-positional-arg-guard-leading-dash-and-double-dash-001 (HIGH — the critic found + I fixed a real ssh option-smuggling hole; argv alone is NOT enough, need leading-dash-reject + `--`).
- **Ticket:** forge #83 → done. **1/5 of M3.A** (the tickets are in the forge BACKLOG tagged M3.A — sprint-create is erroring forge-side; group them when it recovers). marley_remote parses ssh targets + builds the (injection-safe) ssh argv.
