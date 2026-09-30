# `marley_sdk`

> Per-crate architecture note, written 2026-09-30 at #607. Provenance: **`[Marley-original]`**
> (`serde` and `serde_json` only). The design record is
> [fleet-contract.md](../marley/fleet-contract.md), D20 in
> [three-prong-plan.md](../marley/three-prong-plan.md).

The contracts Marley reads to show agents, their work and their hosts. A provider sends data in
these shapes and Marley draws it; no provider sends UI. Pure: serde types and functions of their
inputs, with no clock, no IO and no gpui. `marley_fleet` stays the session envelope the rail and
Marley's MCP tools use; this crate is what a workflow store and a host collector send.

## Modules
- `work`: `marley.work/v1` (`WORK_CONTRACT`), from a central workflow store. `Handshake`
  (`contract`, `provider`, `capabilities`, `poll_s`, `stale_after_s`, and `offers`),
  `AgentList` of `AgentSummary`, `AgentDetail` with `AgentInfo`, `WorkItem`, `Run` (its
  `active_phase` and `failed`), `PhaseRun` with `Gate`s, `Event`, `Usage` of `TokenUse`,
  `Question` and `Changes`.
- `host`: `marley.host/v1` (`HOST_CONTRACT`), from a collector on a host. `HostSnapshot` with
  `HostInfo`, `Cpu`, `Memory`, `Disk`, `Network` and the `AgentProcess`es running there.
- `stale`: `is_stale(last_seen_ms, now_ms, poll_s, stale_after_s)`. An agent reads stale after
  three polls with no sign of it (`STALE_POLLS`), or the handshake's wider `stale_after_s`; one
  with no `last_seen_ms` never does. `DEFAULT_POLL_S` is 5.
- `pseudo`: `Pseudo`, a provider with no store behind it. `Pseudo::new(started_ms)` parses the
  fixtures and `at(now_ms)` returns a `PseudoState` (the list, each detail, the hosts).

## Reading what a provider sends
- No type denies unknown fields, and every optional field takes `default`, so a provider may send
  more than this crate names and leave out what it does not keep.
- `State`, `Attention`, `Outcome`, `PhaseState` and `GateState` fall back to `Unknown` on a
  value this version does not name (`serde(other)`).
- `TokenUse`'s fields are `input`, `output` and `cache_read`, renamed to the wire's
  `input_tokens`, `output_tokens` and `cache_read_tokens`. Tokens only: no money in v1 (Chad,
  2026-09-30).

## The pseudo provider
- `fixtures/handshake.json`, `details.json` and `hosts.json` are the contract document's
  examples: build-1 working on RB-142, review-1 (Codex) waiting on question q-19 about RB-139,
  docs-1 on vps-2 with a failed clippy gate on RB-151. The handshake says `poll_s: 2`.
- Each reading walks the JSON and moves every number under a key ending `_ms` so its distance
  from `FIXTURE_NOW_MS` becomes its distance from the reading's moment, then moves the data on:
  - the working agent's run steps through its phases one a minute, earlier gates passed, with an
    event every 8 s (the newest 6 kept) and tokens growing with the time;
  - the hosts' processor and network wander by an integer table (`WOBBLE`), so nothing casts a
    float;
  - the third agent's `last_seen_ms` stops 12 s after the start, so it reads stale.
- A fixture that does not parse is a `PseudoError`, which the Fleet panel shows as the source's
  failure.

## Later
The schemas and a conformance kit (the fixtures as its cases) come with the SDK's own ticket;
the clients over MCP and HTTP are #611, the host collector #610.
