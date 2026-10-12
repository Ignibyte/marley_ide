---
pipeline_id: 811e70bb-361f-48d7-a34d-74c8eaa38727
ticket: docs/planning/tickets/closed/TICKET-761-marleys-keyring-label.md
status: Phase 4 — Complete PASS
title: Marley's keyring label - its Secret Service items under its own name
type: chore
slice: packaging, the release identity (TICKET-445), slice 3 of 5
references:
  - docs/planning/pipeline/completed/445-marley-release-identity.spec.md
  - docs/planning/pipeline/completed/760-marley-calls-itself-marley.spec.md
  - docs/planning/pipeline/completed/565-system-one-layer.spec.md
---

## Title
Marley writes its Secret Service items under stock Zed's fixed label, `zed-github-account`
(`gpui_linux/src/linux/platform.rs:57`), and reads and deletes only items with that label. This
slice gives them Marley's own label, `Marley`, and carries over what Marley wrote before: an item
found only under the old label is read, copied under the new one, and left where it was. The third
of #445's five slices (#445's spec, Out item 2).

## Scope
### In
- **The label.** `KEYRING_LABEL` is `Marley`: `write_credentials` creates items with it, and
  `read_credentials` and `delete_credentials` look for it first.
- **Carried over.** `read_credentials` finds no item with the new label for a URL but one with the
  old label: it returns that item's secret and writes a copy under the new label. The old item
  stays, since a stock Zed may own it.
- **Deleting.** `delete_credentials` removes the URL's item under each label, as a delete removed
  the one shared item before, so a forgotten key does not come back from the old label.
- Docs: the guide (where Marley's keys live), the architecture note for System One's key.

### Out (explicitly deferred)
- Zed's own credentials on `dev` (sign-in, provider keys), which go to the development
  credentials file, not the keyring (`zed_credentials_provider.rs:47-62`): unchanged.
- macOS and Windows keychains: Marley ships on Linux.
- The `marley://` scheme and the desktop entry: #762. The updater: #763.

## Reference (§20)
Upstream Zed, the `gpui_linux` crate: credentials go to the Secret Service through `oo7`, one item
per URL with `url` and `username` attributes and a fixed label (`platform.rs:1028-1095`). Marley
keeps the store, the attributes and the calls, and names its own label, as a packager of gpui would.

### Prior art
- **The code we ship:** `gpui_linux`'s `write_credentials`, `read_credentials` and
  `delete_credentials` (`platform.rs:1028-1095`), over `oo7::Keyring` (`search_items`,
  `create_item` with `replace`, `Item::label`, `Item::delete`); the one caller in Marley's code,
  System One's key (`system_one.rs:501`, `546`, `560`), shown as "Key: keyring" in System One
  calls (`system_one_calls.rs:218-221`); `zed_credentials_provider`, which keeps `dev`'s own
  credentials in a file.
- **Behavior maps:** none.
- **Published material:** the freedesktop Secret Service API (items, labels, attributes).

## UI proof
`script/e2e/761-marleys-keyring-label.sh`, under `compositor sway`, with a private session bus
and a `gnome-keyring-daemon` of the run's own (its keyrings in the run's folder), so nothing
reaches the user's keyring. Before Marley starts, the keyring holds System One's key under the old
label for the run's `compatible` endpoint.
- `761-01-read`: System One calls says "Key: keyring"; the keyring now holds the item under
  `Marley` and still under `zed-github-account`.
- `761-02-forgotten`: after Forget Key, "Key:" names the key as missing; no item is left for the
  URL under either label.
- `761-03-set`: after Set Key with a new value, "Key: keyring"; the one item is under `Marley`.

## Locked-In Decisions
- **D1 — The label is `Marley`.** The label is what a keyring manager shows; Zed's names an
  account it is not. Rejected: `com.ignibyte.Marley` (the app id, not a name a person reads).
- **D2 — Copy on read, keep the old item.** A stock Zed on the same machine may own an item under
  the old label. Marley's items carry `application = marley` (added in Test, F-claude-761-…), since
  the Secret Service replaces by attributes whatever the label. Rejected: a one-time migration of every old-label item (it would copy stock Zed's
  too, and needs state of its own), moving the item (it would take stock Zed's).
- **D3 — A delete removes both.** Otherwise the old item would answer the next read and a
  forgotten key would come back. A delete in Marley removed the shared item before, so nothing
  that worked stops working.

## Acceptance Criteria (EARS)
| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN Marley writes a credential, the system shall store it under the label `Marley`. | Shot 761-03 and the keyring check |
| REQ-002 | WHEN Marley reads a credential that only an item under `zed-github-account` holds, the system shall return it and copy it under `Marley`, leaving the old item. | Shot 761-01 and the keyring check |
| REQ-003 | WHEN Marley deletes a credential, the system shall remove the URL's items under both labels. | Shot 761-02 and the keyring check |
| REQ-004 | Nothing in a run shall reach the user's keyring. | The scenario's private bus and keyring folder; the review |
| REQ-005 | The change shall pass `script/gates.sh --diff`. | The gate |
