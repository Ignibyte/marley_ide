# TICKET-539 — Marley's Chromium introduces itself as Chrome

- **Ticket:** LOCAL #539 (chore, prong 3)
- **Owner:** ba5cc5f0-d61c-4b8e-97c8-fde390e55d4a
- **Pipeline doc:** ../../pipeline/completed/539-headless-chrome-user-agent.spec.md
- **Source ticket:** The Orca survey of 2026-09-25, report 03 §3 item 10: "Drop 'HeadlessChrome' from the user agent. Not queued; check first." (listed in `docs/orca_architecture/README.md` among the smaller things worth a day each); specced because Chad asked on 2026-09-25 for every item decided that day to be specced.
- **Status:** closed

## Summary
Every page in a Browser tab is told it runs in a headless browser. Marley starts Chromium with no user agent of its own (`chromium_args` in `crates/marley_browser/src/service.rs`), so it sends Chromium's headless one: `Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) HeadlessChrome/152.0.0.0 Safari/537.36`, as the running unit reported on 2026-09-25. Bot checks such as Cloudflare's reject it, so some sites refuse to load or to sign in inside Marley. Each page gets Chromium's own identity with the headless marker taken out: the user agent says `Chrome/`, and the user-agent client hints (the `Sec-CH-UA` headers and `navigator.userAgentData`) carry the matching brands and versions, since a user agent that disagrees with its hints is itself a bot signal.

## Acceptance
A page in a Browser tab sees `Chrome/152.0.0.0` and no `HeadlessChrome` in its `User-Agent` header and in `navigator.userAgent`, with hints that agree; the same holds in a cross-site iframe and on the first request of a tab an agent opens.
