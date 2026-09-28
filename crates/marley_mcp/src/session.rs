//! PURE — the `Mcp-Session-Id` lifecycle (#375, cluster C; MCP spec 2025-06-18 "Streamable HTTP → Session
//! Management"). A BOUNDED registry of live session ids + the pre-dispatch gate that maps a request's
//! (method, is-initialize, session header) to an action. The masked transport mints ids (via `secret`) and
//! plumbs the header; every DECISION is here, at cov/MSI 100.

use std::fmt;

use serde_json::Value;

use crate::auth::ct_eq;
use crate::clients::Principal;

/// The maximum number of concurrent sessions.
///
/// A local-DoS bound (the #370 cap-before-alloc lesson applied to sessions), NOT an LRU cache — at
/// the cap a new session is REFUSED, never evicting a live one (D4). Since #491 each Claude Code
/// session's bridge holds one, and a bridge that ends without closing its session keeps it until
/// the TTL, so the bound leaves room for a desk of agents.
pub const SESSION_CAP: usize = 32;

/// The most sessions one outside client holds (#524), so no client uses up [`SESSION_CAP`].
///
/// A client opens no standing stream, whose hang-up is how Marley's own sessions are reaped
/// (BF-375), so a client's next `initialize` at this bound ends that client's own least recently
/// used session instead of being refused: a client that crashed four times is not locked out for
/// half an hour. No other principal's session is ever ended for it.
pub const CLIENT_SESSION_CAP: usize = 4;

/// How long a session may sit IDLE (no validated use) before the sweep expires it (#379).
///
/// Closes the #375 wedge: a client that initializes but never opens a stream and never returns
/// would otherwise hold its slot until restart. 30 minutes — generous against every legitimate
/// cadence (the fleet pump re-reads on every notification; even a quiet standing stream
/// re-validates on reconnect), tiny against a wedge that previously lasted forever. Expiry ≠
/// eviction: D4 stands (a LIVE session is never evicted for capacity; only IDLE-past-TTL ones age
/// out).
pub const SESSION_TTL_MS: u64 = 1_800_000;

/// The most characters of a client's name a session keeps (#571).
pub const MAX_CLIENT_NAME: usize = 64;

/// The name an `initialize` request's `params` give their client (`clientInfo.name`, #571).
///
/// It keeps [`MAX_CLIENT_NAME`] printable ASCII characters. The name is the client's own word, a
/// courtesy that sorts callers, never an authority.
#[must_use]
pub fn client_name_of(params: &Value) -> Option<String> {
    let name: String = params
        .get("clientInfo")?
        .get("name")?
        .as_str()?
        .chars()
        .filter(|character| character.is_ascii_graphic() || *character == ' ')
        .take(MAX_CLIENT_NAME)
        .collect();
    let name = name.trim();
    (!name.is_empty()).then(|| name.to_string())
}

/// A new session could not be admitted — the registry is at [`SESSION_CAP`]. The caller refuses the
/// `initialize` (never evicts an existing session — D4).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SessionFull;

/// One live session: its id, the principal that opened it (#524), the epoch-millis of its last
/// validated use (#379 — the idle clock), and the name its client gave (#571).
#[derive(Clone, PartialEq, Eq)]
struct SessionEntry {
    id: String,
    owner: Principal,
    last_seen_ms: u64,
    client: Option<String>,
}

