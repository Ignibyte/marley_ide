# TICKET-262 — Brand-scrub: remove Warp/Zed mentions from the source + a keep-clean lint

- **Forge ticket:** #262 ae50af5a-4d7a-4d26-a2fb-4d92375b999a (chore, M16)
- **Owner:** ede913c3-d048-4f39-ad2b-b21cef1efc8e
- **AAR:** 5d1aeb91-91ae-426e-b7ec-4e9c5eecb483
- **Pipeline doc:** ../../pipeline/active/262-brand-scrub.spec.md
- **Source ticket:** sprint #29 "M16 — Cleanup + Editor Frontier" (forge)
- **Status:** closed

## Summary
Execute the audited brand-scrub (roadmap Phase A): reword the 56 whole-word
"Warp" comment mentions across 11 `crates/**/*.rs` files in Marley's own terms
(all hits are comment prose — zero strings/identifiers/config, zero "Zed"),
keeping every ticket number and technical value, then wire the keep-clean lint
(`grep -rniwE 'warp|zed' crates --include='*.rs'`, folded into gate:14) in the
same commit so the count stays at zero. The reference transcriptions under
`docs/*_architecture/` are the deliberate exception and keep their mentions.

## Acceptance
Post-scrub detector returns zero matches; the diff is comment-only; ticket
numbers/technical values survive; gate:14 fails on an injected brand word and
ignores the 80+ `…zed` substrings; rustdoc stays warning-free; the color.rs
clean-room attestation survives reworded. Full EARS criteria in the pipeline
spec.
