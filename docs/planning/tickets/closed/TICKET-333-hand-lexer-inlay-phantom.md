# TICKET-333 — editor: the hand-lexer fallback lexes inlay phantom text (narrow, cosmetic)

- **Ticket:** LOCAL #333 (bug, M-unset)
- **Tags:** editor, m21-followup, inspect-found, 331, cosmetic
- **Created:** 2026-07-16
- **Provenance:** exported from forge 2026-08-09 (TICKET-409 pivot; forge-era id 403277b9-176e-4010-a3e0-91d001da45bf)
- **Status:** closed (2026-08-11 — pipeline 73aadd5a; fixed as prescribed: raw-lex + map through the shared `spans_to_display_bytes` remap; gate green, MSI 100 on diff)

## Description

Filed from #331 inspect (F8). On a row the tree-sitter cache cannot serve, the render falls back to `crate::code_syntax::highlight_ranges(&layout.display, lang)` — which lexes the DISPLAY string, and since #331 that string contains server-controlled inlay phantom text. A hint carrying a quote or a lifetime tick (`: &'a str`, and rust-analyzer's closing-brace hints are literally `// fn main`) can start a string/char/comment token inside the phantom whose state BLEEDS RIGHTWARD into real code — e.g. `display = "let x: \"weird = q;"` lexes ` = q;` as part of a Str.

SEVERITY IS LOW AND THE REACHABLE SET IS NARROW — traced honestly at inspect rather than assumed: (a) #331 shipped a language gate, so a non-Rust file never gets hints at all; (b) after an edit BOTH caches invalidate together (the inlay cache is keyed on buffer version, the syntax cache on nonce), so the async-parse window renders no phantoms either; (c) #331's hints-first span ordering already guarantees the phantom's OWN cells paint `Hint` regardless of what the lexer says. What remains is a bleed to the RIGHT of a phantom on a Rust file whose syntax cache is permanently absent — i.e. one containing an exotic line separator (a bare `\r`, FF, NEL, LS, PS), where `lines.get(row)` returns None and the row hand-lexes forever. The consequence is mis-coloured text only: the layout, caret, click and selection all stay correct.

WHY IT WASN'T FIXED IN #331: the two cheap fixes both cost more than the bug. Remapping (lex a phantom-free display, then shift the spans) is NOT a simple insertion offset — a phantom changes tab-stop expansion (`"a\tb"` + a 2-col hint at 1 → `"aXX b"`, one space, vs `"a   b"`), so the two display strings are not related by insertion. Suppressing hints on hand-lexed rows couples the hint render to the syntax cache — a real design entanglement for a rare cosmetic case.

LIKELY RIGHT FIX: give the fallback the same shape as the primary path — lex the RAW line and map through `raw_span_to_display_bytes` (which since #331 ends code spans on the code-side `col_of_span_end`, so it is phantom-safe by construction). That also removes the last place where a display-string offset is treated as a code offset. Check it does not regress fallback highlighting for the non-Rust files that rely on it (TOML/JSON/Shell), which is why it is its own ticket and not a #331 rider.