/// Whether `a` and `b` are the same principal for a session: Marley, or the client of one name.
fn same_owner(a: &Principal, b: &Principal) -> bool {
    a.client_name() == b.client_name()
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
    /// Admit `id` as a live session of `owner`, stamped last-seen `now` (#379). A client at
    /// [`CLIENT_SESSION_CAP`] first loses its own least recently used session (#524).
    ///
    /// # Errors
    ///
    /// `Err(SessionFull)` when already at [`SESSION_CAP`] (reject-new, never evict — D4).
    pub fn assign(&mut self, id: String, owner: Principal, now: u64) -> Result<(), SessionFull> {
        if owner.client_name().is_some() {
            let held = self
                .entries
                .iter()
                .filter(|stored| same_owner(&stored.owner, &owner))
                .count();
            // `min_by_key` keeps the first of equal stamps, so a tie ends the earliest opened.
            let oldest = self
                .entries
                .iter()
                .enumerate()
                .filter(|(_, stored)| same_owner(&stored.owner, &owner))
                .min_by_key(|(_, stored)| stored.last_seen_ms)
                .map(|(index, _)| index);
            if held >= CLIENT_SESSION_CAP
                && let Some(oldest) = oldest
            {
                let _ended = self.entries.remove(oldest);
            }
        }
        if self.entries.len() >= SESSION_CAP {
            return Err(SessionFull);
        }
        self.entries.push(SessionEntry {
            id,
            owner,
            last_seen_ms: now,
            client: None,
        });
        Ok(())
    }

    /// Keeps `client` as the name the client of session `id` gave at `initialize` (#571).
    /// Constant-time over all entries, as [`Self::touch`] is.
    pub fn name_client(&mut self, id: &str, client: Option<&str>) {
        for stored in &mut self.entries {
            if ct_eq(&stored.id, id) {
                stored.client = client.map(str::to_string);
            }
        }
    }

    /// The name the client of session `id` gave at `initialize`, if it gave one (#571).
    /// Constant-time over all entries, as [`Self::validate`] is.
    #[must_use]
    pub fn client_of(&self, id: &str) -> Option<String> {
        let mut client = None;
        for stored in &self.entries {
            if ct_eq(&stored.id, id) {
                client.clone_from(&stored.client);
            }
        }
        client
    }

    /// Whether `id` is a live session of `owner`: another principal's session is unknown to it
    /// (#524). Constant-time over ALL entries (no short-circuit on a match — D8), so the compare
    /// doesn't leak which/whether an id matched.
    #[must_use]
    pub fn validate(&self, id: &str, owner: &Principal) -> bool {
        let mut found = false;
        for stored in &self.entries {
            let matches = ct_eq(&stored.id, id);
            found |= matches && same_owner(&stored.owner, owner);
        }
        found
    }

    /// Refresh `id`'s last-seen to `now` (#379) — called by the gate on every validated use, so only a
    /// genuinely IDLE session ages out. Constant-time shape over all entries (no early exit — D8); the
    /// per-entry branch is on the `ct_eq` RESULT (content-CT preserved), runs only post-`Proceed` (the
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

    /// Terminate `owner`'s session `id`; returns whether it was live (idempotent — a second terminate
    /// returns `false`).
    pub fn terminate(&mut self, id: &str, owner: &Principal) -> bool {
        let before = self.entries.len();
        self.entries
            .retain(|stored| !(ct_eq(&stored.id, id) && same_owner(&stored.owner, owner)));
        self.entries.len() != before
    }

    /// Terminate every session of the client `name` (its cut-off, #524); how many there were.
    pub fn terminate_owned_by(&mut self, name: &str) -> usize {
        let before = self.entries.len();
        self.entries
            .retain(|stored| stored.owner.client_name() != Some(name));
        before - self.entries.len()
    }

    /// The number of live sessions.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether the registry holds no sessions.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
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

/// Decide the session action for a request (MCP spec §Session Management).
///
/// `is_initialize` is whether the JSON-RPC body is an `initialize` request; `session_header` is the
/// presented `Mcp-Session-Id` (already parsed); `owner` holds the request's bearer, and only its
/// own sessions are valid for it (#524). Ordering: DELETE is handled first
/// (terminate/refuse), then a POST `initialize` (assign), then every other request requires a valid
/// session (missing → 400, unknown → 404). An `initialize` is a POST (its response header carries
/// the new id) — a non-POST body claiming `initialize` is NOT treated as one (it would mint a
/// session the response could never echo, orphaning a slot), so it falls through to the
/// valid-session requirement.
#[must_use]
pub fn session_decision(
    http_method: &str,
    is_initialize: bool,
    session_header: Option<&str>,
    owner: &Principal,
    registry: &SessionRegistry,
) -> SessionDecision {
    if http_method == "DELETE" {
        return match session_header {
            None => SessionDecision::Reject(400),
            Some(id) if registry.validate(id, owner) => SessionDecision::Terminate,
            Some(_) => SessionDecision::Reject(404),
        };
    }
    if is_initialize && http_method == "POST" {
        return SessionDecision::Initialize;
    }
    match session_header {
        None => SessionDecision::Reject(400),
        Some(id) if registry.validate(id, owner) => SessionDecision::Proceed,
        Some(_) => SessionDecision::Reject(404),
    }
}

/// The #379 gate ORCHESTRATOR the transport calls.
///
/// It sweeps the idle sessions, decides, and touches on a validated use — one seam, so
/// `session_decision`/`validate` keep their #375 signatures (and their call sites) untouched.
///
/// Sweep-BEFORE-decide is what makes "expired ≡ unknown" true by construction: an expired id is
/// simply ABSENT when the gate runs, so the SHIPPED 404 arm answers — no new decision arm, no wire
/// change, nothing for a client to distinguish (#373's reconnect already recovers via
/// `SessionExpired`). The touch fires only on `Proceed` (a `Terminate` removes the entry anyway; an
/// `Initialize` stamps its own fresh entry via `assign`).
pub fn session_gate(
    http_method: &str,
    is_initialize: bool,
    session_header: Option<&str>,
    owner: &Principal,
    registry: &mut SessionRegistry,
    now: u64,
) -> SessionDecision {
    let _swept = registry.sweep(now);
    let decision = session_decision(http_method, is_initialize, session_header, owner, registry);
    if decision == SessionDecision::Proceed
        && let Some(id) = session_header
    {
        registry.touch(id, now);
    }
    decision
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARLEY: Principal = Principal::Marley;

    // REQ-011 — the cap is reject-new (never evict): 8 assigns succeed, the 9th is refused, and all 8
    // stay valid; a terminate frees exactly one slot.
    #[test]
    fn registry_caps_at_eight_and_rejects_without_eviction() {
        let mut reg = SessionRegistry::default();
        assert!(reg.is_empty()); // fresh → empty
        for i in 0..SESSION_CAP {
            assert_eq!(reg.assign(format!("id-{i}"), MARLEY, 0), Ok(()));
        }
        assert!(!reg.is_empty()); // after assigns → not empty
        assert_eq!(reg.len(), SESSION_CAP);
        // the 9th is refused — typed error, no eviction.
        assert_eq!(
            reg.assign("id-overflow".to_string(), MARLEY, 0),
            Err(SessionFull)
        );
        for i in 0..SESSION_CAP {
            assert!(
                reg.validate(&format!("id-{i}"), &MARLEY),
                "existing session {i} survives the refusal"
            );
        }
        assert!(!reg.validate("id-overflow", &MARLEY));
        // terminating one frees a slot for a new assign.
        assert!(reg.terminate("id-0", &MARLEY));
        assert!(!reg.validate("id-0", &MARLEY));
        assert!(!reg.terminate("id-0", &MARLEY)); // idempotent
        assert_eq!(reg.assign("id-new".to_string(), MARLEY, 0), Ok(()));
        assert!(reg.validate("id-new", &MARLEY));
    }

    // REQ-007/008/009/010 — the gate arms.
    #[test]
    fn session_decision_covers_every_arm() {
        let mut reg = SessionRegistry::default();
        reg.assign("live".to_string(), MARLEY, 0).unwrap();

        // a POST initialize → assign a fresh session (no prior id required).
        assert_eq!(
            session_decision("POST", true, None, &MARLEY, &reg),
            SessionDecision::Initialize
        );
        // a NON-POST body claiming initialize is NOT an Initialize — it falls through to the valid-session
        // requirement (here: no header → 400), so it can't mint a session the response would never echo.
        assert_eq!(
            session_decision("GET", true, None, &MARLEY, &reg),
            SessionDecision::Reject(400)
        );
        // a non-initialize request with a valid id → dispatch.
        assert_eq!(
            session_decision("POST", false, Some("live"), &MARLEY, &reg),
            SessionDecision::Proceed
        );
        // GET (non-initialize) with a valid id → dispatch (the standing stream requires a session too).
        assert_eq!(
            session_decision("GET", false, Some("live"), &MARLEY, &reg),
            SessionDecision::Proceed
        );
        // missing id on a non-initialize → 400.
        assert_eq!(
            session_decision("POST", false, None, &MARLEY, &reg),
            SessionDecision::Reject(400)
        );
        // unknown/terminated id → 404 (client re-initializes).
        assert_eq!(
            session_decision("POST", false, Some("stale"), &MARLEY, &reg),
            SessionDecision::Reject(404)
        );
        // DELETE with a valid id → terminate.
        assert_eq!(
            session_decision("DELETE", false, Some("live"), &MARLEY, &reg),
            SessionDecision::Terminate
        );
        // DELETE with a missing id → 400; with an unknown id → 404.
        assert_eq!(
            session_decision("DELETE", false, None, &MARLEY, &reg),
            SessionDecision::Reject(400)
        );
        assert_eq!(
            session_decision("DELETE", false, Some("stale"), &MARLEY, &reg),
            SessionDecision::Reject(404)
        );
    }

    // #379 REQ-001 — the sweep expires at the exact boundary: idle for TTL-1 survives, idle for
    // TTL expires; a swept id then gets the SHIPPED 404 through the gate.
    #[test]
    fn t379_req001_sweep_expires_at_the_boundary() {
        let mut reg = SessionRegistry::default();
        reg.assign("s".to_string(), MARLEY, 1_000).unwrap();
        assert_eq!(
            reg.sweep(1_000 + SESSION_TTL_MS - 1),
            0,
            "one ms fresh survives"
        );
        assert!(reg.validate("s", &MARLEY));
        assert_eq!(
            reg.sweep(1_000 + SESSION_TTL_MS),
            1,
            "the boundary ages out"
        );
        assert!(!reg.validate("s", &MARLEY));
        assert_eq!(
            session_gate(
                "POST",
                false,
                Some("s"),
                &MARLEY,
                &mut reg,
                1_000 + SESSION_TTL_MS
            ),
            SessionDecision::Reject(404),
            "a swept id answers with the SHIPPED 404 — the client re-initializes"
        );
    }

    // #379 REQ-002 — a validated use refreshes last-seen: the gate's Proceed touch keeps a session
    // alive past its ORIGINAL expiry; it then expires from the TOUCHED stamp (both directions).
    #[test]
    fn t379_req002_gate_touch_refreshes_last_seen() {
        let mut reg = SessionRegistry::default();
        reg.assign("s".to_string(), MARLEY, 0).unwrap();
        // a validated use at t=10_000 (the gate touches on Proceed)…
        assert_eq!(
            session_gate("POST", false, Some("s"), &MARLEY, &mut reg, 10_000),
            SessionDecision::Proceed
        );
        // …so the ORIGINAL expiry point no longer kills it (fatal WITHOUT the touch)…
        assert_eq!(reg.sweep(SESSION_TTL_MS), 0, "the touch moved the clock");
        assert!(reg.validate("s", &MARLEY));
        // …and it expires from the TOUCHED stamp.
        assert_eq!(reg.sweep(10_000 + SESSION_TTL_MS), 1);
        assert!(!reg.validate("s", &MARLEY));
    }

    // #379 REQ-003 — the wedge, end-to-end: a registry FULL of idle sessions frees at the next
    // gate pass and a fresh initialize succeeds.
    #[test]
    fn t379_req003_full_of_idle_frees_for_a_new_initialize() {
        let mut reg = SessionRegistry::default();
        for i in 0..SESSION_CAP {
            reg.assign(format!("idle-{i}"), MARLEY, 0).unwrap();
        }
        assert_eq!(
            reg.assign("blocked".to_string(), MARLEY, 0),
            Err(SessionFull)
        );
        // the next request's gate sweeps the idle 8 and admits the initialize.
        assert_eq!(
            session_gate("POST", true, None, &MARLEY, &mut reg, SESSION_TTL_MS),
            SessionDecision::Initialize
        );
        assert!(reg.is_empty(), "the sweep freed every idle slot");
        assert_eq!(
            reg.assign("fresh".to_string(), MARLEY, SESSION_TTL_MS),
            Ok(())
        );
    }

    // #379 REQ-004 — the #375 invariants hold: FRESH sessions are never swept (the gate removes
    // nothing), the cap still refuses the 9th, terminate-of-a-swept-id is an idempotent false, and
    // a future-stamped entry (clock skew) reads fresh rather than underflowing.
    #[test]
    fn t379_req004_fresh_sessions_and_invariants_unaffected() {
        let mut reg = SessionRegistry::default();
        for i in 0..SESSION_CAP {
            reg.assign(format!("fresh-{i}"), MARLEY, 5_000).unwrap();
        }
        assert_eq!(
            session_gate("POST", false, Some("fresh-0"), &MARLEY, &mut reg, 6_000),
            SessionDecision::Proceed
        );
        assert_eq!(reg.len(), SESSION_CAP, "no fresh session was swept");
        assert_eq!(
            reg.assign("ninth".to_string(), MARLEY, 6_000),
            Err(SessionFull)
        );
        // a swept id terminates as false (idempotence over absence): the sweep at 5_000+TTL ages
        // out the seven UNtouched seats; fresh-0 (touched at 6_000 by the gate above) survives.
        assert_eq!(reg.sweep(5_000 + SESSION_TTL_MS), SESSION_CAP - 1);
        assert!(
            reg.validate("fresh-0", &MARLEY),
            "the touched seat survives"
        );
        assert!(
            !reg.terminate("fresh-1", &MARLEY),
            "a swept id terminates as false"
        );
        // clock skew: a stamp in the FUTURE never underflows — it reads fresh.
        reg.assign("future".to_string(), MARLEY, u64::MAX).unwrap();
        assert_eq!(reg.sweep(0), 0);
        assert!(reg.validate("future", &MARLEY));
    }

    // #379 REQ-006 — expired ≡ unknown at the wire: the gate outcome for a swept id VALUE-EQUALS
    // the outcome for a never-assigned id (no new arm, nothing to distinguish).
    #[test]
    fn t379_req006_expired_is_indistinguishable_from_unknown() {
        let mut swept = SessionRegistry::default();
        swept.assign("was-live".to_string(), MARLEY, 0).unwrap();
        let outcome_swept = session_gate(
            "POST",
            false,
            Some("was-live"),
            &MARLEY,
            &mut swept,
            SESSION_TTL_MS,
        );

        let mut never = SessionRegistry::default();
        let outcome_unknown = session_gate(
            "POST",
            false,
            Some("was-live"),
            &MARLEY,
            &mut never,
            SESSION_TTL_MS,
        );

        assert_eq!(outcome_swept, outcome_unknown, "value-equal refusals");
        assert_eq!(outcome_swept, SessionDecision::Reject(404));
    }

    // REQ-013 — the registry's Debug redacts the session ids (a derived Debug would leak them).
    #[test]
    fn debug_redacts_session_ids() {
        let mut reg = SessionRegistry::default();
        reg.assign("super-secret-session".to_string(), MARLEY, 0)
            .unwrap();
        let rendered = format!("{reg:?}");
        assert!(rendered.contains("***"), "not redacted: {rendered}");
        assert!(
            !rendered.contains("super-secret-session"),
            "id leaked: {rendered}"
        );
        assert!(rendered.contains("live: 1")); // the count is fine to show
    }
}
