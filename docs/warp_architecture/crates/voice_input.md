# voice_input

> Per-crate reference (Marley round 2) — crate dir `crates/voice_input`. Marley is forked from Warp (warpdotdev/warp).
>
> **Provenance (06 re-review):** `[Warp-derived/AGPL — de-auth]` — mic capture over permissive `cpal`/`rubato` but streams audio to the external Wispr service; not ported. See [subsystem §06](../subsystems/06-platform-settings-infra.md#provenance--licensing).

| | |
|---|---|
| **Subsystem** | [platform-settings-infra](../subsystems/06-platform-settings-infra.md) |
| **License** | AGPL v3 (workspace `AGPL-3.0-only`) |
| **Internal deps** | 1 (`warpui_core`) |
| **Used by** | 1 (`warp`) |

## Purpose

`voice_input` is the **microphone-capture and audio-encoding** half of Warp's voice-to-text feature. It records from the system default input device, downsamples the captured audio to the 16 kHz mono PCM that the transcription backend (Wispr) expects, and hands the caller a base64-encoded WAV blob. It is deliberately **capture-only**: it does *not* call any transcription API, hold network credentials, or do speech recognition — it just turns "press to talk" into a finished WAV. The whole crate is a single `lib.rs` (~430 lines).

It exists so the app can offer voice input without smearing `cpal`/`hound`/`rubato` audio-stack details across the app crate, and so the mic state machine lives behind one small typed entity.

## Key types, modules & public API

Everything is at the crate root (no submodules).

- **`VoiceInput`** — the singleton entity (`impl Entity` with `type Event = ()`, `impl SingletonEntity`). Constructed via `new(ctx: &mut ModelContext<Self>)`. Public methods:
  - `start_listening(&mut self, ctx, source: VoiceInputToggledFrom) -> Result<VoiceSession, StartListeningError>` — opens the `cpal` input stream, builds the resampler, returns a session handle.
  - `stop_listening(&mut self, ctx) -> Result<(), anyhow::Error>` — pauses the stream and spawns WAV conversion; the result is delivered through the `VoiceSession`.
  - `abort_listening(&mut self)` — stops without producing audio; the session receives `Aborted`.
  - State queries: `is_listening()`, `is_transcribing()`, `is_active()`, `state() -> &VoiceInputState`, `start_time() -> Option<Instant>`.
  - `set_transcribing_active(&mut self, bool)` — lets the app flip the entity into/out of `Transcribing` while it talks to the (external) transcription service.
  - Public field `should_suppress_new_feature_popup: bool`.
- **`VoiceInputState`** (`enum`, `#[derive(Default)] = Idle`) — `Idle`, `Listening { stream, chunk_size, enabled_from, resampler, resampled, result_tx }`, `Transcribing`.
- **`VoiceSession`** — owned by the caller; `async fn await_result(self) -> VoiceSessionResult`. Backed by a `futures::channel::oneshot` receiver.
- **`VoiceSessionResult`** — `Audio { wav_base64: String, session_duration_ms: u64 }` or `Aborted { session_duration_ms: Option<u64> }`.
- **`VoiceInputToggledFrom`** — `Button` or `Key { state: KeyState }` (telemetry on how recording was triggered).
- **`StartListeningError`** (`thiserror`) — `AlreadyRunning`, `AccessDenied`, `Other(#[from] anyhow::Error)`.

Internals worth knowing: a `cpal` input callback averages channels to mono and ships frames over an `async-channel` to `on_audio_frame` → `resample_audio_frame` (a `rubato::SincFixedIn` resampler to `TARGET_SAMPLE_RATE = 16000`), accumulating into a shared `Vec<f32>`; `convert_to_wav` then encodes 16-bit PCM WAV via `hound` and base64-encodes it. Tunables: `DEFAULT_CHUNK_SIZE = 512`, `NUM_CHANNELS = 1`, `STREAM_TIMEOUT = 6 min`.

## Depends on (internal)

- [warpui_core](./warpui_core.md) — the entity/runtime substrate: `Entity`, `ModelContext`, `SingletonEntity`, `event::KeyState`, `platform::MicrophoneAccessState` (OS permission check), and `async::block_on` / `ctx.spawn` / `ctx.spawn_stream_local` to run capture and conversion off the main thread.

## Used by (internal dependents)

- [warp](./warp.md) — the app crate (dir `app/`). The entity is wired through `app/src/lib.rs`, `app/src/root_view.rs`, `app/src/settings/ai.rs`, `app/src/workspace/view.rs`, `app/src/terminal/universal_developer_input.rs`, and related input/handler modules. Compiled only when the app enables its `voice` feature (this crate has no features of its own).

## Related crates

- [warpui_core](./warpui_core.md) — supplies the platform mic-permission and async runtime hooks.
- The (external) transcription/AI path — the consumer of `wav_base64` — lives in the agent/AI subsystem (see [ai](./ai.md) / [warp_core](./warp_core.md)); this crate stops at the WAV.
- Out-of-repo audio stack: `cpal` (device I/O), `rubato` (resampling), `hound` (WAV).

## Marley relevance

**Classify: STUB (or REMOVE).** Voice input is a non-core, cloud-coupled convenience that touches none of the four Marley goals. It is only compiled behind the app's `voice` feature, so the cleanest path for an offline/de-Warp Marley boot is to **leave the `voice` feature off** — the crate then never compiles in, no stub needed. If something still references the entity, STUB it: `start_listening` returns `Err(StartListeningError::AccessDenied)` (or a new "disabled" variant) so the UI degrades gracefully.

Relevant to goal (3) *de-auth + login stub* indirectly: the produced WAV is meant to be sent to Wispr, a hosted, authenticated transcription service. With auth stubbed out there is no transcription endpoint to call, so keeping voice live would surface a dead button — another reason to default it off. For goal (4) *de-Warp rebrand*, the only touch is the package name `voice_input` (already neutral) and any UI copy referencing the feature. Defer any real work; this is low priority.

## Notes / gotchas

- **Out-of-repo native deps**: `cpal` links the platform audio backend (CoreAudio / WASAPI / ALSA). The error-callback comment notes ALSA on Linux can fire error callbacks in a tight loop on device disconnect — the code logs only the first error per session and debug-logs the rest to avoid flooding Sentry. Keep that throttling if you touch it.
- **macOS quirk**: on macOS `cpal` will happily create a stream of all-zero frames even when the user *denied* mic access, so the crate explicitly re-checks `ctx.microphone_access_state()` and returns `AccessDenied` rather than recording silence.
- Hard-codes 16 kHz mono 16-bit because that's the Wispr requirement; not configurable.
- `edition = "2024"`. Has no own LICENSE marker — inherits workspace AGPL v3.
- Dropping a `VoiceSession` does **not** stop recording (documented) — it only abandons the result channel; call `abort_listening`/`stop_listening` to actually stop the stream.
