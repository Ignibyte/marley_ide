# TICKET-688 — A notice for the manager's reports

- **Ticket:** LOCAL #688 (feature, prong 2 C)
- **Owner:** ce546757-1075-466f-a3cf-b6696d2211c9
- **Pipeline doc:** [688-a-notice-for-the-managers-reports.spec.md](../../pipeline/completed/688-a-notice-for-the-managers-reports.spec.md)
- **Source ticket:** `docs/planning/intake/marley-agent-manager-foreman.md`, phase 2; rustal-harness
  TICKET-106 (the thread) and TICKET-111 (`rh acp`), its D179
- **Status:** closed

## Summary
The Agent Panel shows a manager's report sent between turns but raises nothing for it (#685).
The harness keeps each thread record's author, kind and time and serves the thread by cursor and
subscription (its TICKET-106); it sends no desktop notice of its own and leaves that to Marley.
Marley follows the manager's thread and, for a manager record of kind report or confirmation that
arrives while its thread is not in front, raises a notice: a desktop notification and the manager
thread's rail row marked, the same way Marley raises an agent that needs the user. Waits on the
harness's TICKET-106.

## Acceptance
With a harness manager posting a report while the user looks at another tab, Marley shows a desktop
notice naming the manager and the report's first line, and the manager thread's row is marked
until the user opens it; a report while the thread is in front raises no notice.

## The harness thread's shape (its TICKET-106 plan, 2026-10-08; names may shift until it closes)
- **Records:** `{id, author: manager|person, kind: message|report|confirmation, text, refs[],
  answers?, created_ms, sequence}`.
  - A manager's record also carries its session.
  - A person's message carries its delivery UUID and, once it reaches the manager, a hand-off.
  - A confirmation carries its action (the exact read-back), `expires_ms` (60 s), an answer once
    given, and a state: pending, accepted, rejected or expired. A second, late or mismatched
    answer is refused by name: `thread_confirmation_used`, `thread_confirmation_expired`,
    `thread_confirmation_mismatch`, `thread_confirmation_unknown`.
- **Reading** (any grant):
  - `owner://thread`, which can be subscribed to, gives `{manager, cursor, records}` (the latest
    256), and `notifications/resources/updated` fires on each record, hand-off or answer.
  - `owner://thread/{after}` gives up to 128 `{sequence, change}` after a cursor. One older than
    retention gets `resync_required`.
  - The tool `thread_read` gives the same page. The CLI is `rh thread read`.
- **Writing:**
  - The person, under the write grant: `thread_send {text, delivery?, refs?, answers?}` and
    `thread_confirm {id, action, choice: accept|reject}`.
  - The manager: `thread_post` and `thread_status`.
- **The notice's event:** a new record with `author: "manager"` whose `sequence` is past what a
  thread view has shown. A confirmation's `expires_ms` can drive a countdown.

## Final (harness TICKET-106 closed 2026-10-08; its MANAGER.md, "The thread")
- **Pages.** A page entry is `{sequence, kind, change}`. `kind` is `thread_record`,
  `thread_answer` or `thread_handoff`.
- **The `owner://thread` snapshot.** Each record carries its `sequence`. A confirmation also
  carries `state` and an `answer`, and a person's message carries `handoff {record, session,
  delivery}`.
- **Sending.** `thread_send` returns `{id, delivery, state: "recorded"}`. A retry with the same
  delivery and other text is `thread_delivery_conflict`.
- **Refusals.** A person's refused write reads `operation_failed: NAME: …`, with the code after
  the prefix: `thread_confirmation_{used,expired,mismatch,unknown}`, `thread_delivery_conflict`,
  `thread_record_shape`, `thread_record_unknown`, `thread_full` (PR-693: pass them by form).
- **CLI.** `rh thread show`, `rh thread read --after N`, `rh thread send TEXT [--delivery U]
  [--ref R]… [--answers ID]`, `rh thread confirm ID --action TEXT accept|reject`.
- **The event.** A new `thread_record` with author `manager`, past the last sequence a view has
  shown.
- **Why it waits on 111.** The ticket's "thread in front" is the Agent Panel's `rh acp` thread
  (item 7), so it waits on TICKET-111 too.
