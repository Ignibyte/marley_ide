# marley_mcp — bearer CSPRNG + discovery-file perms + per-session Mcp-Session-Id (#370 hardening) — Notes

- **Forge ticket:** #375 (b20f9b7b-b210-4310-810d-3399eabd7d45)
- **AAR:** 704f2b42-ad76-4e5b-875c-a2b9cc2e7af3 (opened at /work)
- **Local ticket doc:** docs/planning/tickets/open/TICKET-375-marley-mcp-hardening.md
- **Pipeline spec:** 375-marley-mcp-hardening.spec.md

## Phase 1 — Plan
- **Request:** Defensive hardening of Marley's OWN loopback MCP server (#370) — authorized security
  work on our own codebase. Threat model: more-than-one-local-client + a hostile local process /
  multi-user box (none of it blocked the L1 single-client loopback slice; #370 shipped with the weak
  bearer + no sessions HONESTLY documented as follow-ups — transport.rs:70-71, the #370 notes' P3
  deviation D3 and D-OPEN-LISTEN). Sprint #34 "M23.5 — Fleet Layer-1 Consolidation". Soft order:
  after #374 (grant wiring may touch `start_mcp_server`'s surface; this ticket edits the same fn).
- **Classification / tier:** chore (hardening), M23.5 → full pipeline
  (plan→design→implement→inspect→validate→complete→commit). Touches one shipped crate
  (`marley_mcp`) + its thin app glue (`mcp_host.rs`/`app.rs`); no new crate, no UI surface, no
  pixels. Sibling contracts: #372/#373 own the CLIENT half of session echo/re-initialize; #374 owns
  the grants-from-settings wiring — this ticket must not collide with either (server-side only).
