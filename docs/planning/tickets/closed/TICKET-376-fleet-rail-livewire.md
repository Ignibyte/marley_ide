# TICKET-376 — Fleet rail live wire — app-side FleetSubscription from [[projects.orchestration]]

- **Forge ticket:** #376 (f2983e9d-fb30-4df3-a613-0b41d5609afd) (feature, M24)
- **Owner:** unclaimed
- **AAR:** pending-promotion
- **Pipeline doc:** ../../pipeline/queued/376-fleet-rail-livewire.spec.md
- **Source ticket:** sprint #35 "M24 — Fleet Layer 2"; the §12.1 runbook's precondition 1 (app-side wiring)
- **Status:** closed

## Summary
The fleet rail (#369) still renders only the `fleet-demo-feed` fixture: the self-healing
`FleetSubscription` (#368, integration-proven #372, reconnect+backoff #373) has NO app-side caller, and
`orchestration_for` (#371) has no consumer — the re-export at `marley_app/src/lib.rs:98-99` exists
precisely so it isn't dead code before this ticket. Wire the shipped halves together: at boot (the
settings-apply path — settings load once, app.rs:1256), when the active project's root has a
`[[projects.orchestration]]` entry with a loopback `brain_endpoint`, start exactly ONE
`FleetSubscription` on its own background thread; delivered snapshots flow through the existing gpui
pump into `RootView.fleet_snapshot` (a pump-installed change sets dirty —
`PR-claude-pump-state-change-must-set-dirty-to-repaint`); the rail header gains a connection-state
indicator (live / reconnecting / off) from a typed state the subscription loop publishes; quit stops
the thread cleanly via the shipped stop-flag + join Drop. An unconfigured root keeps today's behavior
verbatim, `fleet-demo-feed` verb included; a live delivery supersedes a demo snapshot. The loopback
guard and never-logged-bearer posture carry to the settings-sourced endpoint.

## Acceptance
Configured root ⇒ one subscription starts at boot and the rail renders the live snapshots (proven
app-level against a scripted fixture forge, the #372 livewire pattern); the header state reads
live/reconnecting/off from the typed subscription state (never inferred from snapshot age); no
orchestration entry ⇒ no subscription and #369's shipped behavior is unchanged; quit joins the thread
promptly (stop honored within the ~1s poll quantum, `SubscriptionExit::Stopped`); an unreachable
endpoint keeps the last snapshot rendered and reads reconnecting, and a reconnect's delta read loses no
seats (#373 carry-forward). Pure decision seams (start-gate, endpoint construction, state→label,
supersede) at cov/MSI 100; thread/channel/boot/pump glue masked. Full EARS in the pipeline spec.
