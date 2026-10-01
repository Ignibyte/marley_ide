# TICKET-638 — Mutation runs on GitHub's runners

- **Ticket:** LOCAL #638 (chore, the testing tooling)
- **Owner:** claude-opus-5-5, 2026-10-01
- **Pipeline doc:** none yet
- **Source ticket:** Chad, 2026-10-01: "for mutations we may run these soon but I want to set up
  the ability to run these on high cpu usage quick on the cloud because it takes hours to run
  these"; #636's run took 1 h 27 min on the dev box for the nine pure cores
- **Status:** open

## Summary
A mutation pass over Marley's crates takes hours on the dev box, two workers at a time. GitHub
gives a public repository's Actions standard runners (4 vCPUs) at no cost, up to 20 jobs at once,
and the fork `Ignibyte/marley_ide` is public. `script/mutants` runs the pass, or one shard of it
(cargo-mutants' `--shard K/N`, round-robin), and merges shards' outcomes into one report;
`script/mutants cloud` pushes HEAD to a `mutants/<stamp>` branch, which starts a workflow that
runs N shards on N runners at once and merges them, then downloads the report and deletes the
branch. The fork's Actions are off (GitHub's default for a fork, until its owner turns them on
in the Actions tab); `script/mutants cloud-setup` then turns off the 47 workflows the fork
carries from Zed, so only Marley's runs.

## Acceptance
`script/mutants run --shard 0/N` runs a shard locally and leaves its outcomes; `script/mutants
report` over #636's outcomes gives #636's table; with Actions on, `script/mutants cloud` returns
a merged report of the nine pure cores in well under half an hour.
