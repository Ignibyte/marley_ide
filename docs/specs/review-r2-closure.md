# Round-1 Spec Review — Closure

**Status: GO_WITH_FIXES** (per the exit criteria stated in [review-r2.md](review-r2.md): close B1–B4, `spec-provenance.sh` exits 0, fold the highs).

## The convergence loop
| Round | Verdict | Blockers | Total findings |
|---|---|---|---|
| review-r1 | NOT_READY | 11 | 86 |
| review-r2 (after reconcile+fix) | NOT_READY | 4 (all clean-room) | 29 |
| **closure (this)** | **GO_WITH_FIXES** | **0** | highs/mediums folded |

The 9 structural/buildability blockers closed in the fix pass (one owner per shared type via `standards/seam-contracts.md`; the 3 missing specs authored). The 4 that survived were all the clean-room spec-layer wall being *described* but not *real*.

## The 4 closed here
- **B1 — gate-16 self-test unsatisfiable** → implemented `scripts/spec-provenance.sh` (the interim shell enforcement of [SPEC-gate](SPEC-gate.spec.md); the mutation-tested `marley_spec_provenance` Rust crate is the ticketed replacement). It isolates each spec's "Public surface (the contract)" + `spec_source`, skips blockquote "what-was-stripped" notes and the gate's own denylist data (gate R8), and fails closed. **Exits 0 on the current tree.**
- **B2 — transcription `spec_source`** → the 6 specs citing `warp_architecture/…` or fictional behavior-doc paths (`marley_core`, `editor`, `input-classifier`, `markdown-render`, `syntax-highlight`, `app-shell`, + `foundation-spike`) were rewritten to behavior-level prose (observable I/O, no fork paths/names).
- **B3 — fictional behavior docs** → closed by B2 (no dangling doc references remain).
- **B4 — asset return-type contradiction** → `SPEC-assets` aligned to the owner's `AssetState<T::Output>` (asset-foundation §10.1).

## Folded into the relevant tickets (not blocking)
The remaining ~8 highs + mediums/lows from [review-r2.md](review-r2.md) are carried into their component tickets as known refinements — e.g. the editor `BufferEvent` subscription + selection-ownership seam, the gate-6/miri scope note, two unmapped EARS clauses, asset manifest/version threading. Each is a within-implementation fix, not a cross-spec contradiction.

## Clean-room posture (honest)
The spec-layer wall is now **mechanically green** (gate 16). The deeper legal-provenance question — that the build derives from a fork-reference — remains an explicit **IP-counsel sign-off** item (tracked in [../marley_architecture/](../marley_architecture/) and the licensing decision), not an engineering gate. Posture A holds: Marley-original naming, behavior-level specs, gate-enforced.

**The spec set is ticket-ready.**
