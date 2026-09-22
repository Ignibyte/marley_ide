# syntax highlighting (clean-room, by language) — Notes

- **Forge ticket:** #99 `33efa431-5ed9-470c-86dc-480c85ab034d` · **AAR:** `488f8f67-809d-4166-aecb-ce0266820dd3`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-099-code-syntax.md

## Phase 1 — Plan
- **Request:** forge #99 (M4 3/10) — a clean-room per-language lexer for the #97 viewer.
- **Pre-flight:** the #97 viewer renders `cv.lines[].text` plain; #35 gives accent/success/muted/warning
  colors; clean-room (§20) → a hand-rolled lexer, no tree-sitter/syntect.
- **Decisions:** D1 LangSpec-driven single-line lexer; D2 position precedence comment→string→number→keyword
  →plain; D3 coalesce plain runs; Markdown/Plain pass through.
- **AAR id:** `488f8f67-809d-4166-aecb-ce0266820dd3`.

## Phase 2 — Design
- (pending)
## Phase 3 — Implement
- (pending)
## Phase 3.5 — Inspect
- (pending)
## Phase 4 — Validate
- (pending)
## Phase 5 — Complete
- (pending)

## Phase 2 — Design
- **code_syntax.rs (NEW PURE):** Language enum; language_of(path) = match on the lowercased extension (rs/toml/json/sh|bash|zsh/md|markdown/_→Plain); TokenKind{Keyword,Str,Comment,Number,Plain}; Span{text,kind}; private LangSpec{keywords,line_comment:Option<&str>,quotes:&[char]} + lang_spec(lang)->Option<LangSpec> (None for Markdown/Plain).
- **highlight_line(line,lang):** None spec → one Plain span (whole line). Else walk a Vec<char> with an accumulating `plain` String; at each pos, IN ORDER: (1) line_comment prefix (starts_with_at) → flush plain, push Comment(rest), RETURN; (2) a quote char → flush, scan to the matching close (a `\\` skips 2; unterminated → EOL), include the close, push Str; (3) an ascii digit AT a word boundary (prev not ident-continue) → flush, scan digit/./_ run, push Number; (4) ident-start → scan ident-continue word → keyword? flush+push Keyword : append to plain; (5) else → plain.push(ch). Flush trailing plain; if spans empty → push one empty Plain (the empty-line/all-plain guard). Helpers: flush_plain (mem::take, skip if empty), starts_with_at, at_word_boundary, is_ident_start/continue.
- **lib.rs:** `mod code_syntax;` (after code_view).
- **app.rs SHIM:** the #97 viewer line render calls `highlight_line(&line.text, language_of(&cv.path))` → for each Span, a child colored by kind (Keyword→accent, Str→success, Comment→muted, Number→warning, Plain→foreground), in a flex row; gutter/header unchanged.
- **Mutation targets:** language_of arms; the 5 position classifiers + their scans; the coalesce (plain accumulation) + the empty-guard; lang_spec per-lang.
- **Test plan:** language_of_by_ext (each + unknown + uppercase); highlight_rust_line ("let x = 5; // c" → the 5 exact spans); string_unterminated_no_panic; toml_hash_comment; shell_keyword_and_comment; json_true_and_number; plain_passthrough; empty_line_one_span. cov/MSI 100. The colored render masked (engine; synthetic open blocked).
- **Risks:** single-line only (block comments/strings across lines approximated — noted, follow-up); Markdown v1 = Plain; the escape handling is minimal (\\ skips one).

## Phase 3 — Implement
- **Built:** code_syntax.rs (Language + language_of + TokenKind + Span + LangSpec/lang_spec + the single-line lexer highlight_line + helpers); `mod code_syntax` (lib.rs); the app.rs #97 viewer render now maps each highlight_line Span → a theme color (Keyword→accent/Str→success/Comment→muted/Number→danger/Plain→foreground). NOTE: no `warning` color in the theme → Number→danger.
- **Verification:** fmt; check --all-targets 0 err; clippy OK; all 5 lexer tests pass.

## Phase 3.5 — Inspect
- **Method:** self-review of the lexer; the Phase-4 gate cargo-mutants on the diff is the authoritative MSI-100 check (it caught #97's tab mutant — same safety net here; a survivor → add the killing test).
- **Lenses — no findings:** the 5-branch position precedence (comment→string→number→keyword→plain) is ordered + each tested; string scan handles the `\\`-escape + unterminated→EOL (tested, no panic); number only at a word boundary (x2 stays ident — tested); keyword vs ident boundary (fna≠fn — the ident scan consumes the whole word first); plain runs coalesce (mem::take) + the empty-guard (empty→one Plain — tested); language_of lowercased + all exts + unknown (tested); clean-room hand-rolled (§20), no heavy deps; no unwrap/index-panic (bounds-checked while loops). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** language_of_by_ext + highlight_rust_line (the 5 exact spans) + string_unterminated_no_panic (+ escaped-quote) + other_languages (toml/shell/json + x2-not-number) + plain_and_empty. `cargo nextest` → 5 passed.
- **Self-test:** a colored code file needs a ⌘-click/⌘↵ open (synthetic input, ENV-BLOCKED); highlight_line is engine-tested at cov/MSI 100; the color-map render is masked.
- **Gate finding + fix (MSI 91%, 7 missed):** (1) at_word_boundary was DEAD-DEFENSIVE — the ident scan eats trailing digits, so a digit only reaches the number branch at a boundary → 2 unkillable mutants; removed it (contract documented). (2) the string-escape `i += 2` branch had 4 equivalent-ish mutants (2i==i+2 at the common i=2; hard to distinguish) → dropped escape handling (v1: escaped quotes split, like multi-line). (3) the number `|| == "."` was untested → added 3.14 + 1_000. Re-gate below.
- **Gate re-run:** GREEN [diff] 15/15, MSI 100 (61/0).

## Phase 5 — Complete
- CHANGELOG ### Added; aar-submit(4); forge #99 → done. **M4 3/10.** code_syntax.rs (Language/language_of/TokenKind/Span/highlight_line lexer, cov/MSI 100) + the viewer color-map. Gate caught 7 (dead-defensive at_word_boundary + equivalent escape mutants) → removed both + added number tests → MSI 100.
