//! PURE — the `Mcp-Session-Id` lifecycle (#375, cluster C; MCP spec 2025-06-18 "Streamable HTTP → Session
//! Management"). A BOUNDED registry of live session ids + the pre-dispatch gate that maps a request's
//! (method, is-initialize, session header) to an action. The masked transport mints ids (via `secret`) and
//! plumbs the header; every DECISION is here, at cov/MSI 100.

use std::fmt;

use crate::auth::ct_eq;

/// The maximum number of concurrent sessions. A local-DoS bound (the #370 cap-before-alloc lesson applied
/// to sessions), NOT an LRU cache — at the cap a new session is REFUSED, never evicting a live one (D4).
pub const SESSION_CAP: usize = 8;

/// How long a session may sit IDLE (no validated use) before the sweep expires it (#379). Closes the
/// #375 wedge: a client that initializes but never opens a stream and never returns would otherwise
/// hold its slot until restart. 30 minutes — generous against every legitimate cadence (the fleet
/// pump re-reads on every notification; even a quiet standing stream re-validates on reconnect), tiny
/// against a wedge that previously lasted forever. Expiry ≠ eviction: D4 stands (a LIVE session is
/// never evicted for capacity; only IDLE-past-TTL ones age out).
pub const SESSION_TTL_MS: u64 = 1_800_000;

/// A new session could not be admitted — the registry is at [`SESSION_CAP`]. The caller refuses the
/// `initialize` (never evicts an existing session — D4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFull;

/// One live session: its id + the epoch-millis of its last validated use (#379 — the idle clock).
#[derive(Clone, PartialEq, Eq)]
struct SessionEntry {
    id: String,
    last_seen_ms: u64,
}

/// The live session ids. Ids are secrets-adjacent (minted from the same entropy as the bearer), so the
/// manual [`Debug`] redacts them — a derived `Debug` would leak them into a log (D8/REQ-013).
#[derive(Default, Clone, PartialEq, Eq)]
pub struct SessionRegistry {
    entries: Vec<SessionEntry>,
}

impl fmt::Debug for SessionRegistry {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // The COUNT is fine to show; the ids are not.
        f.debug_struct("SessionRegistry")
            .field("live", &self.entries.len())
            .field("ids", &"***")
            .finish()
    }
}

impl SessionRegistry {
    /// Admit `id` as a live session stamped last-seen `now` (#379), or `Err(SessionFull)` when already
    /// at [`SESSION_CAP`] (reject-new, never evict — D4).
    pub fn assign(&mut self, id: String, now: u64) -> Result<(), SessionFull> {
        if self.entries.len() >= SESSION_CAP {
            return Err(SessionFull);
        }
        self.entries.push(SessionEntry {
            id,
            last_seen_ms: now,
        });
        Ok(())
    }

    /// Whether `id` is a live session. Constant-time over ALL entries (no short-circuit on a match — D8),
    /// so the compare doesn't leak which/whether an id matched.
    pub fn validate(&self, id: &str) -> bool {
        let mut found = false;
        for stored in &self.entries {
            found |= ct_eq(&stored.id, id);
        }
        found
    }

    /// Refresh `id`'s last-seen to `now` (#379) — called by the gate on every validated use, so only a
    /// genuinely IDLE session ages out. Constant-time shape over all entries (no early exit — D8); the
    /// per-entry branch is on the ct_eq RESULT (content-CT preserved), runs only post-`Proceed` (the
    /// caller already holds a valid id), and the only signal is the caller's OWN slot position — not a
    /// secret; mirrors `terminate`'s retain shape.
    pub fn touch(&mut self, id: &str, now: u64) {
        for stored in &mut self.entries {
            if ct_eq(&stored.id, id) {
                stored.last_seen_ms = now;
            }
        }
    }

    /// Expire every session idle for [`SESSION_TTL_MS`] or longer at `now` (#379), returning how many
    /// were removed. `now - last_seen >= TTL` expires (the boundary ages out; one millisecond fresher
    /// survives); a stamp in the future (clock skew) never underflows (`saturating_sub` reads as fresh).
    pub fn sweep(&mut self, now: u64) -> usize {
        let before = self.entries.len();
        self.entries
            .retain(|stored| now.saturating_sub(stored.last_seen_ms) < SESSION_TTL_MS);
        before - self.entries.len()
    }

