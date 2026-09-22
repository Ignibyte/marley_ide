# typed panes (PaneKind) — Notes

- **Forge ticket:** #107 `99ad0ddd-b6be-46a2-b6b5-5d987ac0fb08` · **AAR:** `c2c6b7cc-185d-4241-b21c-e4e9a375114c`
- **Local ticket doc:** docs/planning/tickets/open/TICKET-107-typed-panes.md

## Phase 1 — Plan
- **Request:** forge #107 (M5 1/12, FOUNDATION) — panes gain a PaneKind.
- **Pre-flight:** layout.rs PaneGroup = pure geometry (Leaf(PaneId)/Split); workspace.rs Workspace<S> +
  PaneState<S> (per-pane state). The kind attaches to PaneState (D1). Split calls PaneState::new (→ default
  Terminal); close removes from the map (survivors untouched).
- **Decisions:** D1 kind on PaneState not the tree; D2 all-Terminal now + label() keeps variants live.
- **AAR id:** `c2c6b7cc-185d-4241-b21c-e4e9a375114c`.

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
- **workspace.rs (PURE):** `#[derive(Debug,Clone,Copy,PartialEq,Eq,Default)] pub enum PaneKind { #[default] Terminal, FileTree, CodeView, Git }` + `impl PaneKind { pub fn label(self) -> &'static str { match self { Terminal=>"terminal", FileTree=>"files", CodeView=>"code", Git=>"git" } } }`. `PaneState<S>` gains `pub kind: PaneKind`; `PaneState::new` sets `kind: PaneKind::Terminal`.
- **No layout.rs change** (D1 — the tree stays geometry). No app.rs change (#107 — all panes Terminal; seq-2 consumes kind).
- **Mutation targets:** the 4 label arms; the new() default Terminal; kind preserved across split (new pane Terminal) + close (survivors untouched).
- **Test plan:** pane_kind_label (all 4 → labels); pane_kind_defaults_terminal (Workspace::new → PaneId(0).kind==Terminal); pane_kind_survives_split_and_close (state_mut(focused).kind=Git; split_focused→new pane Terminal + original Git; close(new)→original Git). cov/MSI 100. All variants constructed in tests → live under clippy --all-targets.
- **Risks:** none structural — additive field + enum; the split/close algebra is untouched (kind rides the per-pane map).

## Phase 3 — Implement
- **Built:** PaneKind{Terminal,FileTree,CodeView,Git} (Copy, Default=Terminal) + label() in workspace.rs; PaneState<S>.kind field set to Terminal in new(). No layout.rs/app.rs change (all panes Terminal; seq-2 consumes kind).
- **Verification:** fmt; check 0 err (the not-yet-tested variants/label are dead-code WARNINGS only — the Phase-4 tests construct all 4 + call label(), clearing them under clippy --all-targets).

## Phase 3.5 — Inspect
- **Method:** self-review (a tiny additive enum + field on a pure, heavily-tested module; gate cargo-mutants is the MSI authority).
- **Lenses — no findings:** PaneKind is Copy+Default(Terminal); label() total over 4 variants; PaneState::new defaults kind Terminal so split (→ PaneState::new) makes new panes Terminal + the original/​survivors keep their kind (the HashMap entries are untouched by split/close); the PaneGroup layout tree is unchanged (D1 — geometry only). No unwrap/panic; the field is additive (no behavior change for existing panes). No findings.
- **Fix applied:** none.

## Phase 4 — Validate
- **Tests:** pane_kind_label (4 arms) + pane_kind_defaults_terminal (new()+Default) + pane_kind_survives_split_and_close (split→new Terminal/original Git; close→survivor Git). `cargo nextest` → 3 pass; clippy OK (all variants + label now live).
- **Self-test:** N/A — #107 is pure workspace.rs (no UI shim; every pane is Terminal so the render is unchanged). seq-2 adds the visible title bar.
- **Gate:** GREEN [diff] 15/15, cov/MSI 100 (first try).

## Phase 5 — Complete
- CHANGELOG ### Added; forge #107 → done. **M5 1/12 — FOUNDATION.** PaneKind + PaneState.kind (cov/MSI 100). Opens M5 — The Warp Workspace.
