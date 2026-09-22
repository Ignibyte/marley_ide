# TICKET-332 — LSP: an abandoned request must terminate — a typed Abandoned signal per consumer

- **Ticket:** LOCAL #332 (chore, M-unset)
- **Tags:** lsp, m21-followup, inspect-found, 331
- **Created:** 2026-07-16
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id a783c5ed-5bc1-4508-8d82-51bea5e2fbb0)
- **Status:** closed (2026-08-11, pipeline 6f65d651 — shipped)

## Description

Filed from #331 inspect (F6b). `REQUEST_TIMEOUT_TICKS`'s own comment promises "a slow server never wedges a consumer", but the timeout path (`lsp_host.rs` `expire` → `purposes.remove(&id)`) and `on_connection_lost` (`purposes.clear()` + `responses.clear()`) both drop a request's purpose WITHOUT delivering anything — so the consumer's apply fn never runs and its per-feature latch is never cleared. Today every consumer except inlay tolerates this by accident: they are event-driven, so the next keystroke mints a fresh key and overwrites the stale latch. #331 fixed its own instance the contained way (`has_pending_inlay()` — ask the pending table, the one source of truth for in-flight-ness), leaving the general hole for the next poll-driven consumer to rediscover.

THE FIX THAT WAS REJECTED, AND WHY IT MUST NOT BE COPIED NAIVELY: an inspect critic proposed pushing a synthetic `Err` into `responses` for every abandoned purpose, claiming "siblings are unaffected". That is FALSE and was verified false by reading each `Err` arm — `apply_prepare_rename_response` treats Err as `declined` → flashes "Cannot rename this"; `apply_code_action_response` → "Code actions failed"; `apply_code_action_resolve_response` → "Code action resolve failed". A synthetic Err would fire those toasts TEN SECONDS after the user's F2/quick-fix keypress, on three shipped features (#322/#323).

SO THE WORK IS: a DISTINCT terminal signal, not an Err cosplay. Sketch — make `responses` carry `Result<Value, RequestOutcome>` (or add an `Abandoned { reason: Timeout | Disconnected }` variant), push one for every purpose dropped on both abandonment paths, and give each consumer an explicit arm: hover/definition/completion/signature → drop silently (their current Err behaviour is already benign); rename/code-action → clear the latch WITHOUT a flash, or flash something honest and timeout-specific ("The language server didn't answer") rather than the semantic error; inlay → clear the latch (which `has_pending_inlay()` already achieves, so this becomes belt-and-braces and the host query could then retire). Every consumer gains "a request always terminates" instead of each re-deriving it.

AC sketch: a request that times out delivers exactly one Abandoned to its purpose's consumer; no consumer shows a semantic error for a timeout; `on_connection_lost` delivers Abandoned for every cleared purpose (push AFTER the existing `responses.clear()` or it eats them); a headless test proves a wedged latch clears. See PR-claude-poll-driven-inflight-check-must-read-the-owner-001 and PR-claude-critic-shared-fix-safety-claim-needs-per-consumer-check-001.
