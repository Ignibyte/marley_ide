# TICKET-480 — Voice input through Voxtype

- **Ticket:** LOCAL #480 (feature, prong 1: T7d)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/480-voice-input.spec.md
- **Source ticket:** ../../../marley/three-prong-plan.md (T7)
- **Status:** closed

## Summary
Warp's agent bar has a microphone that turns speech into prompt text, through a hosted service.
Omarchy already ships Voxtype, a local push-to-talk dictation daemon running a Whisper model
that types what you say into the focused window; on this box it runs with `base.en`. The agent
bar gets a microphone button that starts and stops Voxtype and shows when it is listening.

## Acceptance
Where Voxtype is installed, the button toggles its recording and shows its state, and the text
lands where the focus was. The EARS criteria are in the spec.
