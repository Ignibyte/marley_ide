# onboarding

> Per-crate reference (Marley round 2). Crate dir: `crates/onboarding`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (round 3):** `[Warp-derived/AGPL]` — Warp's first-run welcome/login/callout flow. Marley removed onboarding entirely; the `[Marley-original]` counterpart is the **M13 workspace launcher** (`crates/marley_app/src/launcher.rs` — the "open a workspace" landing page). See [subsystem 07 → Marley status @ M15](../subsystems/07-app-entry-build-tooling.md#marley-status--m15).
>
> **Provenance `[Warp-derived/AGPL: Warp UI + login-skip funnel — not adopted]`:** Warp's first-run slide deck + the `AI_FEATURES`/`WARP_DRIVE_FEATURES` skip-login confirmation funnel. Marley has no cloud login to gate → **not adopted** (Marley's own boot/launcher instead). See [subsystem 05 → Marley status & Provenance](../subsystems/05-cloud-auth-networking.md#provenance--licensing).

| | |
|---|---|
| Subsystem | [cloud-auth-networking](../subsystems/05-cloud-auth-networking.md) |
| License | AGPL v3 (`license.workspace = true`; no own `LICENSE` marker) |
| Internal deps | 6 |
| Used by | 1 |

## Purpose

`onboarding` is the **first-run experience**: the multi-slide welcome flow shown to new users, plus the in-app "callout" coachmarks that point at UI features afterward. It collects the user's intent (terminal vs agent-driven development), AI setup choices, theme, project selection, and login/skip decisions, and produces a `SelectedSettings` value the app applies on first boot.

It's relevant to the cloud-auth subsystem because onboarding is **where login is first presented and where it can be skipped** — the `OnboardingAuthState` and the skip-login confirmation dialog (driven by `AI_FEATURES` / `WARP_DRIVE_FEATURES`) are the user-facing front door to Warp's account gating.

## Key types, modules & public API

Library entry (`src/lib.rs`), plus a standalone `onboarding` binary (`src/bin/main.rs`, gated behind the `bin` feature for isolated dev/preview of the flow).

- **Top-level enums/consts (`lib.rs`):**
  - `OnboardingIntention { Terminal, AgentDrivenDevelopment }` and `SessionDefault { Agent, Terminal }` — what the user wants Warp to be.
  - `const AI_FEATURES: &[&str]` and `const WARP_DRIVE_FEATURES: &[&str]` — the user-facing feature lists shown in the agent card and the **skip-login confirmation dialog** (kept in one place so the two surfaces stay in sync).
  - `pub fn init(app: &mut warpui_core::AppContext)` — registers the onboarding + callout views.
- **`model`** (`src/model.rs`) — `SelectedSettings { Terminal {..} | AgentDrivenDevelopment {..} }` with `is_ai_enabled()` / `is_warp_drive_enabled()`; `OnboardingAuthState { LoggedOut, FreeUser, PayingUser }`; `UICustomizationSettings` (`agent_defaults()`, `terminal_defaults()`, `tools_panel_enabled()`); `AiSetupChoice`, `AiAccessChoice`; private `OnboardingStep` state machine.
- **`slides`** (`src/slides/mod.rs`) — one type per screen: `IntroSlide`, `IntentionSlide`, `AiSetupSlide`, `CustomizeUISlide`, `AgentSlide` (+ `AgentDevelopmentSettings`, `AgentAutonomy`, `OnboardingModelInfo`), `AiAccessSlide`, `ThirdPartySlide`, `ProjectSlide` (+ `ProjectOnboardingSettings`), `ThemePickerSlide`; shared `OnboardingSlide` trait, `onboarding_bottom_nav`, `layout`, `slide_content`.
- **`callout`** (`src/callout/`) — `OnboardingCalloutView`, `OnboardingKeybindings`: post-onboarding coachmark popovers anchored to UI elements.
- **`agent_onboarding_view`** — `AgentOnboardingView`, `AgentOnboardingAction`, `AgentOnboardingEvent`: the agent-specific onboarding surface.
- **`telemetry`** — `OnboardingEvent` and (under `bin`) `MockTelemetryContextProvider`.
- **`components`** / **`visuals`** — reusable widgets (`feature_optout_dialog`, `onboarding_callout`) and the animated mock terminals/visuals shown on each slide.

## Depends on (internal)

- [`warpui`](./warpui.md) — the UI framework the slides/views are built in (**MIT**).
- [`warpui_core`](./warpui_core.md) — core UI runtime (`AppContext`, `Element`, telemetry feature) (**MIT**).
- [`ui_components`](./ui_components.md) — shared higher-level widgets used across slides.
- [`warp_core`](./warp_core.md) — themes, icons, appearance, feature flags driving slide behavior.
- [`ai`](./ai.md) — `LLMId` / model info for the AI-setup and agent slides.
- [`warp_logging`](./warp_logging.md) — telemetry/log plumbing for `OnboardingEvent`.

(Plus external `rust-embed` (embeds slide assets), `strum`, `pathfinder_color/geometry`, `instant`.)

## Used by (internal dependents)

- [`warp`](./warp.md) — the only consumer; the app calls `onboarding::init` and renders the flow on first run, then applies `SelectedSettings`.

(Total internal dependents: 1.)

## Related crates

- [`warp_server_auth`](./warp_server_auth.md) — the auth state onboarding's login step feeds into; `OnboardingAuthState` mirrors its login tiers.
- [`warpui`](./warpui.md) / [`ui_components`](./ui_components.md) — the UI layers; useful prior art for building Marley's own custom panel.
- [`settings`](./settings.md) — where `SelectedSettings` is ultimately persisted by `warp`.

## Marley relevance

**Classification: STUB / RENAME (and a prime UI reference).**

Onboarding is both a **de-auth surface** and a **rebrand surface**, and it doubles as the cleanest worked example of building a multi-view UI in this framework.

- **(3) de-auth + login stub:** This is one of the two places login is *presented* to the user (the other being the in-app auth gates in `warp`). For offline Marley, **STUB** the login/AI-access slides: short-circuit `OnboardingAuthState` to a logged-in/`FreeUser`-equivalent, skip `AiAccessSlide`/`AiSetupSlide`'s account requirement, and have `is_ai_enabled()` resolve without an account. The `AI_FEATURES` / `WARP_DRIVE_FEATURES` skip-login dialog can be cut entirely.
- **(4) de-Warp rebrand:** Highest-visibility branding in the batch — every slide is literally "Welcome to Warp," and `WARP_DRIVE_FEATURES` / agent copy name Warp products. **RENAME** the user-facing strings, embedded assets (`rust-embed`), and the `onboarding` binary's branded visuals. Logo/theme assets live under `src/visuals/` and the embedded asset dir.
- **(1) custom panel:** `src/bin/main.rs` is a self-contained `warpui_core` app that builds windows, views, themes, and asset providers — an excellent **template** for standing up Marley's custom panel as its own buildable surface.
- **(2) sessions:** Indirect — `SessionDefault`/`OnboardingIntention` pick the default new-session mode the terminal core later honors.

Recommendation: **KEEP the slide framework, STUB the login/account gating, RENAME all Warp-branded copy & assets.** Consider trimming agent/cloud slides Marley won't ship.

## Notes / gotchas

- **`edition = "2021"`** (older than the `2024`-edition cloud crates in this batch).
- **Embedded assets:** `rust-embed` (`#[derive(RustEmbed)]` in `src/bin/main.rs`) bakes slide imagery into the binary — rebrand means replacing those embedded files, not just code strings.
- **Standalone binary** (`[[bin]] name = "onboarding"`, `required-features = ["bin"]`): the flow can be run in isolation, which is handy for iterating on Marley's rebranded onboarding without booting the whole app.
- **Feature-flag-driven behavior:** `SelectedSettings::is_ai_enabled()` branches on `FeatureFlag::OpenWarpNewSettingsModes` — "new vs old onboarding" diverge significantly; test both paths or pin the flag.
- The intricate comment in `is_ai_enabled()` warns that agent intent only signals *intent to require an account* — actual AI enablement is applied later in `apply_onboarding_settings` (in `warp`). The de-auth stub must touch that application point too, not just this predicate.
