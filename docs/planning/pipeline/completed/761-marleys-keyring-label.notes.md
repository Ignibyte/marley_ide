# Marley's keyring label - its Secret Service items under its own name — Notes

- **Local ticket doc:** docs/planning/tickets/closed/TICKET-761-marleys-keyring-label.md
- **Pipeline spec:** 761-marleys-keyring-label.spec.md

## Phase 1 — Plan (2026-10-11)
- **Request:** #445's split (Chad, 2026-10-11, "7.) yes"); its spec's Out item 2 drafted this
  slice. Taken from the Queue under the goal "continue until all tickets are finished", with the
  defaults written in the spec.
- **Classification:** chore, packaging; one Zed hunk (`gpui_linux`), a scenario. Size S.
- **Recall (§18.3):**
  - PR-claude-a-zed-fork-identity-is-more-than-app-name-001: the keyring label is part of the
    identity the channel keyed.
  - #548 and #565 (System One): their scenarios never pressed Set Key or Forget Key, to keep off
    the user's keyring; this one runs a keyring of its own.
  - PR-claude-check-a-unix-socket-path-against-sun-path-001: gnome-keyring's control socket lives
    under `XDG_RUNTIME_DIR`, so the run's goes under `/run/user/<uid>`.
  - Brain: consultation c40fb49c7e6b47299f5e4960f942d5f1, nothing on this seam.
- **Discovery:**
  - `gpui_linux/src/linux/platform.rs:57` `KEYRING_LABEL`; `write_credentials` 1028-1045
    (`create_item(label, {url, username}, secret, replace: true)`), `read_credentials` 1047-1073
    and `delete_credentials` 1075-1093 (both `search_items({url})`, then the first item whose label
    matches).
  - On `dev`, `zed_credentials_provider` uses the development credentials file (47-62), so the
    keyring holds only what calls `cx.*_credentials` directly: System One's keys
    (`system_one.rs:501`, `546`, `560`). System One calls shows the source, "Key: keyring"
    (`system_one_calls.rs:218-221`), with Set Key and Forget Key (281-290).
  - A private keyring works here: a session `dbus-daemon`, `printf PASS | gnome-keyring-daemon
    --login`, then `gnome-keyring-daemon --start --components=secrets`, with `XDG_DATA_HOME` in the
    run's folder and `XDG_RUNTIME_DIR` a short folder under `/run/user/<uid>`; `secret-tool store`
    and `search --all` read it. `--unlock` alone made no login keyring and left `secret-tool`
    waiting on a prompter.

### Design
- **`crates/gpui_linux/src/linux/platform.rs` (Zed crate):** `KEYRING_LABEL = "Marley"` and
  `LEGACY_KEYRING_LABEL = "zed-github-account"`; `read_credentials` takes the new label's item,
  else the old label's, whose secret it copies under the new label (a failed copy logged, the
  secret still returned); `delete_credentials` deletes the URL's items under both. Each hunk with a
  `// Marley:` line.
- **Ledger row:** `crates/gpui_linux/src/linux/platform.rs` (check for an existing row).
- **Docs:** the guide's System One key passage; the architecture note; CHANGELOG.
- **Scenario:** `script/e2e/761-marleys-keyring-label.sh` (Marley-owned).

### Visual check plan
| REQ | The scenario does | The shot or check |
|---|---|---|
| 002 | Seeds the old-label item; opens System One calls | 761-01; `secret-tool search --all url URL` lists both labels |
| 003 | Clicks Forget Key | 761-02; the search finds nothing |
| 001 | Clicks Set Key, types a key, Enter | 761-03; the search finds one item, labelled `Marley`, with the new key |
| 004 | Everything on the private bus and keyring | The user's keyring is never addressed; the review |

### Risks
- **A stock Zed on the same machine** keeps its items under the old label; Marley copies only an
  item for a URL it asks for, and deletes the old item only when the person forgets the key in
  Marley, as before.
- **A keyring that asks to unlock** asks once per read, as before; the copy is a second write in
  the same unlocked session.

## Phase 2 — Code (2026-10-11)
- **Built:** the ledger row for `crates/gpui_linux/src/linux/platform.rs`, first. There:
  `KEYRING_LABEL = "Marley"`, `LEGACY_KEYRING_LABEL`, `KEYRING_APPLICATION`
  (`application = marley`) in both `create_item` calls; `read_credentials` returns the `Marley`
  item, else the old label's, copied under the new label (a failed copy logged) and left in place;
  `delete_credentials` removes the URL's items under both labels. The guide, the architecture
  note, CHANGELOG.
- **Deviations from the design:** the attribute, after the first Test run (F-claude-761-…): the
  label alone does not keep a replace off the old item.
- **The review:** a read that finds both labels returns Marley's item; a copy that fails still
  returns the secret; nothing deletes an old item but an explicit delete.
- **Clippy:** clean under the gate's scope (every Marley crate and `gpui_linux`).
- **The gate:** `script/gates.sh --diff` green on its first run, 17 of 17, on the tree with the
  attribute.

## Phase 3 — Test (2026-10-11)
- **The scenario:** `script/e2e/761-marleys-keyring-label.sh`, under `compositor sway`, on a
  private bus with a keyring of its own (L-claude-761-…), seeded with System One's key under
  `zed-github-account`. Exit 0, three checks passing. Shots read:
  - `761-01-read`: System One calls, "Key: keyring"; the keyring holds `Marley old-key-761` and
    `zed-github-account old-key-761` (REQ-002).
  - `761-02-forgotten`: after Forget Key, "Key: none (set MARLEY_SYSTEM_ONE_KEY, or Set Key in
    System One calls)" and "The key is gone from the keyring."; no item for the URL (REQ-003).
  - `761-03-set`: after Set Key and `new-key-761`, "Key: keyring" and "The key is in the
    keyring."; one item, `Marley new-key-761` (REQ-001).
  - The run's keyring and bus were stopped and removed; the user's daemon (its own pid) was never
    addressed (REQ-004).
- **Fix on the way:** the first run's read left only the `Marley` item: the copy had replaced the
  old one (F-claude-761-…, PR-claude-761-…); the attribute fixed it, and the run passed.

## Phase 4 — Complete (2026-10-11)
- **Documented:** `CHANGELOG.md` (Added, #761); the guide's System One key passage and the
  release-identity line; `marley_workbench.md`'s System One key; the ledger row.
- **Knowledge appended:**
  AD-claude-761-marleys-keyring-items-are-labelled-marley-and-carry-their-own-attribute-001;
  F-claude-761-a-copy-under-a-new-label-replaced-the-old-item-001;
  PR-claude-761-a-keyring-items-label-does-not-tell-replace-apart-001;
  L-claude-761-a-private-secret-service-for-a-scenario-001.
- **Brain:** consultation c40fb49c7e6b47299f5e4960f942d5f1 closed with a decision.
- **Closed:** the ticket moved to `closed/`, the pipeline archived, committed.
