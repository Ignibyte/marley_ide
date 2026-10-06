# T3 Code survey 07: engineering, architecture and releases

Read from a local clone of `github.com/pingdotgg/t3code` at `17c0878941` (2026-10-06). Paths are relative to that
repo root unless they start with `crates/` (Marley). Marley paths outside `crates/` are named
with their folder (`script/`, `docs/marley/`, `CONSTITUTION.md`). Release notes come from the
GitHub releases v0.0.33 to v0.0.45 (2026-08-10 to 2026-10-02), read with `gh release view`.

## 1. Summary

T3 Code (first commit 2026-02-07; 4,866 commits, 1,274 of them in the last 30 days, about 41 a day;
about 410 author names, led by Julius Marminge, Theo Browne and maria; 25,825 stars) is a Node
server that owns every agent process, terminal, checkout and database row, with web, Electron and
React Native clients that attach over one typed RPC contract. The server is written in Effect-TS,
and since 2026-10-02 its orchestration is event-sourced. A pure orchestrator decides events, and one
SQLite transaction commits the events, the projections the clients read, a command receipt and the
side effects still to run, which a leased worker then performs. The repository holds about 839,000
lines of non-test TypeScript (78,000 of them generated protocol bindings) and 542,000 lines of
tests. Its engineering habits are tuned for code that agents write: 474 of the last month's 1,274
commits carry an agent co-author line, and `AGENTS.md` says most contributions come from T3 Code
itself, driven remotely. Most of the mechanism is TypeScript- and client/server-shaped and does not
transfer to a single-process Rust editor. What is worth taking: (1) test runs on a copy of the
user's data that cannot act on anything, from `migrate-dev-db`; (2) user docs that change in the
same change as the feature, rewritten rather than appended; (3) `t3 triage`, which hands a broken
install to the user's own coding agent with a context file and a playbook; (4) protocol shapes for
Codex's App Server regenerated from upstream's schema at each Codex release, with recorded
transcripts; (5) the "Hit every surface" checklist; (6) review rubrics scoped by path, shared by the
author's doc and the reviewer.

## 2. Features

Here "features" are the practices and mechanisms that make up how T3 Code is built and shipped.
Each section ends with **Marley today**. The last section is the feature list as the release notes
show it.

### 2.1 Process architecture: one server, three kinds of client

**What it is.** A Node server (`apps/server`, 274,000 non-test lines) owns provider processes,
PTYs, git, project files and the SQLite store. Clients are the React web app (`apps/web`,
241,000), the Electron shell that wraps it and bundles a server (`apps/desktop`, 40,000) and a
React Native app (`apps/mobile`, 113,000). `packages/contracts` (26,000 lines of Effect Schema)
types everything that crosses the wire; `packages/client-runtime` (37,000) holds the connection
and domain state shared by web and mobile.

**How it works.** `docs/internals/overview.md` sets the rule: "A remote client must never
substitute its own filesystem, provider credentials, or machine state for the environment's. The
desktop app bundles a server, but its renderer follows the same boundary." The desktop renderer
therefore talks to its own bundled server over the same authenticated RPC as a phone does, which
is why most features work remotely with no extra work. Subscriptions send only what a client views,
so "a client viewing one thread does not pay for every thread's history". Version skew is handled
by capability flags in an environment descriptor (the pull-request linking table in
`overview.md`) and by a hard protocol check: `ORCHESTRATION_PROTOCOL_VERSION = 2`
(`packages/contracts/src/environment.ts:13`), sent as a socket query parameter
(`packages/client-runtime/src/connection/compatibility.ts:40`) and refused by `/ws` with HTTP 426
before any RPC runs (`apps/server/src/ws.ts:607`, described in
`docs/internals/legacy-orchestration-migration.md:34-41`). The refusal names which machine to
update.

**Good.** Remote parity comes from the shape, not from per-feature work. The 426 refusal is
better than running half-upgraded.

**Bad.** Every UI action is a round trip through a server, a schema and a projection. The contract
package plus client runtime is 63,000 lines before any feature.

