# Source & local changes

`rustal-ste` is Chad's adaptation of ASD-STE100 for messages between Rustal's agents (2026-10-09):
"for harness communication we should use this", then "adapt it and take it and use it for rustal
specific instructions across agents".

Upstream: <https://github.com/danyuchn/asd-ste100-skill> (MIT), vendored at commit
`32511c6992ecb5f1971e46a2943f2e6adceedafe` (2026-10-04). It was first added to this store as
`asd-ste100` (commit abfef00) and renamed and adapted here.

- **Verbatim upstream:** `LICENSE`, `references/writing-rules.md`, `examples/before-after.md`,
  `examples/linter-edge-cases.md`.
- **Adapted:**
  - `SKILL.md`: upstream's structural rules, simple tenses and scan checklist, under a new
    "When it applies" rule, plus Rustal's terms, verbs, identifiers and message shapes.
  - `scripts/ste-lint.py`: upstream plus a `rustal-term` check (advisory) that reads
    `scripts/rustal-terms.json`, skips backticked text and double-quoted words, and is covered by
    `--selftest`; `--glossary FILE` and `--no-glossary` choose the list.
- **New:**
  - `references/glossary.md` (70 terms, 40 verbs, identifiers, and the defaults in force until
    Chad decides) and `references/message-shapes.md` (13 shapes and their channels). Both were
    built from the rustal-harness, Marley, Rusty and Rustal Brain docs, each row citing its source.
  - `scripts/rustal-terms.json`: the glossary's drift words the linter flags. Generic English words
    are left out to keep the check quiet.

**Copies to keep in step:**
- rustal-harness's shared Claude Code plugin (its TICKET-115), and Marley's carried copy of that
  plugin (Marley #724);
- Marley's own agents (Marley #725).

Marley is a public repository, so this skill holds terms and rules only: no hosts, addresses,
accounts or secrets.

**Reviewed before vendoring:** upstream's `SKILL.md` and references hold writing rules only, and
`ste-lint.py` imports only the standard library and reads only the files it is given.
