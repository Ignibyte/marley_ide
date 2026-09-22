//! `marley_fleet` — the generic session envelope + fleet snapshot the whole control plane stands on.
//!
//! PURE: no gpui, no transport, no product vocabulary — a project-specific string in this crate is
//! a defect; that projection belongs in the consumer. The types here are, at once, the UI model
//! (#369 fleet rail, consumed by the app's fleet rail directly — the forge_client transport that
//! fed it retired at #411), the test surface (#368 adapter projection), and the MCP tool
//! schema (#370 server) — one seam, three consumers. Ticket ① of the M23 Layer-1 fleet train
//! (`docs/marley_architecture/orchestration-shell.md` §12); it fixes the envelope v1 field-level
//! decisions while they are cheap.
//!
//! - [`session`] — the [`Session`] envelope + its closed [`State`] / [`Transport`] vocabularies.
//! - [`reducer`] — the [`SessionEvent`] stream + the pure, replay-safe fold into a [`FleetSnapshot`].
//! - [`attention`](mod@attention) — read-time staleness + attention derivation (no clock; `now_ms` injected).
//! - [`dispatch`] — the `Deposited → Claimed → Started` delivery-state machine (monotone join).
//! - [`verbs`] — the session-verb request/receipt types as data (the MCP tool schema).

pub mod attention;
pub mod dispatch;
pub mod reducer;
pub mod session;
pub mod verbs;

pub use attention::{Attention, AttentionReason, attention, is_stale};
pub use dispatch::{DeliveryAdvance, DeliveryState};
pub use reducer::{FleetSnapshot, SessionEvent, apply, reduce};
pub use session::{Question, Session, State, Transport};
pub use verbs::{
    AnswerRequest, OpenRequest, ReadRange, ReadRequest, Receipt, SendRequest, SurfaceRequest,
};
