# Jev and a System One layer for Marley, 2026-09-25

Chad asked for "a jev first class support (or set up for system 1 models) that can do this or
interact real time for decision making", and to "research typesafe and jevai in general rather
than examples." Three agents read TypeSafe's published material, the gateways that resell Jev,
the independent tests since the launch, rustal-harness's docs and Marley's code, and checked
TypeSafe's claims against two earlier experiments of Chad's that are not public ("Chad's runs"
below); nothing was built or run. Jev answers typed questions about a state with probabilities, in
0.2 to 0.6 seconds, for under a tenth of a cent, and never writes text, plans or counts. Higher
confidence is more often right, but the numbers are not frequencies, so every threshold is fitted
on Chad's own labels, with an abstain band and a "cannot tell" option. The proposal is one Marley
crate for typed decisions, off by default, that starts every use in shadow mode, logs every call
for replay, and lets a model reorder, mark, route and refuse, never approve anything irreversible.
The first uses are the kind of stop an agent made, `browser_find` and `terminal_find` for agents,
and a needs-you order with risk chips in the approvals inbox (#508), where Claude Code's own
reviewer keeps the verdict. Every use on costs about 25 cents a day; data leaving the machine is
the constraint.

## What Jev is

TypeSafe AI launched Jev, its first "System One" model, on 2026-09-15 in early access. Such models
are "built to make fast, structured decisions that software can use directly" and "do not write
replies, produce code, or generate explanations of their reasoning"
(docs.typesafe.ai/concepts/system-one). Jev's training method (RLCD), weights and size are
unpublished, it cannot be self-hosted, and TypeSafe says it "is not a drop-in replacement for the
LLM behind Claude Code" (docs.typesafe.ai/introduction/coding-agents). One model, `jev-1.13.0`, sits
behind the aliases `jev-latest` and `jev-preview`; the models page says to pin the id once
thresholds are tuned (docs.typesafe.ai/models), and a cookbook ran on `jev-1.12` a month earlier.

The API is one call, `POST https://api.typesafe.ai/v1/systemone`, with a bearer key and
`{model, state, questions}` (docs.typesafe.ai/api). Every question is judged "in parallel and in
isolation against the same state", so one request carries everything an event needs.

| Type | Asks | Returns | Rules |
|---|---|---|---|
| `noul` | is a statement true | the probability of true, no confidence | one condition, phrased so high means yes |
| `choice` | one of up to 255 options | the choice, probabilities that sum to 1, a confidence | add a "none" option when the list may not cover the case |
| `score` | a level on a rubric of 2 to 10 | the probability-weighted level, probabilities, a confidence | each level judged alone; "Describe situations, not degrees" |

A request holds 64k tokens, 32k of them for the state and the longest question; an account gets
1,200 requests a minute; the Python SDK retries 429 and 5xx twice, 30 s in all
(docs.typesafe.ai/sdk/python/api/retries). There is no official Rust client; the community guide
(github.com/AbdelStark/awesome-typesafe-jev) lists `s1-rs`, with typed enums and a
`Policy::act(0.85).review(0.6)` gate (MIT OR Apache-2.0, needs tokio). A three-option choice's
confidence is (3 × top probability − 1) / 2 (docs.typesafe.ai/confidence), which 232 of Chad's
recorded three-option answers match within 0.005 on average; if the form holds for n options, a
0.6 floor needs a top probability of 0.80 with two options and 0.63 with twelve.

| | TypeSafe says | Measured in public | Chad's runs |
|---|---|---|---|
| Latency | "End-to-end response time is 70ms-500ms" (launch post) | 239 ms median from France (beri.net); p50 376 ms, p95 545 ms (the action-gate study); p50 0.65 s from Germany (JevBench v1.2); a 791-call run through OpenRouter with a 0.33 s median and a 1.42 s slowest call | 163 to 183 ms at about 1.8k tokens; 348 to 366 ms median, 462 to 551 ms p99 at about 8k tokens over 572 calls; never the 70 ms floor |
| Batching | 13 questions in one call: 0.27 s and $0.000497; one by one: 2.71 s and $0.006090, same answers (cookbook `parallel_questions`) | | 23 questions a call at about 360 ms |
| Price | $0.042 per million input tokens, output free | $0.0000268 a decision (the action-gate study) | about $0.0004 for an 8k-token request |
| Repeatability | noul sd 0.0102 over 15 repeats; the top choice flipped on 2 of 8 questions; abstaining under 0.60 lifted agreement from 90.8% to 99.2% at 74.2% coverage (consistency cookbooks) | no decision changed on repeated runs (the action-gate study) | the same inputs asked again: 90% identical |
| Calibration | "higher confidence means higher accuracy", though it "does not guarantee that an individual answer is correct"; no reliability curve published | ECE 0.107 out of distribution, choices and scores overconfident, right 44.7% of the time at a mean probability of 0.74 on unknowable items (beri.net); with an "unknown" option it abstained on 95% of ambiguous items, without one it picked the stereotype 79% of the time at confidence 0.793 (the KoBBQ audit); a top probability of exactly 1.0 on 56.4% of answers, errors included (jev-certify) | at confidence 0.7 or more, within one rubric level 94 to 96% of the time, under 0.5 only 77 to 83%; a noul between 0.2 and 0.6 was a coin flip; the best threshold ranged from 0.20 to 0.70 by question |
| Accuracy | "similar levels of intelligence on System One tasks compared to existing LLMs, while being two orders of magnitude faster"; 67.8% at 0.4 s in its workflow evals, against an answer key averaged from GPT-6 Astra and Claude Fable 5.1 (evals.typesafe.ai) | JevBench v1.2: top composite of 48 systems, 74.4, and 74.1 on the hard tier where GPT-5.6 Luna scores 94.5. Phishing: one broad question 62.6% against Claude Haiku 4.5's 81.3%, five narrow ones with a fitted regression 95.0% against 93.2%. A 111-case agent action gate: Jev 100 right, Claude Opus 5 102, one unsafe allow each | a written plan's priority followed on 1,536 of 1,536 turns; rewriting only a rubric's level text moved agreement with a human reviewer from 72% to 88% within a level |

Claim and measurement agree on ordering, so thresholds work; they disagree on the numbers, and every
source that set thresholds set them per question. Jev comes near a frontier model when a decision is
split into narrow questions with written criteria, and falls about 20 points behind on one broad
question or a long conditional policy. The criteria text is the instrument. Rewording option
descriptions moved one option from 1% to 19% of picks, and a rule on the question is followed where
the same rule in a preamble is not (0.87 against 0.34). A phrase computed in code ("your strongest
option is X") helps where numbers in the state hurt. Agreement stopped improving by about 600
labels. Jev forms no plan, counts and compares numbers and dates badly, reads text only, loses
accuracy as unrelated state grows, "does not treat it [state] as hostile by default", and cannot say
"don't know" without an option for it (docs.typesafe.ai/model-jaggedness/jev-1.13).

Other models serve the same call. JevBench v1.2 ranks SemIf at 73.1, djev (Apache-2.0,
self-hostable) at 73.0 and Reflex 4B at 70.3; Reflex (github.com/kshetrajna12/reflex) needs an
NVIDIA GPU, and Laya (github.com/receptron/laya) runs on a CPU with a 512-token state and under
about 20 options; Chad's machine has no discrete GPU, so a local model would suit only short
questions. TypeSafe does not train on inputs (typesafe.ai/legal/privacy-policy) and keeps them
"for as long as necessary taking into account the purpose of the Processing"
(typesafe.ai/legal/data-processing); zero retention is an enterprise contract, while Cloudflare
Workers AI, one of three gateways that resell Jev, states it
(developers.cloudflare.com/ai/models/typesafe/jev/). Some uses send what an agent already sends its
own provider; others send what nothing sends today (shell history, a dev server's output).

## The decision layer

One crate, `marley_system_one`, a pure core with thin adapters (plan D8, D13), named after the
request shape with Jev as its first provider. Its rules come from an earlier Jev wrapper Chad uses
and from his earlier ground rules for using Jev (2026-09-23). Question sets are compiled in, named,
versioned and insert-only, with a pinned model id, and a caller never supplies its own. The host
builds the state from facts computed in code, never the caller being judged, and a state empty by
design is a refusal, since it breeds false positives. Every call is a row, failures included, so an
unreachable model and a model that found nothing never look alike. Secrets are masked before the
request, and the masked copy is what is stored. A Jev answer never replaces a deterministic check
and may only add a refusal or an escalation. Model trouble reaches a feature as a `NoSignal` or
`Unavailable` reading with a reason, never as an `Err`. The providers are `typesafe` (or any
`/v1/systemone` server), `openrouter`, `rules` (the deterministic control arm) and `replay`. Unlike
that wrapper, which cannot turn Jev off, Marley ships every feature off, as it ships telemetry off.
HTTP goes through Zed's `http_client`, so there is no tokio runtime, and the client borrows
`s1-rs`'s typed surface under its licence. Settings sit under `marley.system_one` in
`crates/settings_content/src/marley.rs` (off, a pinned model, the projects that may send state, each
feature's mode, a daily budget), and the settings page shows the key's source, never its value.

**Shadow mode first.** A feature is `off`, `shadow` (ask and log, show nothing outside the
Decisions view), `suggest` (show the reading for Chad to confirm with one key) or `act`. Every
feature starts in shadow, where Chad's next actions label the calls for free: which approvals he
allowed, whether he replied after a "finished", whether he stepped in after a "looping?" flag,
whether an agent clicked what `browser_find` returned. A feature moves to `suggest` when its golden
report passes (300 to 600 labels split into tuning, holdout and fresh sets; precision and recall
per class, coverage per threshold, the `rules` provider beside Jev). `script/system-one-eval.sh`
spends real calls, since a check proven against a stub proves only the stub. It is no test gate
under CONSTITUTION §7, though a golden report could join #517's machine checks, and each
feature's e2e scenario runs on the `replay` provider.

**The coach-and-executor playbook.** In an earlier experiment of Chad's, a turn-based game, a coach
model wrote each side's plan and Jev chose each turn. Jev followed the plan on 1,536 of 1,536 turns
and formed none of its own; the same plans run by code alone scored 67.0% against Jev's 67.5%, and
turns no rule covered were lost until every plan ended in an unconditional floor rule. For agent
work a playbook is a versioned rule list over `fact.*` (computed in code: agent kind, foreground
program, the turn's exit codes, the diff stat, quiet time in words) and `jev.*` (the sealed set's
recalibrated answers), with actions from a fixed set: show, rank, route to the owner or the
manager, reply with a fixed sentence, escalate, ask Chad. The first matching rule wins.

```json
{ "name": "question-route-v3", "questions": "question-route/4", "rules": [
  { "when": ["fact.names_credentials_or_money"], "do": "owner" },
  { "when": ["fact.options_all_reversible", "jev.answerable_from_ticket >= 0.85", "jev.route.margin >= 0.15"], "do": "reply:proceed" },
  { "when": [], "do": "manager", "say": "floor: the manager reads it" } ] }
```

With Jev unreachable every `jev.*` predicate fails and the event falls to the floor, which is
today's behaviour. The coach is a Claude Code or Codex session briefed from the catalogs, or a
threshold search over the labels; a revision must beat the sealed version on the same logged events
and on a fresh set, and stays data until Chad seals it, outside every tree an agent can write.

**Logging and replay.** One JSON line per call in `~/.local/share/marley/system_one/`, never in a
project: set and playbook versions, the rule that fired, the model that answered, the masked state
as sent, every probability, the reading, the thresholds, the action, latency, cost and any error.
Outcomes arrive later as lines naming the call id; no row is rewritten. A Decisions tab shows each
call beside what Chad did ("would have routed to the manager (0.91)"), and a status item shows the
day's calls and spend and turns red on failures.

**Abstain bands and thresholds.** A choice or a score needs confidence at or above its floor, 0.5 to
start. A noul inside 0.35 to 0.65 is no signal, the band widened by 1e-9 so floating point cannot
read an edge as clear. Every choice the state may not settle has a `cannot_tell` option, and a noul
never detects "don't know", since in the earlier wrapper's tests an unknowable noul came back at
0.34, a confident no. A threshold belongs to one question, model id and set version, fitted after
recalibration as the lowest value that meets the feature's target, and a bin that authorizes action
needs 200 labels or more (prefactor.tech/blog/jev-calibrated-confidence-is-not-correctness).
TypeSafe's 0.6 floor and 0.85 act line (docs.typesafe.ai/patterns/confidence-routing) are starting
points. Drift breaks bounds (jev-certify's certificate failed 3.6 times over when out-of-scope
traffic rose from 13% to 43%), so the log counts each class and a shift triggers a refit.

## Ranked uses across Marley

| # | Use | Questions | What the answer changes | Waits on | Size |
|---|---|---|---|---|---|
| 0 | The layer, its log, replay and settings | | nothing on screen | #519, #520 (#516 has shipped) | M |
| 1 | What a stopped turn needs | a choice (done and checked, done by claim only, asks Chad, blocked, still going, cannot tell); a noul per part of the last prompt | the rail row's word, the notification's wording ("asks you"), the order | #519, #538, #542 | S |
| 2 | `browser_find`, `terminal_find` | a choice over a page's refs or a block's tagged lines, with `none`, and a `present` noul | the agent gets the top three, or "not sure: read the snapshot", instead of a 7,500-token snapshot | nothing | S each |
| 3 | Inbox order and risk chips | an urgency score; nouls for destroys, outside the project, sends data out, credentials, rewrites history, installs, claims approval | orders #508's inbox and marks entries; approves nothing | #508, #519 | S–M |
| 4 | Stalled or looping | a choice (long task, waiting for input no hook reported, stuck on a lock or the network, frozen, cannot tell); nouls for repeating and progress | a flag on the row, later an escalation to rustal-harness's foreman; never a stop | #519 | M |
| 5 | Who answers a question | owner-class categories by rule, then a choice (owner, manager, agent proceeds, cannot tell) | ranks the inbox; routes once rustal-harness M10's manager runs | #508, #534 | M |
| 6 | A pause before a consequential click | nouls for pays, deletes, sends in the user's name, changes an account | asks in the Browser tab's Agent chip, by default only when the agent runs without its own prompts | #532 | S–M |
| 7 | A running command's error | nouls for a new failure and for recovered | a notification when a dev server prints an error and keeps running | Warp second pass, finding 3 | S–M |

Public evidence for the top of the list: jev-gates' done check scored finished parts of a task at
0.94 or more and a silently skipped part at 0.03 (github.com/rashedInt32/jev-gates); TypeSafe's
line-by-line search judged 218 tagged lines in one request (docs.typesafe.ai/cookbooks/semantic_find).
Later, each folded into its ticket when specced: turn-diff triage (#509), a pick found again by
likeness (#505), ranking worktree agents on one task without merging (#510, #511), voice commands,
filtering a block by meaning (#528) and jump to the failure (plan T2). The next history command,
English at the prompt and a second opinion on candidate secrets read what nothing sends today, so
they run on a local model or stay off; a candidate secret is never sent out to ask about it.

Approvals are not first because the slot is filled. Claude Code's auto mode is now the starting
permission mode where the account and model allow it (code.claude.com/docs/en/permission-modes),
with a classifier that sees user messages and tool calls but no tool results; Anthropic reports
0.4% false positives on 10,000 real actions and 17% false negatives on 52 overeager ones
(anthropic.com/engineering/claude-code-auto-mode), and Codex has `approvals_reviewer =
"auto_review"` (learn.chatgpt.com/docs/sandboxing/auto-review). Agents keep their own prompts by
Chad's decision, so Jev adds a shadow second opinion, the inbox order, and facts for the classifier
through a PostToolUse hook's `classifierContext`. Jev decides approvals last, only for agents with
no reviewer (rustal-harness actors, Zed Agent threads). It does not fit writing, counting, choosing
agents' tools, screenshots, anything per keystroke, or jobs code does exactly (#503's routing,
#507's profiles, #521's ports).

## Budget

| Loop | Blocks an agent? | Deadline | Calls a day | State | Cost a day | With Jev off or down |
|---|---|---|---|---|---|---|
| Stop kind | no | 2 s | about 300 | up to 4k tokens | about $0.05 | today's words |
| Stall and loop | no | a flag within 30 s | about 100, asked at 1, 2, 4 and 8 minutes of quiet | up to 4k | about $0.02 | the 2 s quiet heuristic |
| Routing | the agent already waits | 2 s | about 50 | up to 3k | about $0.01 | every question to Chad |
| Inbox order, chips | no | 600 ms | about 200 | 1k to 3k | under $0.03 | oldest first, no chips |
| Find tools | the agent's own call | 0.6 s a page, 1 to 3 s for 2,000 lines | a few hundred | a page or a chunk | about $0.10 | the tools are not listed |
| Approvals, reviewer-less agents | yes | 1.5 s hard, one retry | about 500 | up to 2k, no tool output | about $0.04 | the agent's own prompt |

These are five agents over an eight-hour day at $0.042 per million input tokens, about 25 cents
with every loop on. Interactive features plan on 200 to 400 ms (p99 near 550 ms at 8k tokens), and
1.5 s covers the slowest call measured through OpenRouter. Retries happen only inside a deadline;
a daily cap set above a normal day stops a runaway loop; one token bucket keeps Marley under 1,200
requests a minute; a breaker opens for two minutes after five failures in a row. No call goes out
when a session's masked state hashes the same as the last one asked.

## Safety rules

1. Code decides the dangerous classes first: destructive commands, protected and secret paths,
   publishing verbs, and project rules such as one `cargo` at a time (Marley's `CLAUDE.md`). There
   Jev can only add caution.
2. An action that changes something needs two keys: code's classification of it as reversible, and
   a recalibrated Jev probability above the question's threshold with no Jev red flag. Jev alone
   never authorizes (the design in github.com/Glubiz/zirv-cli/issues/781).
3. Irreversible actions always go to a person: merges (#511), pushes, deletions outside the
   worktree, credential use, deploys, spending, messages to people. No playbook action reaches
   them, and the validator refuses a playbook that tries.
4. No destructive command is ever auto-approved. A planted never-allow set of at least 300 items
   (zero false allows in 300 bounds the rate at 1% with 95% confidence) runs on every change.
5. A verdict binds to the exact call: classify last, hash the exact `tool_input`, and apply the
   verdict to nothing else (github.com/langchain-ai/langchain/issues/40694 shows a call replaced
   after it was classified).
6. Hostile text stays out of acting decisions: "the owner approved this" got 3 of 30 dangerous
   commands past a Jev gate (github.com/eugeniughelbur/jev-gate). Uses that read agent output only
   display, rank and route; an acting decision's state holds the call and code's facts, and after
   three automatic refusals in a row the session goes back to Chad.
7. State is masked with #516's redactor and sent only from listed projects; SSH terminals, remote
   projects and environment dumps stay out, and a metadata-only mode sends code's facts alone.
8. The key never goes in settings. Marley reads it from the environment, then the system keyring
   through the keychain provider itself, since on the `dev` channel Zed's credentials provider
   keeps plain JSON in Marley's config directory unless `ZED_DEVELOPMENT_USE_KEYCHAIN` is set
   (`crates/zed_credentials_provider/src/zed_credentials_provider.rs`); #445 settles the label. The
   key is a request header only, and error bodies are cut to 300 characters, as they can echo the
   request.
9. The model id is pinned, and the golden set reruns on every change to a question, a playbook or
   the model and weekly, with an alert when agreement leaves its noise band.

## How it relates to rustal-harness

rustal-harness's M9 gives every Codex session and actor one state envelope in Marley's shape, and
`rh mcp` serves `fleet_snapshot`, `fleet_events` and, under a write grant, `session_answer`,
`session_send` and `session_stop` (its `docs/FLEET.md` and `docs/MCP.md`). M10 plans a foreman
whose mechanics live in the runtime and a manager with an owner inbox; the roadmap's open choice 1
asks whether the foreman also makes judgment calls (`docs/ROADMAP.md`). This note answers it with
the runtime's mechanics, a System One decider for the frequent typed judgments under a playbook the
manager writes, and the System 2 manager for whatever the decider abstains on or may not touch.

| Event | Code | System One | Manager | Chad |
|---|---|---|---|---|
| crash or provider error | error state, bounded restart | nothing | reads the escalation | told when the manager cannot recover |
| a working session goes quiet | measures the silence | long task, waiting on input, stuck, frozen, cannot tell | interrupts, nudges or restarts | on escalation |
| an agent asks a question | owner-class categories by rule | owner, manager, agent proceeds, cannot tell | answers manager-class ones | answers owner-class ones |
| a stop, a merge, the playbook | stale-generation checks, the validator | never | decides a stop, writes the playbook | decides a merge, seals every playbook that acts |

The decider is an MCP client of `rh mcp`, as the manager and Marley are, so policy stays out of the
runtime. It never claims managed input or types into a terminal; its one write is
`session_answer`, and each answer records who gave it. Three requests to rustal-harness follow:
carry an approval's bounded review (command, cwd, paths) in the question, which today offers only
approve, deny and cancel; a decision-record event kind in the journal; and a write grant per verb,
since `--grant write` covers stop, answer and send together. Size L, after M10 and #534 (C1).

## Recommendations, ranked

1. #519 and #520 first; #516, shipped, already gives agents and Jev the same mask.
2. `marley_system_one` with the `typesafe`, `rules` and `replay` providers, the log, the key
   handling and the settings section, every feature off. M.
3. Shadow on the stop kind, stalls and routing the day the layer lands, and the find tools in
   shadow. S each.
4. The stop kind acts first, display and order only, once recall on the classes that need Chad
   reaches 95% (an abstention counts as needing him) and "done with evidence" is 90% precise.
5. Inbox order and risk chips on #508, with auto mode kept as the approver; then stall flags,
   wired into rustal-harness's foreman escalation when M10 lands.
6. The coach loop once four to six weeks of labels exist; a local model as an experiment only.

## Open questions for Chad

1. The note starts with supervision (stop kind, stalls, routing) and the find tools, and keeps
   approvals in shadow beside Claude Code's reviewer. Is that the "this" you meant? Default: yes.
2. TypeSafe direct, or Cloudflare Workers AI for its stated zero retention? Default: TypeSafe, off.
3. Which projects may send state, and should projects holding other people's data use the
   metadata-only mode or stay out? Default: none until listed.
4. Should any feature ever grant something without you, such as a read-only command inside the
   project? Default: no.
5. Keep Claude Code's auto mode as the approver in Marley's terminals? Default: keep it.
6. Should rustal-harness's foreman include a System One decider, asking through Marley's layer
   when embedded? Default: yes, with an answer-only grant, escalating only at first.
7. Who seals a playbook: you for every version, or the manager for display-only decisions after
   the golden gate? Default: you for anything that acts.
8. Label 300 turn endings and 100 questions yourself, or check 50 that a System 2 judge labels?
9. A local model (Laya, run like Marley's Chromium) for what must not leave the machine? Default:
   not yet.
10. What agreement with your own actions, over how many events, turns a use on?

## Sources

URLs given inline are not repeated, and rustal-harness and Marley files are named where used.
TypeSafe: https://typesafe.ai/blog/introducing-system-one-models-and-jev (the launch post),
https://evals.typesafe.ai/, and https://docs.typesafe.ai/llms.txt, which lists the cookbooks cited.
Independent tests:
https://www.beri.net/article/typesafe-jev-typed-decision-model-calibration-decomposition-shadow-eval,
https://github.com/fstandhartinger/jevbench/blob/main/RESULTS-v1.2.md (JevBench),
https://github.com/ghubnab99/jev-enterprise-decision-fabric/blob/main/docs/evaluations/agent-action-gate-v1.md
(the action-gate study), https://github.com/nikkoxgonzales/jev-certify/blob/main/results/REPORT.md,
https://github.com/jujumilk3/jev-calibration-audit/blob/main/FINDINGS.md (the KoBBQ audit) and
https://openrouter.ai/blog/insights/what-is-jev/ (the 791-call run, as OpenRouter reports it).
