---
pipeline_id: 3b915c8e-8721-4b88-aea3-d21dc6617f89
ticket: docs/planning/tickets/closed/TICKET-859-a-project-mark-in-notifications-and-the-switcher.md
status: Phase 4 — Complete PASS
title: A project's mark in its notifications and the switcher
type: feature
slice: The Marley look, W1.2 follow-up (docs/marley/look-plan.md)
references: [docs/marley/look-plan.md]
---

## Title
A desktop notification from a project's terminal carries the project's logo file as its image,
and the rail's switcher draws each row's project mark beside the project's name.

## Scope
### In
- `crates/gpui/src/platform.rs` (Zed crate): `SystemNotification.image: Option<PathBuf>`, and its
  constructors in gpui's tests and example (`image: None`).
- `crates/gpui_linux/src/linux/system_notifications.rs` (Zed crate): the image as the
  `image-path` hint.
- `project_icons.rs`: `logo_file(root)`, the file the mark came from.
- `notifications.rs`: the terminal's project's logo file on its banners; `harness.rs` and
  `schedule_runs.rs` pass none.
- `rail_switcher.rs`, `rail.rs`: a switcher row keeps its project's folder and draws the mark.

### Out (explicitly deferred)
- macOS and Windows notification images: gpui's other platforms leave the image out; Marley runs
  on Linux here.

## Reference (§20)
The Desktop Notifications Specification's `image-path` hint; Zed's gpui notifications.

### Prior art
- `gpui::SystemNotification` (tag, title, body, actions); `notify_rust::Notification::image_path`.
- `project_icons` (#564, #831): the mark cache keeps the file each mark came from (`chosen`).
- `rail_switcher.rs`: rows name their project as text.
- `script/e2e/538-notifications-with-content.sh`: a private bus with a server that logs banners.
- The brain (consultation a802421c): nothing on this seam.

## UI proof
Scenario `script/e2e/859-project-mark-in-notifications-and-switcher.sh` (sway): a project with
`logo.png`, a private notification server that logs each banner's image hint, two terminals.
- `859-01-banner`: a terminal's OSC 777 notify while the focus is elsewhere; the log's line names
  the logo file (REQ-001), shot with the log printed in the other terminal.
- `859-02-switcher`: Ctrl+Tab in the rail: each row's mark beside its project (REQ-002).
- `859-03-day`: Marley Day, the switcher (REQ-002).

## Locked-In Decisions
- D1 — The image is the logo file the mark came from; a project with only its letter sends none.

## Acceptance Criteria (EARS)

| # | EARS requirement (`shall`) | Verify |
|---|---|---|
| REQ-001 | WHEN a terminal of a project with a logo posts a desktop notification, the notification shall carry the logo file as its image. | `859-01-banner` |
| REQ-002 | The rail's switcher shall draw each row's project mark beside its project's name, in Marley Night and Marley Day. | `859-02-switcher`, `859-03-day` |

## Phase Plan
- **P1 Plan**, **P2 Code**, **P3 Test**, **P4 Complete**.
