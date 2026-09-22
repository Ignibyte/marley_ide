---
pipeline_id: 3a4f8cee-467b-4acb-9248-f15c3c608a03
ticket: forge#375 (b20f9b7b-b210-4310-810d-3399eabd7d45) · local docs/planning/tickets/open/TICKET-375-marley-mcp-hardening.md
aar_id: 704f2b42-ad76-4e5b-875c-a2b9cc2e7af3
status: Phase 5 — Complete PASS
title: marley_mcp — bearer CSPRNG + discovery-file perms + per-session Mcp-Session-Id (#370 hardening)
type: chore
milestone: M23.5
references:
  - docs/planning/pipeline/completed/370-marley-mcp-server-l1.spec.md
  - docs/marley_architecture/orchestration-shell.md
  - crates/marley_mcp/src/transport.rs
  - crates/marley_mcp/src/auth.rs
  - crates/marley_app/src/mcp_host.rs
---

## Title
Defensive hardening of Marley's OWN loopback MCP server (#370) against the threat model L1 honestly
deferred (transport.rs:70 self-documents the weak bearer; the #370 notes name the CSPRNG bearer as
the D-OPEN-LISTEN follow-up): **more than one local client + a hostile local process / multi-user
box**. None of this blocked the single-client loopback slice; all of it is required before the
manager seat and a second client share the box with anything untrusted. Three clusters: (A) a
128-bit OS-CSPRNG bearer + refuse-to-start-on-entropy-failure + a constant-time-ish compare; (B) the
discovery file created/corrected to 0600 and removed on clean shutdown; (C) the MCP-spec
`Mcp-Session-Id` lifecycle over a BOUNDED pure `SessionRegistry`. **Soft order: after #374** (the
grant wiring may touch `start_mcp_server`'s surface — this ticket edits the same fn to delete the
salt derivation).