**Marley today: has part, by a different shape.** Marley is one GPUI process. Chromium runs in a
systemd user unit (#488), the MCP server is in-process (#491), and the harness runs outside (#534,
#632). Zed's own remote development splits a headless server from the client for projects, but
many Marley features stop at that line: "A remote project's terminals get no turns" (guide,
Per-turn diffs), remote terminals get no `MARLEY_BIN` (guide, An agent's own reports). Report 04
covers what remote parity would take. The contract side is partly there: `marley_fleet` and
`docs/marley/fleet-contract.md` type the fleet, `crates/marley_mcp/src/registry.rs` holds every
tool's JSON schema, and the plugin and agent versions are checked (#547, #648).

### 2.2 Effect services and thin transports

**What it is.** Every server capability is a method on an Effect service in its domain folder.
WebSocket handlers, HTTP routes, MCP tools, scheduled tasks and the CLI only decode, call one
method and map errors (`docs/internals/effect-services.md`).

**How it works.** The stated reason is reach: "Users trigger a capability from the WebSocket,
agents reach it through MCP tools, and scheduled tasks and the CLI run it too. Logic written into
one handler is missing from the others" (`effect-services.md`, "Where a feature lives").
Dependencies come from the Effect environment, never as constructor parameters or module globals.
Errors are `Schema.TaggedError` classes whose attributes are bounded ("no raw payloads, command
arguments or output, signed URLs, credentials"), with the exact value only in `cause`. Production
code reads time through `DateTime.now` and randomness through `effect/Random`, so tests substitute
`TestClock` and a seeded `Random` (`docs/orchestration-v2/testing-strategy.md`, "Allowed Test
Substitutes"). A checked-in review agent enforces the same text (2.11).

**Good.** "Could an agent (MCP) or a scheduled task use this capability? If not, is that
deliberate?" is on the pre-push list in `effect-services.md`. It makes agent reach a per-change
decision.

**Bad.** The conventions run to several hundred lines across two documents and a reviewer
prompt, for a library most contributors learn on the job.

**Marley today: has part.** Marley's MCP tools call into the workbench, and §14 sets Rust
conventions (no `unwrap` on input paths, never `let _ =` on a fallible call). Redaction keeps
secrets out of what tools return (#516, #562). There is no rule that a user action gets an agent
route or a recorded reason why not: the rail's Merge, Remove, Review, Launch and port Stop have no
MCP tool. Item 5 below.

### 2.3 Event-sourced orchestration v2

**What it is.** The orchestration rewrite that landed on 2026-10-02 as one squash commit,
`feat(orchestrator): introduce new orchestrator (#2829)`: 1,907 files changed, 380,865 lines added
and 203,787 removed, with 45 co-author lines including Claude, Devin and the t3-code bot. As of
2026-10-06 it ships only in the `v0.0.46-nightly` builds; the stable v0.0.45 predates it.

**How it works.**

- The orchestrator (`apps/server/src/orchestration-v2/Orchestrator.ts`) serializes commands and
  decides events from projection state "without performing provider or filesystem work"
  (`docs/internals/overview.md:55`).
- `EventSink.commitCommand` (`EventSink.ts:527-584`) runs in one SQLite transaction: it reserves the
  command id with `insertIfAbsent` (line 532), appends the events, folds them into projections,
  enqueues outbox effects (line 558) and writes the receipt (line 568). A command id seen before
  commits nothing and returns the original receipt and events, so a client retry after a dropped
  socket cannot start a second turn.
- Publication order is protected by a one-permit "publish lane": a writer takes it as the last
  step inside its transaction and releases it after publishing, because "clients drop any event at
  or below the newest sequence they have applied" and a descheduled writer could otherwise publish
  out of order (`EventSink.ts:233-260`).
- The effect worker (`EffectWorker.ts`) claims outbox rows with a 30-second lease, tries each up to
  five times with exponential delay capped at 30 seconds (lines 539-541, 725-730), and treats
  interrupt races such as "is not active" as success rather than failure (lines 41-60). Tests call
  `drain` instead of sleeping: "A test that needs a timeout to pass is wrong" (`AGENTS.md`,
  Verifying).
- A provider turn ending and its follow-up work settling are separate milestones. Checkpoint
  capture and workspace refresh run in `RunFinalizationService.ts` after the turn, and "A late
  checkpoint or diff must not extend the recorded provider duration or keep the client showing
  provider work as active" (`overview.md:77-79`). A provider that cannot roll back its
  conversation "must reject that operation before changing the filesystem" (`overview.md:85`).
- Startup recovery selects candidates (queued runs, pending requests, live sessions, unfinished
  deliveries) from projection state before reading any full thread
  (`docs/internals/performance-regressions.md`, "Event store and startup").

**Good.** Receipts make every command idempotent at the API boundary. The publish lane is a small,
well-explained fix for a real ordering race. Separating turn end from finalization keeps spinners
honest.

**Bad.** `Orchestrator.ts` is 10,939 lines in one file, beside Theo's own note in `AGENTS.md`:
"Do not introduce machinery because it looks architecturally impressive." A 380,000-line squash
merge cannot be bisected inside, and V1 history survives only as an import: run records,
checkpoints, tool activity and approvals do not migrate (`legacy-orchestration-migration.md`,
"Imported data").

**Marley today: lacks, by design.** Marley does not own agent sessions. The terminal agent's own
CLI keeps its conversation, the rail reads events (#519, #650), and supervised sessions belong to
rustal-harness (`docs/marley/three-prong-plan.md`, prong 2). The one receipt-like piece Marley has
is the fleet contract's retry ids on `session_send` and `session_open` (#533). The outbox,
receipts and the publish-lane rule are lessons for the harness, listed under Skip.

### 2.4 Persisted history and the fork's migration trap

**What it is.** Rules for keeping old data readable when the schema moves.

**How it works.** Persisted events must stay decodable on replay, since "an image-only server can
fail the entire environment's startup when replaying one such event" (`docs/internals/providers.md`,
"Attachments and stored history"). The V2 cutover copies `state.sqlite` to `statev2.sqlite` and
migrates only the copy, so V1 keeps working. One paragraph is written for forks:
`effect_sql_migrations` compares ids only, so "There is no safe id range for a fork inside this
ledger: any id at or below a future upstream id masks it forever, so fork schema changes belong in
a separate migration table or outside the migrator entirely"
(`legacy-orchestration-migration.md:47-52`). `migrate-dev-db` also checks for two branches that
claim the same migration slot (`apps/server/scripts/migrate-dev-db.ts:17-21`).

**Marley today: has.** Zed's `sqlez` panics at startup when a recorded migration's text differs
(`crates/sqlez/src/migrations.rs:3`), the same hazard in a stricter form. Marley already keeps its
rows in domains of its own: `MarleyTerminalIdsDb`
(`crates/marley_workbench/src/terminal_ids.rs:119`), `MarleyAgentSessionsDb` (`resume.rs:277`),
`MarleyBrowserTabsDb` (`browser.rs:7091`) and `MarleyRustyGraphTabsDb` (`rusty/graph_store.rs:225`).
The touchpoint ledger records the reason for #575: "a terminal's id lives in Marley's own table ...
so no Marley migration joins upstream's list". Settings migrations sit in Marley's own file,
`crates/migrator/src/migrations/marley.rs` (ledger row, #643).

### 2.5 Performance discipline

**What it is.** Budgets written as tests, and a culture of `perf` PRs: 109 of them between
2026-08-10 and 2026-10-02, against 350 `feat` PRs.

**How it works.**

- Wire budgets. `vp run test:perf:v2-wire` pins cold opens to "75 recent timeline rows and about
  1 MiB of encoded data", omits raw command output and tool-result bodies at the wire boundary, and
  caps resume catch-up at 128 events and 1 MiB (`performance-regressions.md:15-35`).
  `apps/server/integration/transferBudgetV2.integration.test.ts` measures HTTP and WebSocket
  transfer per thread and fails on a violation (line 351); CI publishes the report into the job
  summary and a PR comment (`.github/workflows/ci.yml:262-297`,
  `.github/workflows/thread-transfer-report.yml`).
- Live streams. One subscription holds at most 1,000 items and 8 MiB
  (`LiveStreamBudget.ts:17-18`). Tool updates are coalesced so only "the latest in-flight update
  for each stable tool-call id" survives, and terminal updates are never discarded
  (`ThreadLiveEventCoalescer.ts:43-46`).
- Leases. Command palette results keep live git and PR queries "leased to the visible viewport"
  (`performance-regressions.md`, "Background work and navigation"); background GitHub polling moved
  to batched GraphQL, quoted from v0.0.45: "fix(server): background GitHub polling uses ~74% fewer
  calls with batched GraphQL (#14673)".
- Taste. "Our users drive agents all day and notice a dropped frame, a lying spinner, and a stale
  label. No continuously repainting animations; they peg the GPU on high-refresh displays"
  (`AGENTS.md:157`).

**Good.** A budget that fails locally and posts its numbers on the PR catches regressions in review,
not in a user's report.

**Marley today: has part.** Zed's hang detection logs a foreground stall over 100 ms and a frame
over 24 ms in release builds (`crates/zed/src/reliability/hang_detection.rs`). The rail reads
ports, drift and PR state only while it shows (guide, The rail), and the MCP tools cap what they
return (`terminal_read`: 2,000 lines, 256 KiB). Orca report 07 (row 12) already proposed
typing and restore budgets. Nothing new to take.

### 2.6 Native work in child processes, with deadlines

**What it is.** A rule that anything native that can crash or stall runs outside the main process.

**How it works.** "Native modules never load in the Electron main process on the startup path ...
new native capability goes in a child with a deadline, not an `import` in main"
(`overview.md:118-123`). Process telemetry comes from a standalone Rust monitor,
`native/resource-monitor/src/main.rs` (1,334 lines, `sysinfo`), which samples continuously only
while a diagnostics view subscribes; "A missing or failed collector leaves the server running"
(`docs/internals/resource-telemetry.md`). Window capture uses small Rust helpers per compositor
(`native/hyprland-snap-shot`, `native/kde-snap-shot`) and a C helper for the browser keyring
(`native/browser-secret/main.c`).

**Marley today: has.** Chromium runs as its own systemd unit (#488). gate:22 confines process
starts to four adapter modules listed in `.config/spawn-sites.txt`, with a pinned count and a
planted violation (#541), and the workbench's adapter kills a program whose call is dropped,
"as a call raced against a timeout is" (`crates/marley_workbench/src/process.rs:14-16`). Each
caller picks its own deadline; there is no default one.

### 2.7 Observability

**What it is.** One server-side model: pretty logs to stdout, completed spans to a local NDJSON
trace file, and optional OTLP export (`docs/operations/observability.md`).

**How it works.** `t3 trace summary --since 30m` prints counts, rates and latency percentiles per
span name straight from the trace file, so it works while the server is stalled. An event-loop
monitor records a `server.eventLoop.stall` span when the loop blocked for more than 2 s, with CPU
and page-fault counts to tell JavaScript work from disk waits (`observability.md:90-120`).
`SIGUSR2` writes a V8 heap snapshot. `T3CODE_OTEL_SDK_DISABLED` and `OTEL_SDK_DISABLED` turn off
every export (`observability.md:617-632`). Span attributes carry high-cardinality detail and metric
labels stay low-cardinality (`observability.md:488-520`). The relay (T3 Connect) reports sweep
counters as span attributes in Axiom (`docs/operations/release.md`, "Managed tunnel cleanup
rollout").

**Marley today: has part.** Marley has `Marley.log`, the launcher's `stderr.log` (guide, Start it
and update it), Zed's hang detection and in-app mini profiler (`crates/miniprofiler_ui`), and the
System One calls log. It has no span file of its own operations (MCP calls, browser actions, git
and `gh` reads). That would help only once something is slow; not ranked.

### 2.8 Product telemetry

**What it is.** PostHog events from the server, on by default.

**How it works.** `T3CODE_TELEMETRY_ENABLED` defaults to true
(`apps/server/src/telemetry/AnalyticsService.ts:76`). Identity is a hashed provider account id or an
installation id; events carry provider, model, effort, permission mode, turn result, duration and
token totals, never prompts or conversation ids (`docs/user/telemetry.md`). Each event gets a uuid
so retries can be told apart, and a batch is dropped after five failed sends
(`AnalyticsService.ts:55`), because "Without these limits, one stuck batch was sent every second for
days" (`docs/internals/product-analytics.md`, "Delivery").

**Marley today: has the opposite.** Telemetry is off by default (#514). Nothing to take.

### 2.9 Tests: real paths, replayed transports

**What it is.** A testing strategy that substitutes only the outside world.

**How it works.** `docs/orchestration-v2/testing-strategy.md` allows replacing the provider
transport, the clock, randomness, the filesystem and the database, and forbids mocking the
orchestrator, adapters, normalizers, event sink, projections or checkpoint policy. Replay
transcripts are raw provider frames with outbound expectations, recorded against the real SDKs:
`record:codex-replay` (App Server JSON-RPC), `record:claude-replay` (Claude Agent SDK `query()`
messages) and `record:cursor-replay`. Fixtures carry protocol, provider version, model and capture
time. The doc names "roughly ten strong tests" first (one message, multi-turn, steering,
interrupt, subagents, rollback, approvals, provider switch). Server tests run in six CI shards
(`ci.yml:231-238`). A live verifier, `apps/server/scripts/verify-background-live.ts`, runs the
production server against real Claude with real usage and keeps an evidence directory with the
database, logs, events and a verdict, and has a `--fail-gate` mode that must exit nonzero
(`docs/operations/background-verification.md`).

**Good.** Recording the protocol, then replaying it through production code, tests the adapter
against what the agent really sends. The live verifier's planted failure checks the checker.

**Marley today: has part.** Marley proves each change with an e2e visual check (§7), keeps a
golden set (#517, `just regress`), and runs mutation passes by hand (#636, #638). Scenarios use
stand-in `claude` and `codex` programs on the PATH. Nothing records a real agent's protocol for
replay. Orca report 07 (row 6) proposed captured PTY transcripts for screen-reading rules; T3's
refinement is to record the structured protocols Marley now reads (Codex's App Server since #650,
Claude Code's hook events since #519), tagged with the agent's version. Item 4.

### 2.10 Test data from real data, copied one way

**What it is.** `vp run migrate-dev-db` builds a worktree's dev database from a read-only snapshot
of the developer's real one.

**How it works.** The header of `apps/server/scripts/migrate-dev-db.ts` (lines 3-25) states the
contract: keep recent projects and threads that have fully stopped; skip working, settled and
archived threads and anything with pending recovery; drop scheduled tasks and queued effects "so
the dev server never adopts live work"; drop auth sessions, pairing links, command receipts and
provider runtime rows. The deletes are at lines 330-358. `AGENTS.md` lists "Writing to the live
install" as the second of three ways to hurt yourself, and adds "Copy in, never symlink. Data flows
one way: into your sandbox, never back out."

**Marley today: has part.** Every e2e scenario runs the debug `marley` on a copy of the user's
profile (§7). `script/e2e.sh:626-662` turns off what would act: Rusty (its service pointed at a
port nothing listens on), Voice, Codex's App Server, Claude Code's IDE link and agent prompts in a
tab, and points `MARLEY_RUSTY_MCP` and `MARLEY_CODEX` at nothing. It leaves on what came later or
was not named: `marley.push` (#535), which can send a line to Chad's phone through ntfy when a
scenario's Claude Code stand-in finishes in a terminal not in front; `marley.embedded_harness`
(#632), which starts `rh serve`; `marley.harness` (#534) and `marley.fleet.hosts` (#610), which
read the real harness and, while the Fleet panel shows, SSH into the real hosts every 5 s, and so
put real sessions and host names into a run's shots; and `marley.system_one` (#565, #548), which
reads the real key, though its project list keeps the scratch repositories from sending. The
regression runner goes through the same script (`script/regress:51`). A copy that turns off a list
of known actors goes stale with each new actor. Item 1.

### 2.11 Lint rules and review agents

**What it is.** A custom oxlint plugin, knip for unused code, and LLM review checks kept as files in
the repository.

**How it works.** `oxlint-plugin-t3code/index.ts` registers eleven rules, among them
`require-suppression-reason` (a lint or type-checker suppression must say why),
`no-test-in-loop` (use `.each`), `no-native-title-tooltip` and `no-hermes-unsupported-apis` for the
React Native engine. `vp run knip:check` fails CI on unused files and dependencies
(`docs/operations/development.md`, Unused code). The review agents live in
`.macroscope/check-run-agents/`: each file has frontmatter naming a model, effort, `include` and
`exclude` globs, a per-run and per-PR budget, and the body is the rubric. They review changed lines
only ("older code in the same file that predates these conventions is not a finding"). The
author's doc and the reviewer's rubric are kept in step by rule: "The Effect Service Conventions
review check enforces the same rules; keep the two in step" (`effect-services.md`, opening). The
UI rubric names a reference component set to imitate (`ui-consistency.md`).

**Marley today: has, mostly stricter.** gate:12 refuses an `#[allow]` without a justification,
gate:9 runs cargo-shear, gate:20 semgrep, gate:21 Zed's dylint lints, and gate:22 ratchets spawn
sites (`script/gates.sh`). The review is §18.1, one generic review of the diff at the end of Code,
with optional critics. Rubrics scoped by path, read by both the author and the reviewer, do not
exist. Item 6.

### 2.12 The contributor contract: AGENTS.md and CONTRIBUTING.md

**What it is.** The rules an agent reads before changing T3 Code (`CLAUDE.md` is one line,
`@AGENTS.md`).

**How it works.**

- "The three ways to hurt yourself" (`AGENTS.md:59-63`): killing by pattern ("Your own agent
  process has this worktree's path in its argv"), writing to the live install, baking origins into
  the bundle.
- "Hit every surface" (`AGENTS.md:65-76`): entry points (chat, Settings, command palette,
  keybinding), clients (web, desktop, mobile), providers (a decision per adapter, "even if the
  decision is 'not supported here'"), agents (MCP reach), contracts, reverse states ("Snooze needs
  unsnooze. Close needs reopen. A one-way door is a bug."), connection modes, docs. The preamble
  says "the most common defect in this repo is a change that works on the path you tested and is
  missing everywhere else", and asks the agent to "say which entries applied".
- Verifying: "Smallest proof that the change works", "Do not run repo-wide checks ... CI owns the
  full suite" (`AGENTS.md:96-105`).
- Plans and research notes are never committed; "A merged PR is the implementation record."
- `CONTRIBUTING.md` sets one problem per PR, prior maintainer approval for features, and evidence
  ("A checkbox or 'tests pass' alone does not show that the change works"). An automated triage
  agent loads the policy and the exemption list from `main` at a resolved SHA, so a PR cannot
  change the rules it is judged by.
- `AGENTS.md:41`: "Most T3 Code contributions will come from T3 Code itself, often controlled
  remotely."

**Marley today: has part.** CONSTITUTION.md is stricter on process (phases, hooks, the gate receipt
of §15, provenance in §20). The guide already says to find a hung Marley with `pgrep -x marley`, by
exact name. What Marley lacks is the surfaces checklist: features here are often shaped by one
layout, one agent or local projects only, and the gaps are found later (#552 brought Claude Code's
notifications to Codex and OpenCode, #617 added rows under closed projects). Item 5.

### 2.13 Documentation rules

**What it is.** A split between internal notes for decisions and traps, and user docs per feature.

**How it works.** `AGENTS.md` ("Documentation"): internal docs answer "what a maintainer would get
wrong without it"; never "narrate control flow, maintain file catalogs, or append PR summaries".
User docs give "each major feature a concise section explaining what it does, how to start, and
anything unintuitive", in the product's voice, updated "when how to use it changes". "When a
documented decision or constraint changes, rewrite or remove the affected text. Do not append
another account of the new behavior." Docs is also the last entry of "Hit every surface".

**Marley today: lacks the user-docs half.** §21 requires a CHANGELOG entry (hook-enforced) and the
architecture docs, but not `docs/marley/guide.md`. The guide still says it describes Marley "as of
2026-09-25 (through #516)" (`docs/marley/guide.md:8`), and its "What is planned" table lists
TICKET-517, 519, 520, 503, 504, 518, 505 and 507 as next (lines 2390-2398), all shipped. Its
Limits of the Browser tab still says "Every tab shares one Chromium profile" (line 1682), which
#507 changed. Shipped features with no word in the guide include `terminal_run` (#556), phone
pushes (#535), Resume Claude Code Sessions (#540), the close guard and Undo (#550) and the block
filter (#528). Chad tests by walking the guide and walkthrough, so a stale guide sends him to
wrong stops. Item 2.

### 2.14 Release engineering

**What it is.** Three trains (stable, nightly, preview), six desktop artifacts, a self-contained
CLI and a hosted web app, from one workflow (`.github/workflows/release.yml`,
`docs/operations/release.md`).

**How it works.**

- Nightly: a scheduled check every 30 minutes builds when there are new commits and six hours
  have passed since the last nightly (`release.md:354`). There were 108 nightlies in the last 30
  days and 540 in all; 41 stable tags (v0.0.2 to v0.0.45).
- Stable promotes the verified nightly: "A manual stable release builds the commit of the latest
  published nightly, not `main` HEAD" (`release.md:18`), so merges during verification never leak
  in. Release-note line, v0.0.39: "feat(ci): ship stable releases from the latest nightly commit
  (#10410)".
- Preview builds any branch through the full sign, notarize, smoke and publish path, with no update
  feed and an npm dist-tag nothing resolves, so an unmerged change gets a real release run.
- The CLI ships as a Node single-executable archive per platform with `SHA256SUMS`; every runtime
  T3 manages (SSH environments, the boot service, `t3 update`, the install scripts) downloads and
  verifies that archive, and "nothing in the product installs from npm". Each archive is executed
  on its build runner before upload (`scripts/smoke-cli-archive.ts`).
- Server self-update invariant: servers update to the client's exact version, so the CLI is
  published to npm before the desktop release and the hosted web app move (`release.md:367-394`).
- The desktop app checks for updates but never downloads without a click (`release.md:402`).
  Linux gets an AppImage and a `.deb` that updates itself; AUR packages `t3code-bin` and
  `t3code-nightly-bin` are maintained in `packaging/aur/` and pushed by `publish-aur.yml`.
- A PR labelled `preview:mac` gets a signed DMG built so that the signing job never runs PR code
  (`release.md:55-82`).

**Marley today: has what one user needs.** `just install` builds the release `marley`, can run the
golden set first (`--regress`), and prints the commit it installed (`script/install-marley`). The
binary carries its commit through Zed's `ZED_COMMIT_SHA` (`crates/zed/build.rs:49-70`). Orca report
07 (row 2d) proposed keeping the previous binary for a one-command rollback; it is not built.
TICKET-445 (release identity, updater) waits until the first release. Nothing here is worth taking
before then.

### 2.15 `t3 triage`: a broken install handed to the user's agent

**What it is.** A CLI command, first in v0.0.34: "feat(cli): npx t3 triage hands broken installs to
your own coding agent (#6563)".

**How it works.** `apps/server/src/cli/triage.ts` (279 lines) "writes a `context.md` with machine
facts (version, paths, server liveness), then launches claude or codex interactively, seeded with
the playbook from `triagePrompt.ts`" (lines 1-10). The agent's own permission prompts gate what it
runs. Without an agent CLI, prompt and context are written to disk to paste elsewhere. The
playbook (`triagePrompt.ts`, 218 lines) tells the agent to ask the user what went wrong first,
read the context file, fetch a newer playbook from `main` if one exists, clone the source at the
installed tag into a cache, then investigate in order of evidence: the trace file, provider event
logs, then the SQLite database, read freely but written only with the user's permission. The
in-binary playbook "must stay byte-identical to `.github/triage/PLAYBOOK.md`", and a test fails
when they drift (lines 5-12).

**Good.** It turns support into the workflow every user already has, and keeps the knowledge of
where logs live in one maintained text.

**Marley today: lacks.** When Marley misbehaves, Chad or an agent has to know that a dev-channel
panic is in `stderr.log` only, that `Marley.log` is beside it, where the endpoint file and the
browser units are, and which agent versions Marley found (guide, Troubleshooting). Item 3.

### 2.16 Codex protocol shapes generated from upstream

**What it is.** `packages/effect-codex-app-server` (61,024 lines, most of it generated) types the
whole Codex App Server protocol.

**How it works.** `scripts/generate.ts` downloads the JSON Schema files from
`openai/codex/codex-rs/app-server-protocol/schema/json` at a pinned `UPSTREAM_REF` (lines 15-18)
and generates Effect schemas. Each Codex release is a regenerate-and-review step, visible in the
notes: "feat(codex): require Codex 0.156 and regenerate its protocol (#13481)" (v0.0.43) and
"feat(codex): regenerate protocol bindings for Codex 0.159 (#14311)" (v0.0.45). A `probe` script
talks to a real App Server, and `record:codex-replay` records sessions for replay (2.9).

**Marley today: has part.** Marley's App Server client (#650, #651) reads hand-written shapes "the
ones Codex 0.155.1 and 0.158.0 generate as JSON Schema; a field Marley does not read is ignored"
(`crates/marley_agent/src/codex_events.rs:14-15`), and only runs on Codex 0.155.1 to 0.158.0
(`crates/marley_agent/src/versions.rs:101`). The shapes were checked once, at #650's and #651's
Plan, by running `codex app-server generate-json-schema` on both binaries and comparing
inventories, the method rustal-harness keeps in its `scripts/codex_profile.py`
(`docs/planning/pipeline/completed/650-codex-app-server-state.spec.md:235`). The scenarios run a
scripted Python stand-in, not recorded frames (`script/e2e/650-codex-app-server-state.sh:2`). T3
already targets 0.159. When Chad's Codex updates past 0.158, Marley falls back to reading the
terminal until someone repeats the check by hand. Item 4.

### Release notes by month

PR titles are quoted as the notes print them, with the PR number. Counts are PR lines by
conventional-commit type in the stable notes; nightlies between stables are not counted twice.

| Month | Stable releases | feat | fix | perf | refactor |
|---|---|---|---|---|---|
| 08-10 to 09-01 | v0.0.33 to v0.0.38 | 103 | 407 | 21 | 5 |
| 09-02 to 09-30 | v0.0.39, v0.0.40, v0.0.42, v0.0.43, v0.0.44 | 236 | 866 | 83 | 166 |
| 10-01 to 10-02 | v0.0.45 | 11 | 34 | 5 | 1 |

About 3.7 fixes ship per feature. v0.0.39 alone carried 76 features and 376 fixes; v0.0.44 was one
fix ("fix(codex): Pro Max accounts load, so Ultrafast shows up (#14304)").

**August (to v0.0.38, 09-01).** The sidebar and pull requests were rebuilt, and attachments grew.

- Sidebar: "feat: sidebar v2 is now the default sidebar (#5672)", "feat(web): drag pinned threads
  into your own order (#5581)", "feat(web): keep unsent drafts one click away in the sidebar
  (#5777)", "feat(web): show how many subagents are running at a glance (#5745)".
- Pull requests: "feat: multi-provider pull requests page with in-app reviews (#4849)", "feat:
  link pull requests to threads (#8160)", "feat(web): send PR line requests to agent (#6597)",
  "feat: allow disabling auto-settle on merge (#5880)".
- Browser panel: "feat(server): let users withhold browser access from agents (#7083)",
  "feat(desktop): mute a browser tab (#7252)", "feat(desktop): remember recently used sites in the
  Browser panel (#5270)".
- Usage: "feat(usage): usage page reading provider transcripts across environments (#5684)".
- Files and media: "feat(server): accept PDF, ZIP, and other file uploads up to 50MB (#8235)",
  "feat(web): play video attachments in chat (#8688)", "feat(web): safely attach HEIC photos as
  JPEG images (#8161)".
- Claude: "feat(claude): compact old threads before they burn through usage (#8144)", "feat(models):
  discover Claude models from remote manifest (#9084)".
- Mobile: "feat(mobile): add offline iPhone voice input (#8614)", "feat(mobile): pick, share, and
  receive files in threads (#8237)", "feat(mobile): add built-in themes (#6619)".
- Engineering: "feat(packaging): maintain AUR packages in-repo (#4128)", "feat(server): run the
  background service on macOS via launchd (#6286)", "feat(cli): npx t3 triage hands broken installs
  to your own coding agent (#6563)", "feat(server): vp run migrate-dev-db seeds worktree dev dbs
  with real data (#5773)", "feat(desktop): build macOS previews from a PR label (#8182)",
  "feat(lint): ban native title tooltips and migrate to styled Tooltip (#7209)".

**September.** Continuity, limits, more machines, devices, and the release pipeline.

- Continuity: "feat(updates): continue active threads across server restarts (#9167)", "feat:
  rewind conversations while keeping file changes (#11358)", "feat(web): choose queue or steer for
  follow-up messages (#11964)", "feat(codex): support async questions (#9512)", "feat(threads):
  dismiss async questions without replying (#10431)", "feat(chat): attach files to question answers
  (#9871)".
- Providers: "feat(providers): add Google Antigravity via the official ACP agent (#9348)",
  "feat(providers): add context compaction across harnesses (#8808)", "feat(providers): expose
  native slash commands across clients (#11519)", "feat(chat): show provider thinking traces
  (#11784)", "feat(providers): check remote compatibility ranges (#13130)", "feat(codex): connect
  ChatGPT accounts with managed authentication (#14290)".
- Usage and limits: "feat(usage): show Codex and Claude subscription limits on a Limits tab
  (#9507)", "feat(usage): pool subscription limits per provider across accounts and environments
  (#10300)", "feat: show provider usage limits with /usage-limits (#9875)", "feat(server): show and
  redeem Claude banked resets (#13118)".
- Machines: "feat(connections): balance new threads across connected machines (#9895)",
  "feat(desktop): allow disabling the local environment (#9194)", "feat(desktop): run the WSL
  backend from the Linux CLI archive (#11511)".
- Devices: "feat(devices): add simulator and emulator support (#10677)", "feat(devices): connect
  simulator hosts over SSH (#10856)", "feat(web): add an interactive 3D device workspace (#12787)".
- Pull requests: "feat(pull-requests): link multiple pull requests to threads (#10839)",
  "feat(prs): navigate, merge and rebase GitHub stacks (#10875)", "feat(source-control): support
  Forgejo and Gitea with fj and tea (#11436)", "feat(web): pull request files can be marked as
  viewed (#7721)", "feat(web): sort pull requests by what is blocked on me (#12508)".
- Composer and threads: "feat(web): enable rich text composer by default (#12160)", "feat(web):
  cite assistant responses with inline citations (#9146)", "feat(web): recall sent prompts with the
  up arrow (#9173)", "feat(composer): fold large pastes into text attachments (#11442)", "feat(web):
  undo settle, snooze and archive, with a mod+z shortcut (#12848)", "feat(web): start new threads
  with multiple models in separate worktrees (#12179)".
- Capture and browser: "feat(desktop): add cross-platform window capture (#8103)", "feat(desktop):
  browser profiles for the preview browser (#7254)", "feat(desktop): import from Chrome, Edge,
  Brave, Vivaldi, Opera, Arc and Firefox (#7260)".
- Settings and worktrees: "feat(settings): add shared project defaults and scoped overrides
  (#9754)", "feat(settings): add automatic storage cleanup per machine and project (#11598)",
  "feat(web): show each worktree setup step and let users cancel it (#11372)", "feat(web):
  first-run welcome wizard with agent setup and project import (#5362)".
- Terminal: "feat(web): run shell commands from chat in the thread terminal (#13060)".
- Observability: "feat(cli): summarize the server trace file from the command line (#13698)",
  "feat(observability): record event loop stalls in the server trace (#13697)",
  "feat(observability): write a server heap snapshot on SIGUSR2 (#13694)", "feat(observability):
  honor the OpenTelemetry kill switch (#13355)".
- Release: "feat(ci): ship stable releases from the latest nightly commit (#10410)",
  "feat(cli): add t3 update for self-contained installs (#11451)", "feat(server): manage runtimes as
  release archives only, never from npm (#11510)", "feat(release): ship a Linux .deb that updates
  itself (#13575)".
- Mobile: "feat(mobile): add Android agent notifications and ongoing activity (#10416)",
  "feat(mobile): add pooled subscription usage widgets (#11506)", "feat(mobile): view and control
  agent devices (#12531)".

**October (to 2026-10-06).** v0.0.45: "feat: start threads without a project (#13612)", "feat:
start a new project from just a name (#14527)", "feat(web): restart the agent session from cmd+k
to load new skills and plugins (#14542)", "feat(web): update providers on every machine with one
click (#14678)", "feat(web): beta Working section hides busy threads until they need you (#13926)".
After it, on `main` and in nightlies only: the V2 orchestrator (#2829, 2.3) and
"feat(preview): run the browser on the environment server (#15328)" (2026-10-06).

## 3. Bring to Marley

Ranked by usefulness to Chad's daily work. Size: S a day, M a few days, L a week or more.

1. **Test runs that act on nothing.** *Why.* Every e2e and golden run starts the debug `marley` on
   a copy of Chad's profile, and the copy still carries ntfy pushes to his phone, the embedded
   harness, the real harness and fleet hosts, and System One's key (2.10). A scenario whose agent
   stand-in finishes a turn can push to the phone today, one that opens the Fleet panel puts real
   host names into its shots, and each new actor repeats the gap. T3's `migrate-dev-db` states the
   contract the other way round: the copy keeps what a run needs and drops anything that would act
   ("so the dev server never adopts live work"). *Seam.* `script/e2e.sh`'s settings block (lines
   643-660): keep an allowlist of look-and-feel keys (layout, theme, fonts, keymap, terminal and
   block display) and drop every other `marley` key, so a scenario turns a feature on with a
   stand-in as Rusty's already do. Also unset `MARLEY_SYSTEM_ONE_KEY` and
   `MARLEY_CLOUDFLARE_API_TOKEN` for the run, and give a scenario that needs System One the `replay`
   provider, which answers from a file. *Size.* S. *Hard.* A scenario that relied on a
   user setting surviving the copy must now set it; the golden set's 53 scenarios need one run to
   find them.

2. **The guide changes with the feature.** *Why.* Chad tests Marley by walking the guide and the
   walkthrough. The guide's header, its planned table and its Browser limits are stale, and five
   shipped features have no entry (2.13). §21 enforces the CHANGELOG and not the guide. T3's rule is
   that a feature's user doc changes in the same change, rewritten in place, and its "Hit every
   surface" list ends with docs. *Seam.* §21 gains a third duty: the guide section the change
   affects, rewritten (not appended), and the walkthrough stop when one exists;
   `docs/planning/pipeline/_templates/pipeline.notes.md` gets a "Guide:" line naming the section or
   saying why none applies. A one-off pass fixes today's drift: the header line, "What is planned",
   Browser limits, and entries for #528, #535, #540, #550 and #556. *Size.* S, plus about a day for
   the catch-up pass. *Hard.* The guide is 2,411 lines; rewriting in place means finding the
   section, which a heading index in `docs/marley/README.md` would ease.

3. **`marley: triage`.** *Why.* When Marley misbehaves, the facts an agent needs are spread over
   `stderr.log` (the only record of a dev-channel panic), `Marley.log`, the endpoint file, the
   browser units, the agent versions Marley found and the commit it was built from. T3's
   `t3 triage` writes those into a context file and starts Claude Code or Codex with a playbook
   that asks the user first, reads evidence in order, and files a report (2.15). *Seam.* A palette
   command (and `marley --triage` for when the UI is gone) in `marley_workbench` that writes
   `context.md` into the data directory: `ZED_COMMIT_SHA`, data and log paths, the tail of both
   logs, `systemctl --user list-units 'marley-browser-*'`, #648's version readings, and
   `settings.json` with secrets masked by #516's redactor. It opens Claude Code in a terminal of the
   Marley checkout with `docs/marley/triage-playbook.md` as the prompt; the playbook ends by
   writing a file under `docs/planning/intake/`, never a public issue. *Size.* S to M. *Hard.*
   The triage terminal runs inside the Marley being triaged; `marley --triage` must work with no
   window, so it opens the system terminal instead.

4. **Codex's App Server, checked at each Codex release.** *Why.* Marley reads Codex's state and
   answers its approvals only on Codex 0.155.1 to 0.158.0 (2.16). Codex updates itself; past
   0.158 the rail and inbox lose Codex's real state until someone repeats #650's schema comparison
   by hand. T3 makes the same comparison a routine step of every Codex release and records real
   sessions for replay. *Seam.* A script, `script/codex-schema-check`, that runs the installed
   Codex's `codex app-server generate-json-schema` (reusing rustal-harness's `codex_profile.py`
   method) and compares the fields `crates/marley_agent/src/codex_events.rs` and #651's request
   types read; no change in those fields means the range in `versions.rs` can widen after one e2e
   run, and the agent bar's "Untested Codex" chip could name the script. Second step: a recorder
   that saves a real App Server session as JSON lines, replayed by the Python stand-in that #650's
   and #651's scenarios already use, so they test real frames. The same recorder shape suits
   Claude Code's hook-event stream (#519) under #648's version table. *Size.* S for the check, M
   with the recorder. *Hard.* `generate-json-schema` is an experimental Codex subcommand; keep
   reading upstream's `schema/json` at the release tag as the fallback.

5. **A surfaces checklist in the plan template.** *Why.* T3 names "a change that works on the path
   you tested and is missing everywhere else" its most common defect, and asks each change to say
   which surfaces applied. Marley's equivalents recur in its own history: features that work in
   the Marley layout but not Zed's, for Claude Code but not Codex, for local projects only, or with
   no way back out. *Seam.* `docs/planning/pipeline/_templates/pipeline.spec.md` gets a "Surfaces"
   block, each line marked applies, not applicable, or later with a ticket: entry points (palette,
   key, rail menu, settings page, `settings.json`), layouts (Marley, Zed), places (local project,
   remote project, group with no folder, closed project, worktree), agents (Claude Code, Codex,
   Gemini CLI, OpenCode, the Agent Panel), agent reach (an MCP tool, or why not), reverse states
   (the way out and the way to see it), and docs (item 2). *Size.* S. *Hard.* None; it costs a few
   minutes per spec.

6. **Review rubrics by path, read by author and reviewer.** *Why.* §18.1's review is one generic
   pass. T3 keeps each rule set once: the author reads `effect-services.md`, and a reviewer scoped
   by globs to the changed lines reads the same rules. Marley has rules of this kind scattered
   over `.rules`, §14, the zed-touchpoints ledger and the prevention rules. *Seam.* A
   `.claude/review/` folder of rubrics with `include` globs: touchpoint minimality for any path
   outside `crates/marley_*`; GPUI re-entrancy and errors reaching the UI for `marley_workbench`;
   no-ai-slop for `docs/**` and `CHANGELOG.md`; redaction for `marley_mcp`. `/pipeline:code`'s
   review step loads the rubrics whose globs match the diff. *Size.* S. *Hard.* Rubrics drift from
   the docs they summarise; T3 states "keep the two in step" and Marley would need the same line in
   each.

## 4. Skip

- **The event-sourced orchestrator, receipts and the outbox.** Marley does not own agent sessions;
  the CLIs and rustal-harness do. Hand the harness three lessons: command receipts keyed by a
  client id make retries safe, a publish lane keeps subscribers in sequence order, and a turn's end
  is a different milestone from its finalization.
- **Effect-TS service conventions, the oxlint plugin and knip.** TypeScript-specific; Marley's
  gates already hold the equivalents (gate:9, gate:12, gate:20 to 22).
- **Wire transfer budgets and their PR report.** Marley has no client and server on a wire; its MCP
  tools already cap their output.
- **A server-wide span file and `trace summary`.** Useful once something is slow; Zed's hang
  detection and logs cover what Chad has hit so far.
- **Product telemetry.** Marley keeps telemetry off (#514).
- **Three release trains, signing, npm OIDC, AUR, auto-update and the self-update invariant.** One
  user who builds from source; TICKET-445 holds the release identity until the first release.
- **Hosted manifests** (model classification, compatibility ranges, a playbook fetched from
  `main`). They exist so old installs learn without an update; Chad rebuilds.
- **Contribution triage automation, vouch lists and label-gated signed previews.** No outside
  contributors.
- **Deterministic clock and randomness in tests.** §7 retired per-ticket unit tests; revisit in the
  testing phase.
- **The 10,939-line `Orchestrator.ts` and a 380,000-line squash merge.** Both are what Marley's
  small-change rule exists to avoid.

## 5. Open questions

1. **Test-run settings.** Should an e2e copy keep only an allowlist of display settings (item 1),
   or keep today's denylist and add push, System One, the two harness keys and fleet hosts to it?
   *Default: add those five to the denylist now; the allowlist waits for Chad's answer.*
2. **Enforcing the guide.** A Stop or commit hook that blocks Complete without a "Guide:" line, or
   the template line alone? *Default: the template line, no hook.*
3. **Where triage reports go.** A file under `docs/planning/intake/`, a Rusty capture, or an issue
   on the public `Ignibyte/marley_ide` after a privacy scan? *Default: the intake file; nothing
   leaves the machine.*
4. **Widening Codex's version range.** Automatically when the schema check finds no change in the
   fields Marley reads, or only by hand after an e2e run? *Default: by hand, with the check's
   output in the ticket.*
