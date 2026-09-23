# TICKET-446 — License files for the Marley crates

- **Ticket:** LOCAL #446 (chore, licensing)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** none yet (not specced)
- **Source ticket:** the #438 inspect ledger (`../../pipeline/completed/438-marley-layout-and-rail.notes.md`)
- **Status:** open

## Summary
Every Marley crate declares `license = "MIT OR Apache-2.0"`, but none carries a license file,
and the repository has no MIT license text at all. Zed's own `script/check-licenses`, which its
CI runs, fails on the first Marley crate: it wants a `LICENSE-GPL` or `LICENSE-APACHE` symlink in
every crate. The Apache side is a symlink to the root `LICENSE-APACHE`; the MIT side needs its
text with a copyright line, which is Chad's to give (the holder's name). Waits for that line.

## Acceptance
Each `crates/marley_*` holds `LICENSE-APACHE` (a symlink to the root file) and `LICENSE-MIT`
(the MIT text with Chad's copyright line), and `script/check-licenses` passes over the Marley
crates.