## Scope
### In
- **(A) ENTROPY.** Mint the bearer fresh per server start from 16 bytes of `/dev/urandom` read via
  `std::fs` (D1). Seam split: pure `bearer_from_entropy([u8; 16]) -> String` (32 lowercase hex
  chars, length-pinned — cov/MSI 100) + a masked ~3-line entropy read in the already-excluded
  transport shim. Entropy-read failure → typed error, server refuses to start, NEVER a weaker
  fallback (D2); the app stays up (`McpHost::start`'s no-panic contract, mcp_host.rs:24-27, holds).
  The SAME entropy path feeds session ids. Bearer comparison becomes a pure constant-time-ish fn
  (length check + XOR-accumulate over all bytes, no early content exit — D5), replacing the `==` in
  `auth::bearer_ok` (auth.rs:42). Bearer-never-logged EXTENDS to session ids (regression REQ-013).
- **(B) DISCOVERY FILE.** `mcp-endpoint.json` (today: `std::fs::write` into
  `~/.marley/config`, mcp_host.rs:58-63 → 0644 under the default umask): create 0600 via
  `std::os::unix::fs::OpenOptionsExt::mode(0o600)`; when the file PRE-EXISTS, correct its
  permissions to 0600 BEFORE the new bearer bytes land (open-with-mode only applies on create).
  The file IO helper takes a dir param (`write_discovery_file_in(dir, …)` /
  `remove_discovery_file_in(dir)`) per the house `*_in(dir)` testable-IO idiom
  (`settings_file_in`/`load_manager_in`, settings.rs:344/353) and lives in a NEW covered
  `marley_mcp` module — tempdir metadata-mode asserts, real coverage. REMOVE the file on clean
  shutdown (D7): today NO stop path exists (no `Drop`, no quit hook — audit below); this ticket
  DEFINES clean shutdown as McpHost teardown (`impl Drop for McpHost` → remove; the app's orderly
  quit path drops the host — gpui offers `on_app_quit`, gpui-0.2.2/src/app.rs:1771, wiring is a
  Phase-2 pick). Abnormal exit (kill -9, panic-abort, power loss) leaving the file is acknowledged
  and out of scope.
- **(C) PER-SESSION `Mcp-Session-Id`** per the MCP spec 2025-06-18 "Transports → Streamable HTTP →
  Session Management": assign a fresh id (same entropy path) via the `Mcp-Session-Id` header on the
  HTTP response to the InitializeRequest; every subsequent request must echo it — **400** when
  missing, **404** when unknown/terminated (the client then re-initializes — the CLIENT half ships
  in #372/#373); support HTTP **DELETE**-terminate (D6). Pure seam: a `SessionRegistry`
  (assign / validate / terminate / evict) with a BOUNDED count — cap 8, reject-new-when-full with a
  typed error, REJECTING evict-oldest (D4) — the #370 cap-before-alloc lesson applied to sessions.
  cov/MSI 100 on the registry + gate fn; the header read/write plumbing rides the existing masked
  transport (transport.rs), incl. a DELETE method arm (today DELETE falls into the GET/SSE else
  branch, transport.rs:155-158 — fixed as a side effect).
- **ORDERING/REGRESSION PIN.** Session validation slots into the EXISTING pre-dispatch order
  WITHOUT weakening it. Current order (cited): body cap inside `read_http_request`
  (transport.rs:253-256, pre-allocation) → Origin + bearer guards (transport.rs:122-126) → dispatch
  (`handle_message`, transport.rs:137). Resulting order: **cap → Origin → bearer → session →
  dispatch** — the cap and bearer stay pre-auth-first; an unauthenticated or over-cap request never
  touches the session registry (no pre-auth probe of session-id validity).
- **Regression pins (all #370 invariants):** bearer verified pre-dispatch (transport.rs:122-126);
  Origin check (auth.rs:31-36); loopback-only bind `127.0.0.1:0` (transport.rs:82); the 1MiB
  pre-auth body cap — the BF-mcp-pre-auth-body-alloc DoS fix (transport.rs:253-256); bearer never
  logged (redacted `Debug`, transport.rs:46-54).

### Out (explicitly deferred)
- TLS, non-loopback binds, multi-tenant auth — loopback-only stays binding (orchestration-shell §10).
- Token rotation/expiry beyond process lifetime (per-start mint is the contract).
- Wiring per-session `Subscriptions` across the POST/GET split (the #370 inspect-F4 L1
  "open-stream-IS-subscribe" simplification stays; the registry IS the `Mcp-Session-Id`-keyed store
  that L2 wiring will hang off, but the over-notify shortcut is not this ticket's bug).
- OAuth / anything beyond the static per-start bearer; #371's settings/grants surface (untouched).
- Hardening the *client* half (echo/re-initialize/reconnect) — #372/#373 own it.

## Reference (§20)
**N/A — Marley-specific security hardening; no Warp/Zed behavior analog.** This hardens Marley's own
self-hosted MCP server (bearer entropy, file permissions, protocol session lifecycle) — there is no
terminal/editor BEHAVIOR to match. The behavior contract is the open MCP specification (2025-06-18)
plus POSIX file-permission semantics. Clean-room §20 untouched: no Warp (AGPL) / Zed (GPL) source is
relevant or consulted; the checked behavior maps (below) confirm neither app exposes a comparable
self-hosted-server surface to observe.

### Prior art
1. **Behavior maps — checked, N/A.** `docs/warp_architecture/crates/mcp.md` maps Warp's MCP
   **client** runtime (an rmcp façade connecting OUT to external servers; OAuth for per-server auth
   — :15-24) — no self-hosted server, no bearer-minting/session-assign surface to map.
   `docs/zed_architecture/` has zero MCP-related material (grep). Stated per the sweep rule.
2. **Published — the MCP specification rev 2025-06-18, "Transports → Streamable HTTP → Session
   Management":** the server **MAY** assign a session id via the `Mcp-Session-Id` header on the HTTP
   response to the InitializeRequest; the client **MUST** echo it on every subsequent request; the
   server **SHOULD** respond **400** to a request missing a required session id and **MAY** respond
   **404** for an unknown/terminated session, upon which the client **MUST** re-initialize; the
   client **SHOULD** send HTTP **DELETE** with the header to terminate, and the server **MAY**
   respond 405 if it doesn't support client termination. Cluster (C) implements the server half of
   exactly this section (we choose to support DELETE — D6). The spec's Security Warning
   (Origin/loopback/auth) was already adopted verbatim in #370 and is pinned here as regression.
3. **OUR OWN source (the load-bearing leg — the audit that JUSTIFIES the ticket):**
   - **Bearer minting today:** `transport::derive_bearer(port, salt)` (transport.rs:72-75) =
     `format!("marley-{port:04x}-{salt:032x}")`; salt = `(agents.len() as u128).wrapping_add(1)
     .wrapping_mul(0x9E37_79B9_7F4A_7C15)` (app.rs:6586-6588). **Guessability weakness, plainly:**
     the port is enumerable by any local process (`lsof`/`netstat`), the multiplier is public source,
     and `agents.len()` is a tiny integer (almost always 0-8) → a hostile local process recovers the
     bearer in a handful of guesses. Self-documented as weak (transport.rs:70-71, app.rs:6584-6585)
     with the CSPRNG bearer as the named follow-up — this ticket IS that follow-up (a real fix, not
     verify-and-pin).
   - **Bearer compare today:** `auth::bearer_ok` (auth.rs:41-43) = `!expected.is_empty() &&
     presented == Some(expected)` — an early-exit `==` → content-dependent timing. Loopback-only
     makes this LOW severity (defense-in-depth, not a hot vuln — D5 documents that), but the
     constant-time-ish compare is the cheap pure part.
   - **Discovery file today:** `McpHost::write_discovery_file` (mcp_host.rs:57-63) →
     `std::fs::write(dir.join("mcp-endpoint.json"), …)` into `marley_core::marley_config_dir()` =
     `~/.marley/config` (paths.rs:38-40 — a per-user dir, but the FILE mode is 0666&~umask = **0644
     world-readable** under the default umask 022, `{url, bearer}` readable by any local user).
     **Nothing removes it:** no `Drop` impl in mcp_host.rs or marley_mcp, no quit/shutdown hook
     anywhere in app.rs/main.rs/lib.rs (greps empty) — the file (with the last bearer) persists
     across exits indefinitely.
   - **Session handling today: none.** `read_http_request` parses only Origin / Authorization /
     Content-Length (transport.rs:232-249); `initialize` returns no session id (dispatch.rs:27,
     :68-74); one implicit session — a fresh `Subscriptions::default()` per POST (transport.rs:136),
     the GET stream hardcodes `fleet: true` (transport.rs:174). Any non-POST method — including
     DELETE — falls into the SSE-stream arm (transport.rs:155-158). The CLIENT-side echo idiom
     already exists in-tree (`marley_forge_client::session_header`, lib.rs:241-245, speaking to
     Forge) — proof of shape for the #372/#373 client half.
   - **Pre-auth invariants to pin (all shipped in #370, cited in Scope):** cap → Origin → bearer →
     dispatch; loopback bind; redacted Debug.
   - **Lockfile:** `getrandom` (0.2.17/0.3.4/0.4.3) + `rand`/`rand_core`/`rand_chacha` are present
     **transitively only** (Cargo.lock); NO marley crate declares either (grep over
     `crates/*/Cargo.toml` — zero hits) → using getrandom would mean ADDING a direct dependency for
     one 16-byte draw — rejected (D1).
   - **`std::os::unix` precedent:** already used UNGATED in-tree (marley_project/src/lib.rs:496,
     marley_app/src/editor_surface.rs:1036) — target-native on the darwin-only CI, no cfg arm.
   - **Test approach to mirror (#370):** pure `handle_message` "fake transport" units
     (dispatch.rs:210-225 `call` helper) + per-module pure units; transport.rs + mcp_host.rs are
     masked + coverage-excluded (gates.sh:222). The registry/gate/hex/compare tests follow the same
     style; the discovery-file helper is deliberately NOT masked (tempdir-testable, `*_in(dir)`).

## Locked-In Decisions
- **D1 — Entropy source = `/dev/urandom` via `std::fs` (zero new deps).** On macOS `/dev/urandom`
  IS the CSPRNG and cannot block; `std::os::unix` is target-native on darwin (in-tree precedent,
  ungated — leg 3) so there is NO cfg'd-out arm and NO MSI hole on the macOS-only CI
  (the cross-platform-mutation lesson). **Rejected:** a direct `rand`/`getrandom` dependency —
  transitively-present is NOT usable without adding a direct dep, and one 16-byte draw does not
  warrant one. **Rejected:** keeping/patching `derive_bearer` (port/counter-derived = guessable,
  leg 3). **Rejected:** hashing time/pid/ASLR oddments — still guessable, and "clever" entropy is
  exactly what this ticket exists to kill.
- **D2 — Entropy failure ⇒ refuse to start, typed.** WHEN the `/dev/urandom` read fails, the server
  SHALL NOT start and SHALL NOT fall back to any weaker source (a silently-weak bearer is worse than
  no server); `McpHost::start` surfaces a typed error and the app stays up (its existing no-panic
  contract). **Rejected:** fallback-to-derive_bearer; panic.
- **D3 — The pure/masked line.** PURE (cov/MSI 100): `bearer_from_entropy`/the hex-128 formatter,
  the constant-time compare, the `SessionRegistry` + its request gate, the discovery JSON (already
  pure). MASKED (in the already-excluded transport.rs / mcp_host.rs): the ~3-line entropy read, the
  header read/write + DELETE routing, the Drop/quit wiring. **Deliberately NOT masked:** the
  discovery-file IO helpers — they take a dir param (`*_in(dir)`, the settings.rs:344/353 idiom),
  live in a NEW covered `marley_mcp` module, and are proven by tempdir metadata asserts (mode
  0o600), because file-permission correctness IS this ticket's payload, not plumbing.
- **D4 — Session cap: reject-new-when-full (cap 8), NEVER evict-oldest.** Silently killing a live
  manager's session to admit a new client is worse than refusing the new client — the cap is a
  local-DoS bound (the #370 cap-before-alloc lesson applied to sessions), not an LRU cache. A
  refused initialize gets a typed error; existing sessions are untouched. **Rejected:**
  evict-oldest/LRU; an unbounded registry (the exact allocation-DoS class F1 fixed for bodies).
- **D5 — Constant-time-ish compare as DEFENSE-IN-DEPTH.** Length check + XOR-accumulate over all
  bytes — no early-exit content branch; used for the bearer AND session-id comparisons. Documented
  honestly: on a loopback-only server this is hardening posture, not a hot vulnerability — which is
  why it's a 10-line pure fn, not a new dependency. **Rejected:** a `subtle`-style crate dep
  (overkill); leaving `==` (free to fix while we're in the fn).
- **D6 — DELETE-terminate SUPPORTED.** It's cheap (a registry `terminate` + a status line), the
  spec's client half SHOULD send it, and it fixes the current any-non-POST-becomes-SSE wrinkle
  (transport.rs:155-158). **Rejected:** 405-unsupported (spec-legal but leaves terminated-session
  hygiene to process exit only).
- **D7 — Clean shutdown = McpHost teardown; the file must not outlive the bearer.** `impl Drop for
  McpHost` removes the discovery file (via the tested `remove_discovery_file_in`); the app's orderly
  quit path drops the host (today NO stop path exists — leg 3; Phase 2 picks the wiring, with
  gpui's `on_app_quit` (gpui-0.2.2/src/app.rs:1771) as the grounded candidate). Abnormal exit
  leaving a stale file is acknowledged + out of scope (the next start overwrites it 0600 with a
  fresh bearer, so the lingering content is a dead credential).
- **D8 — Session ids are secrets-adjacent: same handling as the bearer.** Minted from the same
  entropy path, compared with the same constant-time fn, NEVER logged (the redacted-Debug idiom,
  transport.rs:46-54, extends to the registry), never written to disk.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN the server starts, the bearer shall be derived from 16 bytes of OS CSPRNG (`/dev/urandom`) formatted by the pure hex fn as exactly 32 lowercase hex chars — no port/pid/counter input. | pure `bearer_from_entropy` units (length 32, lowercase-hex charset, `[0;16]`→`"00…0"`, distinct inputs → distinct outputs); §18.1 inspect confirms the masked read is the only entropy source + `derive_bearer` is deleted |
| REQ-002 | WHEN the entropy read fails, the server shall refuse to start with a typed error — no weaker fallback, no panic, the app stays up. | pure credentials-builder unit fed an injected failing entropy `io::Result` → typed `Err`; negative: no code path constructs a bearer from anything but the entropy bytes (inspect) |
| REQ-003 | WHEN a presented bearer or session id is compared, the comparison shall length-check then XOR-accumulate over ALL bytes (no early content exit), and an empty expected value shall never match. | pure compare units: equal→true; first-byte diff→false; last-byte diff→false; length diff→false; empty expected vs empty/nonempty presented→false; `bearer_ok` regression suite (auth.rs) stays green on the new fn |
| REQ-004 | WHEN the discovery file does not exist, it shall be created with mode 0600 containing the fresh `{url, bearer}` JSON. | tempdir unit: `write_discovery_file_in(dir,…)` → `metadata.mode() & 0o777 == 0o600` + content round-trip |
| REQ-005 | WHEN the discovery file PRE-EXISTS (any mode), its permissions shall be corrected to 0600 BEFORE the new bearer bytes are written — the new bearer never exists in a >0600 file. | tempdir unit: pre-create the file 0644 with stale content → run the writer → assert mode 0600 + new content (mechanism — chmod-then-write vs remove-then-create-with-mode — is Phase 2's pick; the assert pins the observable) |
| REQ-006 | WHEN the McpHost is torn down on clean shutdown (Drop via the app's orderly quit), the discovery file shall be removed; a missing file is not an error. | tempdir unit on `remove_discovery_file_in` (removes; idempotent on absent); Drop-impl unit if constructible headlessly, else §18.1 inspect on the ~2-line Drop wiring |
| REQ-007 | WHEN a client sends `initialize`, the HTTP response shall carry a fresh `Mcp-Session-Id` (same entropy path, hex-128) and the registry shall hold that session. | pure registry `assign` units (id format; registered; distinct per assign); gate unit: initialize needs no prior id; transport header-write is masked (inspect) |
| REQ-008 | WHEN a non-initialize request arrives WITHOUT `Mcp-Session-Id`, the server shall respond 400 before dispatch. | pure session-gate unit (missing header + non-initialize → `Missing`→400 mapping) in the #370 `handle_message`-style fake-transport lane |
| REQ-009 | WHEN a request carries an UNKNOWN or TERMINATED session id, the server shall respond 404 (the client re-initializes — #372/#373's half) and shall NOT dispatch. | pure gate unit (unregistered id → `Unknown`→404; terminated id → same); no dispatch output on the refused arm |
| REQ-010 | WHEN an HTTP DELETE arrives with a VALID `Mcp-Session-Id`, the server shall terminate that session (subsequent requests with it → 404) and respond success; a DELETE with a missing/unknown id follows REQ-008/009. | pure registry units: `terminate(known)`→true, then `validate`→unknown; `terminate(unknown)`→false; gate unit for the DELETE arm |
| REQ-011 | WHEN the registry holds 8 live sessions and a new `initialize` arrives, the server shall refuse the new session with a typed error; the 8 existing sessions shall remain valid — no eviction. | pure registry unit: assign×8 ok → 9th refused (typed) → all 8 still validate; after one `terminate`, a new assign succeeds |
| REQ-012 | WHILE the server runs, the pre-dispatch order shall remain: 1MiB body cap → Origin → bearer → session gate → dispatch; an over-cap or unauthenticated request shall be refused WITHOUT consulting the session registry. | regression: all existing auth/cap #370 units green unchanged; the gate fn takes no auth inputs (structural); §18.1 inspect checkpoint on `serve_connection`'s guard order |
| REQ-013 | WHILE the server runs, no log/Debug/error path shall emit the bearer OR any session id; the discovery file remains the ONLY place the bearer lands, and session ids are never written to disk. | redacted-Debug units (`format!("{:?}")` on ServerHandle + SessionRegistry contains `***`, not the secret); grep-style inspect for format-string leaks on the new paths |
| REQ-014 | WHILE the server runs, the #370 invariants shall hold unchanged: loopback-only bind, Origin validation, bearer verified pre-dispatch, the 1MiB pre-auth body cap. | regression: the full existing marley_mcp suite green; cap-constant assert; bind-literal + guard-order inspect (masked lines) |

## Floors (constitution)
Pure seams (hex formatter, compare, registry + gate, discovery JSON) at **cov/MSI 100**; the entropy
read + socket + Drop/quit wiring masked in the already-excluded files; the discovery-file `*_in(dir)`
helpers COVERED (tempdir). Exhaustive `match` over defensive catch-all (the gate/registry outcomes
are closed enums — the #370 gate-4 lesson); typed errors end-to-end, no `unwrap` on any input path;
test code avoids never-run branches (irrefutable patterns / `matches!`, the #370 gate-4 idiom).

## Phase Plan
- **P2 Design** — module map + exact signatures: NEW `marley_mcp/src/session.rs` (registry + gate,
  pure) + NEW `marley_mcp/src/discovery.rs` (`write_discovery_file_in`/`remove_discovery_file_in`,
  covered) + `auth.rs` (compare swap) + entropy split (pure formatter placement; masked read in
  transport.rs); transport.rs deltas (header parse/write, DELETE arm, refuse-start plumbing);
  mcp_host.rs (Drop, dir-param call sites) + app.rs (`start_mcp_server` salt-block deletion —
  post-#374 surface); pick REQ-005's mechanism + REQ-006's quit wiring; per-REQ test plan.
- **P3 Implement** — pure seams first (session/discovery/auth/entropy formatter), then the masked
  transport/host deltas; delete `derive_bearer` + the app.rs salt derivation.
- **P3.5 Inspect** — adversarial: session gate genuinely post-auth? any entropy fallback path? any
  format-string that could carry a secret? REQ-005's no->0600-window claim honest? DELETE arm can't
  be reached unauthenticated? provenance (§20).
- **P4 Validate** — write + RUN the units per REQ; **run `cargo mutants --list -f` on the ACTUAL
  touched files** (session.rs, discovery.rs, auth.rs — the syntactic-form lesson) before claiming
  the kill set; gate green (`--diff`), cov/MSI 100 on the pure seams, tempdir mode asserts in-suite.
- **P5 Complete** — CHANGELOG; note the hardened posture in orchestration-shell §10's shipped state;
  AAR (lessons: entropy-refuse-typed, perms-before-secret-bytes); archive; close #375.