- **Forge recall (§18.3):** drafted DOCS-ONLY (no MCP calls in this drafting lane); recall drawn
  from the on-disk record — the #370 spec/notes pair (pipeline/completed/370-…): D1's binding
  security frame (loopback + Origin + bearer, never logged), inspect F1 (the pre-auth 1MiB body-cap
  DoS fix = the cap-before-alloc lesson this ticket re-applies to SESSIONS), inspect F4 (the L1
  open-stream-IS-subscribe simplification — explicitly NOT this ticket's bug), P3 deviation D3 (the
  weak bearer, named follow-up = THIS ticket), D-OPEN-LISTEN (session granularity deferred);
  orchestration-shell.md §10 (two-credential posture, never-logged bearer, loopback-only binding);
  prevention rules re-applied: exhaustive-match-over-catch-all + test-code-no-never-run-branches
  (#370 gate-4), run-`cargo mutants --list`-on-the-actual-files (the syntactic-form trap), the
  cross-platform-mutation note (macOS-only CI — why an ungated unix-native path matters, D1).
  Live knowledge-context/bulletins recall re-runs at `/work` promotion.
- **Discovery — THE CURRENT-STATE AUDIT (leg (c); this is the ticket's justification):**
  - **(1) Bearer minting today:** `transport::derive_bearer(port: u16, salt: u128)`
    (crates/marley_mcp/src/transport.rs:72-75) = `format!("marley-{port:04x}-{salt:032x}")` —
    called from `spawn` (:84) with the OS-assigned port. The salt (crates/marley_app/src/app.rs:6586-6588,
    in `start_mcp_server`) = `(self.agents.len() as u128).wrapping_add(1).wrapping_mul(0x9E37_79B9_7F4A_7C15)`.
    **Guessability, plainly:** every input is knowable by a hostile local process — the port via
    `lsof`/`netstat`, the multiplier from public source, `agents.len()` a tiny integer (~0-8 in
    practice) → the full bearer enumerates in a handful of tries. The code SELF-DOCUMENTS the
    weakness ("A weak per-boot bearer… a strong CSPRNG bearer is a D-OPEN-LISTEN follow-up",
    transport.rs:70-71; "not a security boundary — a CSPRNG bearer is a follow-up", app.rs:6584-6585)
    → the ticket item is a REAL FIX, not verify-and-pin.
  - **(2) Bearer compare today:** `auth::bearer_ok(presented: Option<&str>, expected: &str)`
    (crates/marley_mcp/src/auth.rs:41-43) = `!expected.is_empty() && presented == Some(expected)` —
    a short-circuiting `==` (memcmp-style early exit) → content-dependent timing. Loopback-only ⇒
    LOW severity; named in the spec as defense-in-depth (D5), the pure 10-line part of cluster (A).
    The existing non-empty-expected guard (an empty expected must never match) is KEPT — its test
    (auth.rs:71-77) is regression.
  - **(3) Discovery file today:** `McpHost::write_discovery_file`
    (crates/marley_app/src/mcp_host.rs:57-63) → `std::fs::write(dir.join("mcp-endpoint.json"),
    self.discovery_json())` where dir = `marley_core::marley_config_dir()` = `~/.marley/config`
    (crates/marley_core/src/paths.rs:38-40; per-user, but nothing pins the FILE mode).
    `std::fs::write` creates 0666 & ~umask ⇒ **0644 world-readable** under the default umask 022 —
    any local user on a multi-user box reads `{url, bearer}` (the JSON carries the full
    `Authorization: Bearer …` header value, transport.rs:306-309). **Cleanup: none exists.** No
    `impl Drop` in mcp_host.rs or anywhere in marley_mcp (grep), no quit/shutdown/on_app_quit hook
    in app.rs/main.rs/lib.rs (greps empty) → there IS no "McpHost stop path" to cite; the spec
    therefore DEFINES clean shutdown (D7: `Drop for McpHost` + the app's orderly quit dropping the
    host; gpui 0.2.2 offers `on_app_quit` — gpui-0.2.2/src/app.rs:1771 — as the Phase-2 wiring
    candidate). Abnormal exit leaving the file: acknowledged, out of scope (next start overwrites
    0600 + fresh bearer, so the residue is a dead credential — and REQ-005 covers the pre-existing
    0644 file left behind by the CURRENT code, the actual migration case on chad's machine).
  - **(4) Session handling today: none.** The server never assigns `Mcp-Session-Id`:
    `read_http_request` parses ONLY Origin / Authorization / Content-Length
    (transport.rs:232-249); `initialize` → `initialize_result()` (dispatch.rs:27, :68-74) with no
    session material; the transport writes no session header anywhere. One implicit session: a
    fresh throwaway `Subscriptions::default()` per POST (transport.rs:136); the GET/SSE stream
    hardcodes `fleet: true` (transport.rs:174). **Wrinkle found:** ANY non-POST method — including
    DELETE — falls into the else arm and is served as an SSE stream (transport.rs:155-158); D6's
    DELETE support fixes this as a side effect. The CLIENT-side echo idiom already exists in-tree:
    `marley_forge_client::session_header` (lib.rs:241-245) + its use across adapter.rs:188-208
    (Marley echoing Forge's session id) — shape-proof for the #372/#373 client half; the request
    builders' doc even names the echo contract (lib.rs:194).
  - **(5) Pre-auth invariants to PIN (all shipped by #370, all regression rows):**
    - bearer verified pre-dispatch + Origin check: `serve_connection` refuses on
      `!origin_allowed || !bearer_ok` (transport.rs:122-126) BEFORE `handle_message` (:137);
    - loopback-only bind: `TcpListener::bind("127.0.0.1:0")` (transport.rs:82);
    - the 1MiB pre-auth body cap (BF-mcp-pre-auth-body-alloc, #370 inspect F1): `MAX_BODY_BYTES =
      1 << 20`, over-cap → `Ok(None)` → 400 with NO allocation (transport.rs:253-256) — note it
      runs INSIDE `read_http_request` (:118), i.e. before even the auth guards: cap → Origin →
      bearer → dispatch. The session gate slots between bearer and dispatch (REQ-012) — an
      unauthenticated probe must never learn session-id validity.
    - bearer-never-logged: `ServerHandle`'s redacted `Debug` (transport.rs:46-54) — extended to
      session ids by D8/REQ-013.
  - **Lockfile check:** `getrandom` present at 0.2.17/0.3.4/0.4.3 + `rand`/`rand_core`/`rand_chacha`
    — ALL transitive (Cargo.lock); `grep rand\|getrandom crates/*/Cargo.toml` = zero hits → no
    marley crate may `use getrandom` today; adopting it = a NEW direct dep for one 16-byte draw →
    rejected (D1). `/dev/urandom` via `std::fs` needs nothing; `std::os::unix` already used ungated
    in-tree (marley_project/src/lib.rs:496-497, marley_app/src/editor_surface.rs:1036) — darwin-only
    CI, no cfg'd-out arm, no MSI hole (the cross-platform-mutation memory).
  - **Test approach to mirror:** #370 proves the server via PURE `handle_message` units — the
    in-memory "fake transport" `call` helper (dispatch.rs:210-225) + per-module units (auth.rs
    tests :45-78); transport.rs + mcp_host.rs are masked + coverage-excluded (scripts/gates.sh:222
    already lists both). New pure modules (session.rs registry+gate, the hex formatter, the compare)
    ride the same lane. The discovery-file helpers are deliberately NOT masked: the house
    `*_in(dir)` testable-IO idiom (`settings_file_in` settings.rs:344, `load_manager_in` :353;
    paths.rs's `home_dir_from(Option<PathBuf>)` seam :26-29) → tempdir tests with
    `metadata.permissions().mode() & 0o777 == 0o600` asserts give REAL coverage of the payload
    behavior (file perms ARE the deliverable, not plumbing).
  - **Spec leg (b) (cited by section name):** MCP spec rev 2025-06-18, "Transports → Streamable
    HTTP → Session Management" — server MAY assign via `Mcp-Session-Id` on the InitializeRequest's
    HTTP response; client MUST echo on every subsequent request; server SHOULD 400 a missing
    required id; MAY 404 unknown/terminated (client MUST then re-initialize); client SHOULD
    DELETE-terminate; server MAY 405 if unsupported (we support DELETE — D6). Leg (a): the behavior
    maps checked — docs/warp_architecture/crates/mcp.md is Warp's MCP **client** runtime (rmcp
    façade; :15-24), no self-hosted-server surface; docs/zed_architecture/ has no MCP material
    (grep) → N/A, stated not skipped.
- **Decisions:** D1 `/dev/urandom` via std::fs LOCKED (zero deps; macOS CSPRNG, non-blocking;
  ungated unix-native → no MSI hole; rejected: direct getrandom/rand dep, keeping derive_bearer,
  time/pid hashing). D2 refuse-to-start typed on entropy failure (no weaker fallback; app stays up).
  D3 the pure/masked line (pure: hex formatter, compare, registry+gate, discovery JSON; masked:
  entropy read, header/DELETE plumbing, Drop wiring; NOT masked: the `*_in(dir)` file helpers —
  tempdir-proven). D4 session cap 8, reject-new-when-full, typed; REJECT evict-oldest (never kill a
  live manager's session to admit a stranger; DoS bound, not LRU). D5 constant-time-ish compare =
  defense-in-depth (documented as such; loopback ⇒ not a hot vuln; rejected: a subtle-crate dep).
  D6 DELETE supported (cheap; fixes the non-POST→SSE wrinkle; rejected: 405). D7 clean shutdown
  DEFINED = McpHost Drop removes the file; quit wiring = P2 (gpui on_app_quit grounded); abnormal
  exit out of scope. D8 session ids handled like the bearer (same entropy, same compare, never
  logged, never on disk). Ordering pin: cap → Origin → bearer → session → dispatch (REQ-012).
  14 EARS rows, every SHALL carried by a pure unit / tempdir assert / pinned regression.
- **Open for Phase 2 (bounded):** REQ-005's mechanism (chmod-then-write vs remove-then-recreate
  with mode — the assert pins the observable either way); REQ-006's quit wiring (`on_app_quit` vs
  relying on drop order); where the pure hex formatter lives (auth.rs vs the new session.rs);
  whether the session gate returns typed HTTP-status intents (`Missing400`/`Unknown404`) or the
  transport maps them — the gate stays pure either way; the exact typed-error shape for the cap-full
  initialize refusal (JSON-RPC error vs HTTP status — semantics locked by D4, wire shape P2).

## Phase 2 — Design

### Verified against LANDED code (audit confirmed)
`derive_bearer(port,salt)` = `format!("marley-{port:04x}-{salt:032x}")` (transport.rs:73 — guessable);
`bearer_ok` = `!expected.is_empty() && presented == Some(expected)` (auth.rs:42 — early-exit `==`);
`write_discovery_file` = `std::fs::write(dir.join("mcp-endpoint.json"),…)` (mcp_host.rs:58-63 — 0644, no
removal, NO Drop anywhere); sessions = NONE (`read_http_request` parses only origin/auth/content-length,
transport.rs:232-248; DELETE falls into the SSE else-arm, :155-158). Guard order today: body-cap
(:253-256) → origin+bearer (:122-126) → dispatch (:137). `getrandom`/`rand` transitive-only (no direct
dep — verified). §20 N/A (self-hosted-server hardening; MCP spec + POSIX perms are the contract).

### Pure seams (cov/MSI 100)
**auth.rs** — add `ct_eq(a,b) -> bool` (length-check + XOR-accumulate over all bytes, no early content
exit); `bearer_ok` rewritten to `!expected.is_empty() && presented.is_some_and(|p| ct_eq(p, expected))`.
**NEW secret.rs** — `hex128([u8;16]) -> String` (32 lowercase hex, length-pinned; feeds bearer AND session
ids); `mint_secret(Option<[u8;16]>) -> Result<String, EntropyError>` (`Some`→`Ok(hex128)`, `None`→typed
`Err` = refuse-to-start, D2). The masked layer maps the `/dev/urandom` `io::Result` → `Option` via `.ok()`.
**NEW session.rs** — `SESSION_CAP=8`; `SessionRegistry{ids:Vec<String>}` with `assign(id)->Result<(),Full>`
(reject-new at cap, NEVER evict — D4), `validate(&id)->bool` (ct_eq over all, no short-circuit — D8),
`terminate(&id)->bool` (removes, reports presence), `len()`; `SessionDecision{Initialize,Proceed,Terminate,
Reject(u16)}`; `session_decision(http_method, is_initialize, session_header, &registry)` — DELETE→
Terminate/Reject(400|404); initialize→Initialize; else valid→Proceed / missing→Reject(400) / unknown→
Reject(404). **NEW discovery.rs (COVERED, not masked — file-perm correctness IS the payload)** —
`write_discovery_file_in(dir,json)` (set_permissions 0600 if pre-exists, THEN `OpenOptions.mode(0o600)
.create.truncate.write` — the new bearer never lands in a >0600 file, REQ-005) + `remove_discovery_file_in
(dir)` (idempotent on absent); `std::os::unix::fs::{OpenOptionsExt,PermissionsExt}` (ungated darwin — leg 3).

### Masked wiring (already-excluded transport.rs / mcp_host.rs / app.rs)
- transport.rs: `read_entropy() -> Option<[u8;16]>` (read 16 bytes /dev/urandom via std::fs); `spawn` drops
  `salt`, mints via `mint_secret(read_entropy())` → an `io::Error` on `None` (→ `McpHost::start` returns
  `None`, app stays up); DELETE `derive_bearer`. `ServerData` gains `sessions: SessionRegistry`.
  `read_http_request` parses `mcp-session-id`; `HttpRequest` gains `session: Option<String>`.
  `serve_connection`: after auth, under the lock compute `is_initialize` (via `parse_request(body).method`)
  + `session_decision`; act — Initialize→mint id, `registry.assign` (Full→503), dispatch, write
  `Mcp-Session-Id: id` on the response; Proceed→dispatch; Terminate→`registry.terminate`+200; Reject(c)→
  `write_status(c)`. Guard order becomes **cap → Origin → bearer → session → dispatch** (session slots
  AFTER bearer; an unauth/over-cap request never touches the registry).
- mcp_host.rs: `start(grants)` (drop `salt`); `write_discovery_file` → `discovery::write_discovery_file_in`;
  NEW `impl Drop for McpHost` → `discovery::remove_discovery_file_in` (D7 defines clean shutdown =
  McpHost teardown; the app's orderly quit drops the host).
- app.rs: `start_mcp_server` deletes the salt block → `McpHost::start(self.mcp_expose.grants())`.

### File manifest
| # | File | Change |
|---|---|---|
| 1 | `crates/marley_mcp/src/auth.rs` | ADD `ct_eq`; rewrite `bearer_ok` through it + tests |
| 2 | `crates/marley_mcp/src/secret.rs` | NEW pure — `hex128` + `mint_secret` + `EntropyError` + tests |
| 3 | `crates/marley_mcp/src/session.rs` | NEW pure — `SessionRegistry` + `SessionDecision` + `session_decision` + tests |
| 4 | `crates/marley_mcp/src/discovery.rs` | NEW covered — `write/remove_discovery_file_in` + tempdir mode tests |
| 5 | `crates/marley_mcp/src/lib.rs` | mod + re-exports |
| 6 | `crates/marley_mcp/src/transport.rs` | MASKED: entropy read, spawn mint (drop salt), ServerData.sessions, session-header parse, serve_connection session gate + DELETE + Mcp-Session-Id header; delete `derive_bearer` |
| 7 | `crates/marley_app/src/mcp_host.rs` | MASKED: `start(grants)` (drop salt), discovery::-based write, `Drop` |
| 8 | `crates/marley_app/src/app.rs` | MASKED: delete the salt block at `start_mcp_server` |
| 9 | `CHANGELOG.md` | Added entry |

### Regression Test Plan (per REQ, all on the PURE seams)
| # | Test | Proves |
|---|---|---|
| hex128: len 32, lowercase-hex charset, `[0;16]`→all-zeros, distinct inputs→distinct | pure (secret.rs) | REQ-001 |
| mint_secret: `Some`→Ok(hex128), `None`→typed Err | pure | REQ-002 |
| ct_eq: equal→T, first/last-byte diff→F, length diff→F, empty→F; bearer_ok regression | pure (auth.rs) | REQ-003 |
| write_discovery_file_in: fresh→mode 0600 + content; pre-existing 0644→corrected 0600 + new content | covered (discovery.rs, tempdir) | REQ-004/005 |
| remove_discovery_file_in: removes; idempotent on absent | covered | REQ-006 |
| session_decision: initialize→Initialize; missing→Reject(400); unknown→Reject(404); DELETE valid→Terminate; DELETE missing/unknown→Reject(400/404); valid→Proceed | pure (session.rs) | REQ-007/008/009/010 |
| registry: assign×8 ok → 9th Err(Full), 8 still valid; terminate→then unknown; terminate frees a slot | pure | REQ-011 |
| redacted Debug: `SessionRegistry`/handle Debug shows `***`, not ids/bearer | pure | REQ-013 |
| regression: full existing marley_mcp suite green; cap-const; guard-order inspect | REQ-012/014 |
| `cargo mutants --list -f` on auth/secret/session/discovery + kill; gate green | REQ-all floors |

### Risks / decisions
- **REQ-005 mechanism** — set_permissions-if-exists FIRST, then create-with-mode: the new bearer only lands
  after the mode is 0600 (a fresh file is created 0600; a pre-existing one is chmod'd before truncate+write).
- **Session gate on the GET too** — a GET (non-initialize) now requires a valid session (Proceed/400/404),
  matching the MCP lifecycle; masked-wiring change, behavior-verified by inspect (no headless listener).
- **`Drop for McpHost`** — the app's orderly quit drops the host; abnormal exit leaving a stale file is
  out of scope (next start overwrites 0600 with a fresh bearer — the lingering content is a dead credential).
- **enum-return body mutants** (`SessionDecision`/`EntropyError`) — no `Default` derive → `Default::default()`
  body mutant UNVIABLE (the #203 case); real mutants = the arm/guard swaps, killed by exhaustive units.

## Phase 3 — Implement
- **Built to the manifest.** PURE: `auth.rs` (`ct_eq` + `bearer_ok` rewritten through it); NEW `secret.rs`
  (`hex128` + `mint_secret` + `EntropyError`); NEW `session.rs` (`SessionRegistry` [cap 8 reject-new,
  redacted Debug] + `SessionDecision` + `session_decision`); NEW `discovery.rs` (COVERED — `write/remove_
  discovery_file_in` with 0600). `lib.rs` mods + re-exports. MASKED: `transport.rs` (`read_entropy`, `spawn`
  mints via `mint_secret(read_entropy())` + drops `salt`, `ServerData.sessions`, session-header parse,
  `serve_connection` session gate [Initialize→mint+assign / Proceed / Terminate / Reject], `Mcp-Session-Id`
  response header, `status_reason`; deleted `derive_bearer`); `mcp_host.rs` (`start(grants)` drops salt,
  `discovery::`-based write, NEW `Drop` removes the file); `app.rs` (deleted the salt block).
- **Deviations:** none. The session gate slots after the bearer guard (order: cap → Origin → bearer →
  session → dispatch); the GET SSE path now also requires a valid session (correct MCP lifecycle) — reached
  only for a Proceed. No existing client of the expose server exists (manager seat is future), so the new
  session requirement breaks nothing; the #370 fake-transport units call `handle_message` directly (bypass
  the gate) → unaffected.
- **Compile + run:** `cargo check -p marley_mcp` + `-p marley --tests` clean; marley_mcp lib 52/52 (43
  pre-existing + ct_eq + secret 2 + session 3 + discovery 3 = +9). fmt clean. No stale `spawn(salt)`/
  `start(salt,…)` callsites.
- **Floor posture:** auth/secret/session in the covered/mutated lane; discovery.rs COVERED (tempdir mode
  asserts — the file-perm payload); transport.rs/mcp_host.rs/app.rs masked (already coverage-excluded).

## Phase 3.5 — Inspect
Three independent critics (security/entropy · protocol-lifecycle · floors/regression). The core hardening
verified correct (CSPRNG bearer no-weak-fallback, no secret leak, constant-time compare, post-auth session
gate, #370 invariants intact); one BLOCKING gate finding + three real improvements, all FIXED.

| # | Finding | Sev | Verdict | Fix |
|---|---|---|---|---|
| F1 | **discovery.rs's `Err(err) => Err(err)` non-NotFound arm was UNTESTED** — the 3 tests only produced Ok/NotFound, so the `guard→true` mutant SURVIVED (a remove error silently swallowed as Ok, masking a failure to delete a stale-bearer file) → gate:5 MSI < 100 + gate:4 line-40 uncovered = RED. | **HIGH (blocked the gate)** | REAL | Added `remove_propagates_non_notfound_error` (a directory at the file name → IsADirectory/PermissionDenied ≠ NotFound must propagate). discovery.rs now 5/5 mutants caught. |
| F2 | **Session slots were reclaimed ONLY by explicit DELETE** — no TTL, no disconnect-reap → 8 cumulative reconnects (the MCP-permitted + Marley-#373-idiom re-initialize-on-reconnect) wedge the global cap → 503 forever until app restart, defeating the self-healing. | MED | REAL | Reap a session when its standing GET stream drops (thread the id into `serve_sse_stream`, `terminate` on hang-up) — ties session lifetime to the client's presence (the natural MCP model), so a reconnect frees its old slot; the cap self-heals instead of wedging. |
| F3 | **A non-POST body claiming `initialize` minted an un-echoed (orphaned) session** — `session_decision` returned Initialize for any non-DELETE. | LOW | REAL | Gate `Initialize` on `http_method == "POST"` (an initialize's response carries the new id, so it must be a POST); a non-POST falls through to the valid-session requirement. + a test. |
| F4 | **discovery-write TOCTOU/symlink window** — `path.exists()` → `open(create)` (not `create_new`) + path-based `set_permissions` could follow a planted symlink / open a raced-in 0644 file (same-user threat, outside the loopback model). | LOW (defense-in-depth) | REAL | Dropped the `exists()` pre-check; fchmod the OPEN HANDLE (`file.set_permissions`) before the write — acts on the fd we hold (symlink/TOCTOU-immune), corrects a pre-existing looser mode, bearer lands only at 0600. |

Confirmed sound (no change): bearer ALWAYS `mint_secret(read_entropy())`, `derive_bearer`/`salt` grep-clean, entropy-failure → refuse-to-start → app stays up (no panic); no `println!/log/tracing` anywhere, `ServerData`/`SessionRegistry`/`ServerHandle` Debug all redact secrets; ct_eq no early content exit (first/mid/last-byte + length all killed); session gate strictly post-Origin+bearer, body-cap still pre-auth-first (order cap→Origin→bearer→session→dispatch); an unauth DELETE can't terminate (403 first); the cap DoS surface is an authenticated same-user client (intended boundary); 404-vs-400 matches the spec + the #372/#373 client's `classify_http_failure` 404→SessionExpired→re-initialize; the lock is never held across a socket write. INFO (accepted): the per-initialize entropy read runs under the session lock (macOS `/dev/urandom` can't block — benign); `status_reason`'s `_ => "Error"` is masked totality (dead-by-construction, acceptable).
- **Post-fix:** 53 marley_mcp lib tests green; discovery 5/5 mutants caught.

## Phase 4 — Validate
- **Tests RUN (pure seams):** auth (ct_eq first/mid/last-byte + length + `Some("")` bypass; bearer_ok
  regression), secret (hex128 charset/len/zero/distinct; mint Ok/Err), session (cap-8 reject-no-evict +
  is_empty; the 8-arm gate incl. non-POST-initialize; redacted Debug), discovery (fresh 0600; pre-existing
  0644→corrected; remove idempotent; **non-NotFound propagates**). `cargo test -p marley_mcp`: **53 lib +
  0 doctest pass**; `-p marley` unaffected.
- **Mutants (`cargo mutants --list -f` on the ACTUAL files):** auth.rs 21 caught, secret.rs 5, session.rs
  19 caught + 1 unviable (`session_decision → Default::default()`, no Default derive — #203), discovery.rs
  5/5 (the F1 gap closed). All viable mutants killed by the units.
- **Full gate `scripts/gates.sh --diff`: GATE GREEN [diff] — 15/15** (coverage ≥100% on
  auth/secret/session/discovery, mutation MSI ≥100%, clippy -D warnings [fixed `io::Error::other`],
  machete/gitleaks/audit/deny/miri/visual green). Receipt written.
- **No live-app drive:** the server is masked (transport/host, coverage-excluded, the #370 fake-transport +
  live-wire posture); no UI surface (§7 N/A). The pure seams (entropy formatter, ct_eq, registry+gate,
  discovery IO) carry the behavior at cov/MSI 100; the masked wiring is inspect-verified.
- **Pre-existing:** none touched; the #370 invariants (loopback bind, Origin, pre-auth body cap, bearer
  pre-dispatch) confirmed intact by the regression critic.

## Phase 5 — Complete
- **CHANGELOG:** entry under Added (M23.5 ④, the three hardening clusters).
- **Architecture docs:** `orchestration-shell.md` §10 auth-credentials bullet updated (CSPRNG bearer +
  0600 discovery + session lifecycle + the guard order).
- **Knowledge captured (forge):** failure `BF-375-session-cap-no-reclamation` (F2, MED); prevention rule
  `PR-claude-a-bounded-pool-needs-a-reclamation-path-not-just-a-cap-001` (HIGH — a bounded pool needs a
  reclamation path, not just a cap; litmus: does the pool recover if N clients vanish without a clean
  release?). The blocking F1 (untested non-NotFound arm) is a coverage-completeness lesson (a `.ok()`/
  catch-all error arm needs a non-happy-path test). AAR `704f2b42` submitted.
- **Ticket #375 closed + pipeline archived to completed/.**
