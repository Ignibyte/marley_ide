# TICKET-384 — Fleet header "misconfigured" diagnosability state (#376 F8)

- **Forge ticket:** #384 (3215851c-050a-46ff-86b4-ac06bfa8bafa) (feature, M25 — shipped in M28)
- **Owner:** 57761d04-1446-4aab-a30d-884dafb593c6 (agent session, 2026-08-05)
- **AAR:** a7321a75-efff-45b1-b09d-18848f6a72b0 (submitted — completed, 4 novel findings)
- **Pipeline doc:** ../../pipeline/completed/384-fleet-misconfigured-state.spec.md
- **Source ticket:** sprint #36 "M25 — App-Grade QA Hardening" — #376's inspect F8 (accepted-as-
  follow-up: "a 'Fleet · misconfigured' state = new UI surface, not v1") + the QA-2026-07-21 run's
  #381 find (Finder launch → no forge client, fleet writes fail with in-card reasons the header
  never explains). **Hard-ordered AFTER #381** — its `.mcp.json`-resolution fix decides which arm
  fires for the Finder-launch case.
- **Status:** closed
- **React-first:** UI-affecting — prototype the "Fleet · misconfigured" header + clamped reason
  caption in `marley-web` (fleet strip, per the MARLEY-PARITY.md port map) and confirm the look at
  localhost:5173 BEFORE any Rust; then port 1:1. Full section: the queued spec's
  `## React-first (parity)`.

## Summary
Per the locked #376 D5/D6, three distinct failure arms are silent (or mute about their cause) in the
fleet header today: **(a)** no `[[projects.orchestration]]` entry for the restored active root —
legitimately unconfigured, the ONE arm that should stay quiet; **(b)** an entry whose
`brain_endpoint` fails the loopback wall or doesn't parse (`endpoint_for_brain` → `None`) — renders
the same bare "Fleet" as unconfigured; **(c)** no forge client at all (`.mcp.json`
missing/unreadable/unparseable) — fleet writes fail with clamped in-card reasons
("no forge client (.mcp.json)") while the header never says why. This ticket extends the pure
`fleet_live` presentation seam with a **Misconfigured** state: a distinct header
("Fleet · misconfigured") plus a compact, clamped reason naming which arm fired, populated at the
same post-shell-restore boot site #376 established. `ConnectionState` (the crate's wire-liveness
enum) does NOT grow a variant — misconfiguration is an app-side boot determination, not a wire
state. Genuinely-unconfigured roots keep the bare "Fleet" + demo behavior byte-identical.

## Acceptance
On a root with an orchestration entry whose `brain_endpoint` is rejected (arm b) or whose forge
client is absent (arm c), the right-dock header reads "Fleet · misconfigured" and a compact
clamped reason names the arm; a root with no entry (arm a) renders exactly today's bare "Fleet"
with the demo verb intact; "Fleet · live"/"Fleet · reconnecting" behavior is unchanged; the reason
text passes the shipped control-strip/≤200 clamp and never carries bearer material; a live capture
shows the misconfigured header on a misconfigured root. Full EARS criteria live in the pipeline
spec.
