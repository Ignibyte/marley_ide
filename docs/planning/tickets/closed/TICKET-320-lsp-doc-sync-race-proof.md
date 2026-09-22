# TICKET-320 — LSP doc-sync: make text materialization race-proof by construction + add an app-level Ready (fake_ls) test lane

- **Ticket:** LOCAL #320 (chore, M20)
- **Tags:** M20, lsp, docsync, test-lane, from-validate, 312-followup
- **Created:** 2026-07-15
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id cdfcce0c-9312-4e4d-82f9-a3000b320a25)
- **Status:** closed (2026-08-11 — shipped by pipeline c7587c3f; gate green 15/15, MSI 100)
- **Pipeline:** docs/planning/pipeline/completed/320-lsp-docsync-race-proof.spec.md (promoted + completed 2026-08-11)

## Description

Follow-up to `BF-lsp-didopen-carries-empty-text-ready-race-001`, found by #312's live drive and FIXED there with a minimal ordering change (drain before collect). This ticket makes the bug class impossible and adds the test that would have caught it.

BACKGROUND (the shipped bug, already fixed in #312): the pump collected `needs_text(..)`-gated buffer text BEFORE `host.drain()`, but a host reaches `Phase::Ready` INSIDE `drain()` and both `needs_text` and `reconcile` are Ready-gated. On the tick the phase flipped, the collector saw NOT-Ready (text = "") while `reconcile` saw Ready and sent that empty string as the didOpen text — then recorded the doc as synced, so no didChange ever followed. rust-analyzer held every file as a zero-length document; hover (#311) and definition (#312) both answered null. Wire-proven: 3 didOpens at TEXT_LEN=0 (log 1226 bytes) before; a single didOpen at Content-Length 16781 with real source (log 104220 bytes) after.

PART 1 — STRUCTURAL FIX (make it impossible, not merely ordered correctly):
The minimal fix in #312 is ordering-dependent — a future refactor that moves the collect back above the drain silently reintroduces it. Collapse the two Ready-gated reads into ONE call that DECIDES and MATERIALIZES:
- Change `reconcile(&mut self, open: &[(PathBuf, String, BufferVersion)])` to take `&[(PathBuf, BufferVersion)]` plus a `text_of: impl Fn(&Path) -> String` (or pass `&Buffer` directly — `lsp_host` is marley_app, so it may depend on marley_editor).
- `reconcile` calls `text_of(path)` ONLY on the branches that actually send (didOpen / didChange), preserving the #309 hot-path optimization (`PR-claude-pump-materialize-hot-data-only-when-consumed-001`) with no second phase read.
- DELETE `needs_text` — it becomes unwired API (the #311-F3 rule), and it is the only reason the caller had to duplicate reconcile's decision table in the first place.
See `PR-claude-two-gated-calls-must-read-the-phase-once-001`.

PART 2 — THE MISSING TEST LANE (this is the real gap):
Every M20 headless drive injects responses via `push_response_for_test` into a process-less host that NEVER reaches Ready — so no existing test can observe what the app SENDS. That is precisely why #309/#310/#311 all shipped "driven-proven" with this bug live. See `PR-claude-synthetic-response-tests-never-prove-the-live-wire-001`.
- `fake_ls` (#308) already reaches Ready, but only from `marley_lsp`'s own integration tests via `env!("CARGO_BIN_EXE_fake_ls")`, which does not resolve cross-crate. Bridge it: either move the Ready-lane test into `marley_lsp/tests/`, or resolve the fixture path from `current_exe()`'s target dir in `marley_app`'s headless drive, or add a tiny test-only spawn hook.
- Then add the regression test: boot a headless workspace with a `.rs` file open, spawn a host against `fake_ls`, pump until Ready, and assert the captured outbound didOpen carries the file's REAL text (`TEXT_LEN > 0` AND equal to the buffer). Needs a `#[cfg(test)] sent_bodies_for_test()` capture on `LspHost::send_body`.
- ASSERT THE PAYLOAD, NOT THE METHOD: the bug shipped with a perfectly correct `"method":"textDocument/didOpen"` on the wire. A test that only greps the method name passes on the broken code.

PART 3 — cheap standing guard:
The `[[lsp.servers]]` tee wrapper used to diagnose this is a reusable trick worth documenting in `scripts/selftest/README.md`: point `command` at a script that `tee`s stdin/stdout, drive the app, then read the frames. It is the only thing that showed the truth here. Also worth a note: if a SAME-FILE definition returns empty, stop suspecting the parser — the far side has no document; go read what you sent.
