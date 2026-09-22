---
title: Prompt line editing — Marley-local vs shell-ZLE (tab-completion at the prompt)
status: open
raised_by: TICKET-033 planning (the flagship interactive/raw-mode ticket)
milestone: M1.E / M2 (a product-model decision for chad)
---

## The decision
Marley currently uses a **Warp-style local line editor**: the prompt is edited in a `marley_editor`
Buffer (#28 in-line editing, #29 history) and the *whole command* is written to the shell on Enter
(`write_command`). The shell's own line editor (zsh's ZLE) never sees the keystrokes.

**Tab-completion, reverse-i-search (Ctrl-R), and the shell's own keybindings happen inside ZLE** —
which needs the raw keystrokes as-typed. So getting them requires **streaming every keystroke to
the shell at the prompt and rendering the shell's echo** — the traditional-terminal model. That
directly conflicts with the local editor: you can't both locally edit a Buffer AND let ZLE edit the
same line.

So it's a fork:
- **A) Keep Marley-local editing (Warp model):** tab-completion needs a Marley-side completion
  engine (ask the shell for completions over a side channel, or a native completer) — that's a
  real M2 feature. #28/#29 local editing + history stay.
- **B) Switch to shell-streaming at the prompt (iTerm model):** stream keystrokes to zsh's ZLE,
  render its echo; tab-completion/Ctrl-R/keybindings all "just work". But #28/#29's local editing +
  history become the shell's job (redundant), and the Block model's clean command/output split gets
  murkier (the prompt line is shell-drawn).
- **C) Hybrid:** local editing by default, a key (or heuristic) drops to shell-streaming for a
  completion — complex.

## What #33 delivered without this decision
#33 (interactive/raw mode) made **alt-screen** programs work (vim/top/less/htop) + **Ctrl-C/D/Z**
interrupt — because alt-screen has no "prompt" to conflict with, so raw streaming there is clean.
**Tab-completion at the bare prompt is NOT delivered** — it needs this A/B/C decision.

## Recommendation
Lead with **A** (keep the Warp-local model — it's Marley's differentiator) and build a completion
engine in M2, OR offer a per-user "raw shell" toggle (B) for people who want a traditional terminal.
Chad decides the product direction.
