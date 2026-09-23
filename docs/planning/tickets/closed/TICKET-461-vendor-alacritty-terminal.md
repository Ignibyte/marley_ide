# TICKET-461 — Zed's alacritty_terminal carried in the repo

- **Ticket:** LOCAL #461 (chore, prong 1: T0a)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/461-vendor-alacritty-terminal.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T0, D1)
- **Status:** closed

## Summary
The block terminal catches Marley's shell hooks in alacritty's event loop (three-prong plan,
D1), so Marley has to carry its own copy of the `alacritty_terminal` Zed pins. Chad,
2026-09-23: "i would keep it all in the same so we dont have to manage another library". The
crate is copied unchanged into `vendor/alacritty_terminal`, and Cargo takes it from there
through `[patch]`. The gates, the owned set, the receipt fingerprint and a re-sync note cover
it. The hooks themselves are #462.

## Acceptance
The build uses the vendored copy, Zed's terminal behaves as before, and the gate is green.
