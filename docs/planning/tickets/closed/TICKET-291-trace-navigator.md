# TICKET-291 — multi-frame stack-trace / backtrace navigator

- **Forge ticket:** #291 `66bf7033-82aa-427c-986a-4652e7cec6df` (feature, M18)
- **Owner:** autonomous /goal run (sprint #31 — M18 Terminal↔Editor Fusion)
- **AAR:** `1c28410d-d18b-4bfc-ad88-367f651bf9e6`
- **Pipeline doc:** ../../pipeline/active/291-trace-navigator.spec.md
- **Status:** closed

## Summary
A panic backtrace / Python traceback / Node stack has many source frames. A pure
`parse_trace_frames(output)` extracts every frame's location — crucially the
Python `File "path", line N` shape that #212's `path:line` scanner mangles (the
rust panic / backtrace / node frames are already `path:line`-shaped, so #212/#289
cover them). v1 folds the open-file frames into the #289 diagnostic rows, so the
gutter marks them and #290's F8 navigates them — reusing the shipped nav rather
than a new frame-list panel (deferred).

## Acceptance
A block whose output has a Python traceback referencing the open file marks those
frame lines in the gutter (and F8 walks them); a rust/node trace already covered
by #212 continues to work. Full EARS in the pipeline spec.