    /// Terminate `id`; returns whether it was live (idempotent — a second terminate returns `false`).
    pub fn terminate(&mut self, id: &str) -> bool {
        let before = self.entries.len();
        self.entries.retain(|stored| !ct_eq(&stored.id, id));
        self.entries.len() != before
    }

    /// The number of live sessions.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry holds no sessions.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

/// What the pre-dispatch session gate decides for a request (after the bearer/Origin guards pass).
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SessionDecision {
    /// An `initialize` — mint + assign a fresh session id, dispatch, echo the id on the response.
    Initialize,
    /// A request carrying a valid session id — dispatch normally.
    Proceed,
    /// A `DELETE` carrying a valid session id — terminate it, respond success.
    Terminate,
    /// Refuse with this HTTP status, no dispatch: `400` (a non-initialize request missing the id) or
    /// `404` (an unknown/terminated id — the client must re-initialize).
    Reject(u16),
}

/// Decide the session action for a request (MCP spec §Session Management). `is_initialize` is whether the
/// JSON-RPC body is an `initialize` request; `session_header` is the presented `Mcp-Session-Id` (already
/// parsed). Ordering: DELETE is handled first (terminate/refuse), then a POST `initialize` (assign), then
/// every other request requires a valid session (missing → 400, unknown → 404). An `initialize` is a POST
/// (its response header carries the new id) — a non-POST body claiming `initialize` is NOT treated as one
/// (it would mint a session the response could never echo, orphaning a slot), so it falls through to the
/// valid-session requirement.
pub fn session_decision(
    http_method: &str,
    is_initialize: bool,
    session_header: Option<&str>,
    registry: &SessionRegistry,
) -> SessionDecision {
    if http_method == "DELETE" {
        return match session_header {
            None => SessionDecision::Reject(400),
            Some(id) if registry.validate(id) => SessionDecision::Terminate,
            Some(_) => SessionDecision::Reject(404),
        };
    }
    if is_initialize && http_method == "POST" {
        return SessionDecision::Initialize;
    }
    match session_header {
        None => SessionDecision::Reject(400),
        Some(id) if registry.validate(id) => SessionDecision::Proceed,
        Some(_) => SessionDecision::Reject(404),
    }
}

/// The #379 gate ORCHESTRATOR the transport calls: sweep the idle sessions, decide, and touch on a
/// validated use — one seam, so `session_decision`/`validate` keep their #375 signatures (and their
/// call sites) untouched. Sweep-BEFORE-decide is what makes "expired ≡ unknown" true by construction:
/// an expired id is simply ABSENT when the gate runs, so the SHIPPED 404 arm answers — no new
/// decision arm, no wire change, nothing for a client to distinguish (#373's reconnect already
/// recovers via `SessionExpired`). The touch fires only on `Proceed` (a `Terminate` removes the
/// entry anyway; an `Initialize` stamps its own fresh entry via `assign`).
pub fn session_gate(
    http_method: &str,
    is_initialize: bool,
    session_header: Option<&str>,
    registry: &mut SessionRegistry,
    now: u64,
) -> SessionDecision {
    registry.sweep(now);
    let decision = session_decision(http_method, is_initialize, session_header, registry);
    if decision == SessionDecision::Proceed {
        if let Some(id) = session_header {
            registry.touch(id, now);
        }
    }
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    // REQ-011 — the cap is reject-new (never evict): 8 assigns succeed, the 9th is refused, and all 8
    // stay valid; a terminate frees exactly one slot.
    #[test]
    fn registry_caps_at_eight_and_rejects_without_eviction() {
        let mut reg = SessionRegistry::default();
        assert!(reg.is_empty()); // fresh → empty
        for i in 0..SESSION_CAP {
            assert_eq!(reg.assign(format!("id-{i}"), 0), Ok(()));
        }
        assert!(!reg.is_empty()); // after assigns → not empty
        assert_eq!(reg.len(), SESSION_CAP);
        // the 9th is refused — typed error, no eviction.
        assert_eq!(reg.assign("id-overflow".to_string(), 0), Err(SessionFull));
        for i in 0..SESSION_CAP {
            assert!(
                reg.validate(&format!("id-{i}")),
                "existing session {i} survives the refusal"
            );
        }
        assert!(!reg.validate("id-overflow"));
        // terminating one frees a slot for a new assign.
        assert!(reg.terminate("id-0"));
        assert!(!reg.validate("id-0"));
        assert!(!reg.terminate("id-0")); // idempotent
        assert_eq!(reg.assign("id-new".to_string(), 0), Ok(()));
        assert!(reg.validate("id-new"));
    }

    // REQ-007/008/009/010 — the gate arms.
    #[test]
    fn session_decision_covers_every_arm() {
        let mut reg = SessionRegistry::default();
        reg.assign("live".to_string(), 0).unwrap();

        // a POST initialize → assign a fresh session (no prior id required).
        assert_eq!(
            session_decision("POST", true, None, &reg),
            SessionDecision::Initialize
        );
        // a NON-POST body claiming initialize is NOT an Initialize — it falls through to the valid-session
        // requirement (here: no header → 400), so it can't mint a session the response would never echo.
        assert_eq!(
            session_decision("GET", true, None, &reg),
            SessionDecision::Reject(400)
        );
        // a non-initialize request with a valid id → dispatch.
        assert_eq!(
            session_decision("POST", false, Some("live"), &reg),
            SessionDecision::Proceed
        );
        // GET (non-initialize) with a valid id → dispatch (the standing stream requires a session too).
        assert_eq!(
            session_decision("GET", false, Some("live"), &reg),
            SessionDecision::Proceed
        );
        // missing id on a non-initialize → 400.
        assert_eq!(
            session_decision("POST", false, None, &reg),
            SessionDecision::Reject(400)
        );
        // unknown/terminated id → 404 (client re-initializes).
        assert_eq!(
            session_decision("POST", false, Some("stale"), &reg),
            SessionDecision::Reject(404)
        );
        // DELETE with a valid id → terminate.
        assert_eq!(
            session_decision("DELETE", false, Some("live"), &reg),
            SessionDecision::Terminate
        );
        // DELETE with a missing id → 400; with an unknown id → 404.
        assert_eq!(
            session_decision("DELETE", false, None, &reg),
            SessionDecision::Reject(400)
        );
        assert_eq!(
            session_decision("DELETE", false, Some("stale"), &reg),
            SessionDecision::Reject(404)
        );
    }

    // #379 REQ-001 — the sweep expires at the exact boundary: idle for TTL-1 survives, idle for
    // TTL expires; a swept id then gets the SHIPPED 404 through the gate.
    #[test]
    fn t379_req001_sweep_expires_at_the_boundary() {
        let mut reg = SessionRegistry::default();
        reg.assign("s".to_string(), 1_000).unwrap();
        assert_eq!(
            reg.sweep(1_000 + SESSION_TTL_MS - 1),
            0,
            "one ms fresh survives"
        );
        assert!(reg.validate("s"));
        assert_eq!(
            reg.sweep(1_000 + SESSION_TTL_MS),
            1,
            "the boundary ages out"
        );
        assert!(!reg.validate("s"));
        assert_eq!(
            session_gate("POST", false, Some("s"), &mut reg, 1_000 + SESSION_TTL_MS),
            SessionDecision::Reject(404),
            "a swept id answers with the SHIPPED 404 — the client re-initializes"
        );
    }

    // #379 REQ-002 — a validated use refreshes last-seen: the gate's Proceed touch keeps a session
    // alive past its ORIGINAL expiry; it then expires from the TOUCHED stamp (both directions).
    #[test]
    fn t379_req002_gate_touch_refreshes_last_seen() {
        let mut reg = SessionRegistry::default();
        reg.assign("s".to_string(), 0).unwrap();
        // a validated use at t=10_000 (the gate touches on Proceed)…
        assert_eq!(
            session_gate("POST", false, Some("s"), &mut reg, 10_000),
            SessionDecision::Proceed
        );
        // …so the ORIGINAL expiry point no longer kills it (fatal WITHOUT the touch)…
        assert_eq!(reg.sweep(SESSION_TTL_MS), 0, "the touch moved the clock");
        assert!(reg.validate("s"));
        // …and it expires from the TOUCHED stamp.
        assert_eq!(reg.sweep(10_000 + SESSION_TTL_MS), 1);
        assert!(!reg.validate("s"));
    }

    // #379 REQ-003 — the wedge, end-to-end: a registry FULL of idle sessions frees at the next
    // gate pass and a fresh initialize succeeds.
    #[test]
    fn t379_req003_full_of_idle_frees_for_a_new_initialize() {
        let mut reg = SessionRegistry::default();
        for i in 0..SESSION_CAP {
            reg.assign(format!("idle-{i}"), 0).unwrap();
        }
        assert_eq!(reg.assign("blocked".to_string(), 0), Err(SessionFull));
        // the next request's gate sweeps the idle 8 and admits the initialize.
        assert_eq!(
            session_gate("POST", true, None, &mut reg, SESSION_TTL_MS),
            SessionDecision::Initialize
        );
        assert!(reg.is_empty(), "the sweep freed every idle slot");
        assert_eq!(reg.assign("fresh".to_string(), SESSION_TTL_MS), Ok(()));
    }

    // #379 REQ-004 — the #375 invariants hold: FRESH sessions are never swept (the gate removes
    // nothing), the cap still refuses the 9th, terminate-of-a-swept-id is an idempotent false, and
    // a future-stamped entry (clock skew) reads fresh rather than underflowing.
    #[test]
    fn t379_req004_fresh_sessions_and_invariants_unaffected() {
        let mut reg = SessionRegistry::default();
        for i in 0..SESSION_CAP {
            reg.assign(format!("fresh-{i}"), 5_000).unwrap();
        }
        assert_eq!(
            session_gate("POST", false, Some("fresh-0"), &mut reg, 6_000),
            SessionDecision::Proceed
        );
        assert_eq!(reg.len(), SESSION_CAP, "no fresh session was swept");
        assert_eq!(reg.assign("ninth".to_string(), 6_000), Err(SessionFull));
        // a swept id terminates as false (idempotence over absence): the sweep at 5_000+TTL ages
        // out the seven UNtouched seats; fresh-0 (touched at 6_000 by the gate above) survives.
        assert_eq!(reg.sweep(5_000 + SESSION_TTL_MS), SESSION_CAP - 1);
        assert!(reg.validate("fresh-0"), "the touched seat survives");
        assert!(!reg.terminate("fresh-1"), "a swept id terminates as false");
        // clock skew: a stamp in the FUTURE never underflows — it reads fresh.
        reg.assign("future".to_string(), u64::MAX).unwrap();
        assert_eq!(reg.sweep(0), 0);
        assert!(reg.validate("future"));
    }

    // #379 REQ-006 — expired ≡ unknown at the wire: the gate outcome for a swept id VALUE-EQUALS
    // the outcome for a never-assigned id (no new arm, nothing to distinguish).
    #[test]
    fn t379_req006_expired_is_indistinguishable_from_unknown() {
        let mut swept = SessionRegistry::default();
        swept.assign("was-live".to_string(), 0).unwrap();
        let outcome_swept =
            session_gate("POST", false, Some("was-live"), &mut swept, SESSION_TTL_MS);

        let mut never = SessionRegistry::default();
        let outcome_unknown =
            session_gate("POST", false, Some("was-live"), &mut never, SESSION_TTL_MS);

        assert_eq!(outcome_swept, outcome_unknown, "value-equal refusals");
        assert_eq!(outcome_swept, SessionDecision::Reject(404));
    }

    // REQ-013 — the registry's Debug redacts the session ids (a derived Debug would leak them).
    #[test]
    fn debug_redacts_session_ids() {
        let mut reg = SessionRegistry::default();
        reg.assign("super-secret-session".to_string(), 0).unwrap();
        let rendered = format!("{reg:?}");
        assert!(rendered.contains("***"), "not redacted: {rendered}");
        assert!(
            !rendered.contains("super-secret-session"),
            "id leaked: {rendered}"
        );
        assert!(rendered.contains("live: 1")); // the count is fine to show
    }
}
