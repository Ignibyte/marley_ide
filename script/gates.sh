#!/usr/bin/env bash
# =============================================================================
# script/gates.sh — Marley canonical quality gate (the Zed fork)
# =============================================================================
# The single source of truth for "is this change shippable?" (CONSTITUTION §0).
# Run in the Test phase (/pipeline:test) and read back at commit by
# enforce-commit-gate.sh through the receipt it writes. Strict by charter: no baselines, no
# suppressions, source-fix only. Every gate's verdict is the tool's EXIT CODE,
# never a grep of its output.
#
# THE SCOPE RULE (§0). The Marley-owned surface is crates/marley_*. The gates
# run over those crates plus every crate the change touched (git status, so an
# untracked new crate counts). Upstream Zed code is held to Zed's own bar, fmt
# and clippy, not to the Marley crates' lint levels.
#
# NO TEST RUNS HERE (§7). Chad, 2026-09-23: e2e visualization tests only (#483).
# The tests already in the tree stay, and gate:2 builds them (--all-targets), but
# no gate runs them; a ticket's proof is its e2e scenario under script/e2e/.
#
# Gate numbering follows CONSTITUTION §0. Retired numbers are not reused: gate:3
# (the test suites), gate:4 (coverage), gate:6 (miri) and gate:19 (empty suites)
# retired with the unit tests on 2026-09-23 (#483), and with them the mutation
# run that had been gate:5 until 2026-09-22; gate:15, the macOS visual/AX
# harness, retired with the fork.
#   1 fmt · 2 clippy · 7 audit · 8 deny · 9 shear · 10 gitleaks · 11 shellcheck
#   12 no-suppress · 13 SAST · 14 docs · 16 zed-ledger · 17 manifests
#   18 spelling · 20 semgrep · 21 dylint · 22 spawn sites
#
# Modes (one is required; none, or an unknown one, is a usage error: exit 2):
#   script/gates.sh --diff  DIFF — every gate on the scope. Receipt.
#   script/gates.sh --fast  FAST — every gate on the scope, no receipt.
# --full ran the heavy gates over every Marley crate and is refused since they
# retired. A DIFF run removes the earlier receipt when it starts and writes a new
# one only when the gated files at the end are the ones it started on; the
# commit hook accepts a DIFF green.
#
# Knobs (env): MARLEY_UPSTREAM_BASE names the upstream fork-point commit when
# the `upstream` remote is not fetched.
# =============================================================================

set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2
MANIFEST="Cargo.toml"

usage() {
  cat >&2 <<'USAGE'
Usage: script/gates.sh --diff|--fast
  --diff  every gate on the scope; writes the commit receipt
  --fast  every gate on the scope; writes no receipt
USAGE
}

# The mode is settled before any gate or cargo command runs.
MODE=""
if [ "$#" -eq 1 ]; then
  case "$1" in
    --full) echo "--full ran the heavy gates, which retired with the unit tests (#483); run --diff" >&2; exit 2 ;;
    --diff) MODE="diff" ;;
    --fast) MODE="fast" ;;
  esac
fi
[ -n "$MODE" ] || { usage; exit 2; }

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
. ./.claude/hooks/lib-hook-helpers.sh 2>/dev/null \
  || { echo "FATAL: cannot load .claude/hooks/lib-hook-helpers.sh" >&2; exit 2; }

# A DIFF run revokes the earlier receipt before any gate runs: a tree that
# passed once and fails now must not commit on the older green. The new receipt
# binds the fingerprint taken here.
GATE_START_HASH=""
RECEIPT=""
if [ "$MODE" != "fast" ]; then
  GITDIR=$(git rev-parse --git-dir 2>/dev/null) || { echo "FATAL: cannot resolve the git directory" >&2; exit 2; }
  RECEIPT="$GITDIR/ignibyte-gate-receipt"
  rm -f -- "$RECEIPT" || { echo "FATAL: cannot remove the earlier receipt $RECEIPT" >&2; exit 2; }
  [ ! -e "$RECEIPT" ] || { echo "FATAL: the earlier receipt $RECEIPT is still there" >&2; exit 2; }
  GATE_START_HASH=$(gate_state_hash) || GATE_START_HASH=""
fi

PASS=0; FAIL=0
RESULTS=()

run_gate() {
  local label="$1"; shift
  printf '\n\033[1m▶ %s\033[0m\n    %s\n' "$label" "$*"
  if "$@"; then RESULTS+=("PASS  $label"); PASS=$((PASS + 1))
  else RESULTS+=("FAIL  $label"); FAIL=$((FAIL + 1)); fi
}

need() { command -v "$1" >/dev/null 2>&1 || { echo "MISSING TOOL: $1 — $2" >&2; return 1; }; }

# ── The scope: Marley crates + the crates the change touched ────────────────
# The upstream fork point (`upstream_base`, used by gate:10 and gate:16) is
# defined in lib-hook-helpers.sh, so the commit hook resolves the same commit.

marley_packages() {
  local d
  for d in crates/marley_*/; do [ -f "${d}Cargo.toml" ] && basename "$d"; done
}

# Crates with tracked changes or untracked files, mapped from their directory
# to the package name cargo knows (cargo metadata is the authority; the
# directory name usually matches but is not guaranteed to).
touched_packages() {
  local dirs
  dirs=$(git status --porcelain -- crates 2>/dev/null \
         | sed -E 's/^.. //; s/^"//' \
         | grep -oE '^crates/[^/]+' | sed 's#^crates/##' | sort -u)
  [ -n "$dirs" ] || return 0
  local dirs_json
  dirs_json=$(printf '%s\n' "$dirs" | jq -R . | jq -s .)
  cargo metadata --manifest-path "$MANIFEST" --format-version 1 --no-deps 2>/dev/null \
    | jq -r --arg root "$PWD" --argjson dirs "$dirs_json" '
        .packages[]
        | select(.manifest_path as $m
                 | $dirs | any(. as $d | $m == ($root + "/crates/" + $d + "/Cargo.toml")))
        | .name'
}

MARLEY_PKGS=$(marley_packages | tr '\n' ' ')
TOUCHED_PKGS=$(touched_packages | tr '\n' ' ')
SCOPE_PKGS=$({ echo "$MARLEY_PKGS"; echo "$TOUCHED_PKGS"; } | tr ' ' '\n' | grep -v '^$' | sort -u | tr '\n' ' ')
MARLEY_PKG_ARGS=(); for p in $MARLEY_PKGS; do MARLEY_PKG_ARGS+=( -p "$p" ); done
SCOPE_PKG_ARGS=();  for p in $SCOPE_PKGS;  do SCOPE_PKG_ARGS+=( -p "$p" ); done
echo "scope: marley [$MARLEY_PKGS] touched [$TOUCHED_PKGS]"
if [ "${#SCOPE_PKG_ARGS[@]}" -eq 0 ]; then
  echo "FATAL: no Marley crates found under crates/marley_* and nothing touched" >&2; exit 2
fi

# ── 1. rustfmt (the whole workspace; seconds) ────────────────────────────────
fmt_g() { cargo fmt --manifest-path "$MANIFEST" --all --check; }

# ── 2. clippy on the scope (Zed's lints; ./script/clippy is the CI twin) ─────
# --all-targets builds the tests already in the tree, which no gate runs (§7), so
# they keep compiling.
clippy_g() {
  cargo clippy --manifest-path "$MANIFEST" "${SCOPE_PKG_ARGS[@]}" --all-targets --all-features -- -D warnings || return 1
  # The upstream crates Marley carries build outside the workspace against the lockfile each
  # copy keeps (vendor/README.md), so their tests, the Marley hunks' among them, are built here.
  local vendored
  for vendored in vendor/*/Cargo.toml; do
    [ -f "$vendored" ] || continue
    cargo check --locked --all-targets --manifest-path "$vendored" || return 1
  done
}

# ── 7. security advisories (RUSTSEC) ─────────────────────────────────────────
# Advisories present at the upstream fork point are listed per id in
# .cargo/audit.toml with that commit; a NEW advisory fails here.
audit_g() { need cargo-audit "cargo install cargo-audit" || return 1; cargo audit -f Cargo.lock; }

# ── 8. supply chain (licenses / bans / sources) ──────────────────────────────
deny_g() { need cargo-deny "cargo install cargo-deny" || return 1; cargo deny --manifest-path "$MANIFEST" check licenses bans sources; }

# ── 9. unused dependencies (Zed's tool; the whole workspace) ─────────────────
shear_g() { need cargo-shear "cargo install cargo-shear" || return 1; cargo shear --locked --deny-warnings; }

# ── 10. secrets — commits since the fork point AND the Marley working tree ───
secrets_g() {
  need gitleaks "install gitleaks" || return 1
  local base
  base=$(upstream_base)
  if [ -n "$base" ]; then
    gitleaks git --no-banner -c .gitleaks.toml --log-opts="${base}..HEAD" . || return 1
  else
    echo "secrets: upstream fork point unknown (fetch the upstream remote or set MARLEY_UPSTREAM_BASE) — history scan skipped"
  fi
  local d
  # The e2e runner and its scenarios are Marley's too: a scenario scanned only once committed
  # reaches the public origin before any scan sees it (#584).
  for d in crates/marley_* .claude script/gates.sh script/e2e.sh script/e2e script/regress \
    script/install-marley docs/marley docs/planning CONSTITUTION.md CHANGELOG.md; do
    [ -e "$d" ] || continue
    gitleaks dir "$d" --no-banner -c .gitleaks.toml || return 1
  done
  return 0
}

# ── 11. shell scripts (the hooks, this gate, the e2e runner and its scenarios, the installer) ────────────────────────────────
shellcheck_g() {
  need shellcheck "install shellcheck" || return 1
  need fish "install fish" || return 1
  shellcheck -S info -e SC1091 .claude/hooks/*.sh script/gates.sh script/e2e.sh script/e2e/*.sh script/install-marley script/regress crates/marley_terminal/shell_integration/marley.bash crates/marley_workbench/bin/marley-collect.sh &&
    # fish reads its own script for syntax, which no shell linter can (#466).
    fish --no-execute crates/marley_terminal/shell_integration/marley.fish
}

# The files gates 12/13 scan wholesale: the Marley crates plus any untracked
# Rust file elsewhere under crates/. Tracked edits to Zed crates are judged on
# their ADDED lines only (added_lines) — upstream's own history is upstream's.
scan_files() {
  local d
  for d in crates/marley_*/; do [ -d "${d}src" ] && printf '%s\n' "${d%/}"; done
  git ls-files --others --exclude-standard -- crates 2>/dev/null | grep -E '\.rs$' | grep -vE '^crates/marley_' || true
}
# The upstream commit an in-progress merge brings in, when the merge is of upstream (#695).
upstream_merge_head() {
  local merging
  merging=$(git rev-parse -q --verify MERGE_HEAD 2>/dev/null) || return 1
  git merge-base --is-ancestor "$merging" upstream/main 2>/dev/null || return 1
  printf '%s\n' "$merging"
}
# The lines a diff of crates/ against REF adds.
lines_added_since() { git diff "$1" -U0 -- crates 2>/dev/null | grep -E '^\+[^+]' | sed 's/^+//' || true; }
# While upstream is being merged in, a line is the change's only when it is new against both
# sides: the lines upstream brings are upstream's history (#695).
added_lines() {
  local merging
  if merging=$(upstream_merge_head); then
    LC_ALL=C comm -12 <(lines_added_since HEAD | LC_ALL=C sort -u) \
      <(lines_added_since "$merging" | LC_ALL=C sort -u)
    return 0
  fi
  lines_added_since HEAD
}

# ── 12. no inline suppressions (CONSTITUTION §0/§15) ─────────────────────────
no_suppr_g() {
  local targets unjust blanket added_unjust added_blanket
  targets=$(scan_files)
  # shellcheck disable=SC2086 # the target list is word-split on purpose
  unjust=$(grep -rnE '#!?\[(allow|expect)\(' $targets 2>/dev/null | grep -vE '//[[:space:]]*[^[:space:]]' || true)
  # shellcheck disable=SC2086
  blanket=$(grep -rnE '#!?\[(allow|expect)\((clippy::(all|correctness|suspicious|complexity|perf|style|pedantic|nursery|restriction)|warnings|unused)\b' $targets 2>/dev/null || true)
  added_unjust=$(added_lines | grep -E '#!?\[(allow|expect)\(' | grep -vE '//[[:space:]]*[^[:space:]]' || true)
  added_blanket=$(added_lines | grep -E '#!?\[(allow|expect)\((clippy::(all|correctness|suspicious|complexity|perf|style|pedantic|nursery|restriction)|warnings|unused)\b' || true)
  if [ -n "$unjust$blanket$added_unjust$added_blanket" ]; then
    echo "unjustified / blanket suppressions (CONSTITUTION §0/§15):"
    [ -n "$unjust$added_unjust" ]   && { echo "— missing a // justification (allow/expect):"; echo "$unjust"; echo "$added_unjust"; }
    [ -n "$blanket$added_blanket" ] && { echo "— blanket group suppression (banned outright):"; echo "$blanket"; echo "$added_blanket"; }
    return 1
  fi
  return 0
}

# ── 13. source bans (SAST) — transmute; unsafe without // SAFETY: ───────────
# A `// SAFETY:` comment counts on the `unsafe` line itself or on the line above
# it, where rustfmt and the house style put it.
TRANSMUTE_RE='mem::transmute|(^|[^_[:alnum:]])transmute[[:space:]]*(\(|::<)'
UNSAFE_RE='(^|[^_[:alnum:]])unsafe[^_[:alnum:]]'

# Lines of FILES that say `unsafe` with no `SAFETY:` on them or on the line above.
unjustified_unsafe() {
  awk -v re="$UNSAFE_RE" '
    FNR == 1 { previous = "" }
    $0 ~ re && $0 !~ /SAFETY:/ && previous !~ /SAFETY:/ { print FILENAME ":" FNR ":" $0 }
    { previous = $0 }
  ' "$@"
}

# The same for the lines a tracked edit under crates/ adds, read with one line of
# context so the line above an added `unsafe` is known.
added_unjustified_unsafe() {
  local found merging
  found=$(git diff HEAD -U1 -- crates 2>/dev/null | awk -v re="$UNSAFE_RE" '
    /^(\+\+\+|---) / || /^@@/ { previous = ""; next }
    /^-/ { next }
    {
      line = substr($0, 2)
      if ($0 ~ /^\+/ && line ~ re && line !~ /SAFETY:/ && previous !~ /SAFETY:/) print line
      previous = line
    }
  ')
  [ -n "$found" ] || return 0
  # While upstream is being merged in, upstream's own lines are not the change's (#695).
  if merging=$(upstream_merge_head); then
    LC_ALL=C grep -Fx -f <(lines_added_since "$merging") <<<"$found" || true
    return 0
  fi
  printf '%s\n' "$found"
}

source_bans_g() {
  local targets trans unsafes added_trans added_unsafe files
  targets=$(scan_files)
  # shellcheck disable=SC2086
  trans=$(grep -rnE "$TRANSMUTE_RE" $targets 2>/dev/null || true)
  # shellcheck disable=SC2086
  files=$(grep -rlE "$UNSAFE_RE" $targets 2>/dev/null || true)
  unsafes=""
  # shellcheck disable=SC2086
  [ -z "$files" ] || unsafes=$(unjustified_unsafe $files)
  added_trans=$(added_lines | grep -E "$TRANSMUTE_RE" || true)
  added_unsafe=$(added_unjustified_unsafe)
  if [ -n "$trans$unsafes$added_trans$added_unsafe" ]; then
    echo "banned source primitives (CONSTITUTION §0/§14):"
    [ -n "$trans$added_trans" ]    && { echo "— transmute:"; echo "$trans"; echo "$added_trans"; }
    [ -n "$unsafes$added_unsafe" ] && { echo "— unsafe without a // SAFETY: justification:"; echo "$unsafes"; echo "$added_unsafe"; }
    return 1
  fi
  return 0
}

# ── 14. docs — rustdoc on the Marley crates, warning-free; doc-todos ─────────
# `-D warnings` denies rustdoc's lints, but cargo and rustdoc print warnings no
# lint level reaches, so a printed `warning:` line fails the gate as well (no
# `--quiet`: it silences cargo's own warnings, an unused manifest key among them). The
# todos half bans ACTIONABLE markers (`TODO:` `FIXME(` `XXX!`) in the Marley
# crates' Rust source and in the Marley-AUTHORED committed docs. docs/planning/
# is working scratch, docs/warp_architecture/ transcribes Warp's own markers,
# docs/marley/history/ is the gpui-era changelog verbatim, and docs/src/ is
# Zed's user manual.
docs_g() {
  local out status hits
  out=$(RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$MANIFEST" --no-deps "${MARLEY_PKG_ARGS[@]}" 2>&1)
  status=$?
  [ -z "$out" ] || printf '%s\n' "$out"
  [ "$status" -eq 0 ] || return 1
  if grep -qE '^warning:' <<<"$out"; then echo "cargo doc printed a warning (above)"; return 1; fi
  hits=$({
    grep -rnE '(TODO|FIXME|XXX)[:(!]' --include='*.md' \
      CONSTITUTION.md docs/marley docs/marley_architecture docs/specs docs/zed_architecture docs/orca_architecture docs/t3code_architecture docs/decisions docs/tickets .claude 2>/dev/null \
      | grep -vE '^docs/marley/history/'
    grep -rnE '(TODO|FIXME|XXX)[:(!]' --include='*.rs' crates/marley_* 2>/dev/null
  } || true)
  [ -z "$hits" ] || { echo "actionable TODO/FIXME/XXX markers in the Marley crates or their committed docs:"; echo "$hits"; return 1; }
  return 0
}

# ── 16. the Zed touchpoint ledger (CONSTITUTION §0/§14) ─────────────────────
# Every path outside the Marley-owned set that differs from the upstream fork
# point needs its row in docs/marley/zed-touchpoints.md, and every row's path
# must still differ. The owned set and the check live in lib-hook-helpers.sh,
# shared with enforce-zed-ledger.sh; an unknown fork point fails closed.
zed_ledger_g() { zed_ledger_check "$(upstream_base)"; }

# The Marley crates' directories, one argument each.
marley_dirs() { local p; for p in $MARLEY_PKGS; do printf '%s\n' "crates/$p"; done; }

# ── 17. manifests — cargo-sort and taplo over the Marley manifests ───────────
manifests_g() {
  need cargo-sort "cargo install cargo-sort" || return 1
  need taplo "cargo install taplo-cli" || return 1
  local -a dirs=() manifests=()
  local d
  while IFS= read -r d; do dirs+=( "$d" ); manifests+=( "$d/Cargo.toml" ); done < <(marley_dirs)
  cargo sort --check "${dirs[@]}" || return 1
  RUST_LOG=warn taplo fmt --check "${manifests[@]}" || return 1
}

# ── 18. spelling — typos over the repository, with Zed's config, as Zed's CI runs it
typos_g() { need typos "cargo install typos-cli" || return 1; typos --config .config/typos.toml; }

# ── 20. semgrep — the rules in .semgrep.yml that clippy and gitleaks do not cover
# The pin is the engine the rules were proven on. The version check and metrics
# stay off, so the gate makes no network call, and `--no-git-ignore` scans a new
# file before it is tracked (semgrep otherwise reads only what git lists).
SEMGREP_PIN="1.156.0"
semgrep_pinned() {
  need semgrep "pipx install semgrep==$SEMGREP_PIN" || return 1
  local version
  version=$(SEMGREP_ENABLE_VERSION_CHECK=0 semgrep --version 2>/dev/null | tail -n1)
  [ "$version" = "$SEMGREP_PIN" ] || { echo "semgrep ${version:-(no version)} is not the pinned $SEMGREP_PIN"; return 1; }
}
semgrep_g() {
  semgrep_pinned || return 1
  local -a dirs=()
  local d
  while IFS= read -r d; do dirs+=( "$d" ); done < <(marley_dirs)
  SEMGREP_ENABLE_VERSION_CHECK=0 semgrep --config .semgrep.yml --error --quiet --strict --metrics=off \
    --no-git-ignore "${dirs[@]}"
}

# ── 21. dylint — Zed's own lints (tooling/lints) on the Marley crates ─────────
# The library catches what clippy cannot see in gpui code: an entity updated or notified
# while a view renders, blocking IO where a synchronous context runs, an async block with no
# await, and string and map misuses. Its lints warn, and Zed's crates keep them there. Each
# Marley crate root denies them under the driver's `dylint_lib` cfg, so a Marley hit fails
# the check and the verdict stays cargo's exit code. A lint the library adds warns until it
# joins those lists. The library builds on the nightly its package pins, and the check keeps
# its own target directory under $CARGO_TARGET_DIR/dylint.
dylint_g() {
  need cargo-dylint "cargo install cargo-dylint dylint-link --locked" || return 1
  need dylint-link "cargo install cargo-dylint dylint-link --locked" || return 1
  cargo dylint --all -- --all-targets "${MARLEY_PKG_ARGS[@]}"
}

# ── 22. spawn sites — processes start only in the adapter modules listed ─────
# CONSTITUTION §14 keeps spawns in adapter modules. .config/spawn-sites.yml finds every call that
# starts a process (semgrep parses Rust, so a comment or a string naming one is no spawn);
# .config/spawn-sites.txt lists the files allowed to hold them. The step fails on a spawn outside
# the list, on a listed file that starts nothing, on a count other than the pin (a new spawn
# moves the pin in a reviewed change; one removed brings it down, so the ground is kept), on a
# scan of fewer files than the floor (a broken target list cannot pass empty), and when the rule
# misses a planted form or matches a planted decoy. The pin and the floor live here, apart from
# the list, and the planted file is written from this script on every run.
SPAWN_SITES_PIN=7
SPAWN_SCAN_FLOOR=120
spawn_scan() {
  SEMGREP_ENABLE_VERSION_CHECK=0 semgrep --config .config/spawn-sites.yml --json --quiet --strict \
    --metrics=off --no-git-ignore "$@"
}
spawn_self_test() {
  local dir file json found want
  dir=$(mktemp -d) || return 1
  mkdir -p "$dir/crates/marley_planted/src"
  file=$dir/crates/marley_planted/src/planted.rs
  # No imports: semgrep resolves an imported short form to its full path, which would let a
  # short pattern go missing unseen.
  cat >"$file" <<'RS'
fn planted(options: &Options, size: Size) {
    let a = std::process::Command::new("a"); // planted
    let b = smol::process::Command::new("b"); // planted
    let c = tokio::process::Command::new("c"); // planted
    let d = util::command::new_command("d"); // planted
    let e = util::command::new_std_command("e"); // planted
    let f = util::command::Command::new("f"); // planted
    let g = gpui_util::new_std_command("g"); // planted
    let h = alacritty_terminal::tty::new(options, size, 0); // planted
    let i = Command::new("i"); // planted
    let j = new_command("j"); // planted
    let k = new_std_command("k"); // planted
    let l = tty::new(options, size, 0); // planted
    // std::process::Command::new("m") names a spawn in a comment // decoy
    let n = "util::command::new_command(\"n\")"; // decoy
}
RS
  want=$(grep -n '// planted$' "$file" | cut -d: -f1 | tr '\n' ' ')
  json=$(spawn_scan "$dir") || { echo "the planted self-test: semgrep failed"; rm -rf "$dir"; return 1; }
  rm -rf "$dir"
  found=$(jq -r '.results[].start.line' <<<"$json" | sort -n | tr '\n' ' ')
  [ "$found" = "$want" ] || { echo "the planted self-test: the rule found lines [$found] where the planted spawns are [$want]"; return 1; }
}
spawn_sites_g() {
  semgrep_pinned || return 1
  need jq "pacman -S jq" || return 1
  local -a dirs=()
  local d json found listed site file count scanned verdict=0
  while IFS= read -r d; do dirs+=( "$d" ); done < <(marley_dirs)
  json=$(spawn_scan "${dirs[@]}") || { echo "semgrep failed on the Marley crates"; return 1; }
  found=$(jq -r '.results[] | "\(.path):\(.start.line)"' <<<"$json" | sort)
  listed=$(sed -E 's/#.*$//; s/[[:space:]]+$//' .config/spawn-sites.txt | grep -v '^$' | sort -u)
  while IFS= read -r site; do
    [ -n "$site" ] || continue
    file=${site%:*}
    if [ "$(grep -cxF -- "$file" <<<"$listed")" -eq 0 ]; then
      echo "a process starts outside the listed adapters at $site; move the call into one of: $(tr '\n' ' ' <<<"$listed")"
      verdict=1
    fi
  done <<<"$found"
  while IFS= read -r file; do
    [ -n "$file" ] || continue
    if [ "$(grep -cF -- "$file:" <<<"$found")" -eq 0 ]; then
      echo "$file starts no process, or is gone: delete the line from .config/spawn-sites.txt"
      verdict=1
    fi
  done <<<"$listed"
  count=$(grep -c . <<<"$found")
  if [ "$count" -ne "$SPAWN_SITES_PIN" ]; then
    echo "$count spawn calls where SPAWN_SITES_PIN is $SPAWN_SITES_PIN: move the pin in the same reviewed change"
    verdict=1
  fi
  scanned=$(jq '.paths.scanned | length' <<<"$json")
  if [ "$scanned" -lt "$SPAWN_SCAN_FLOOR" ]; then
    echo "semgrep scanned $scanned files, under the floor of $SPAWN_SCAN_FLOOR"
    verdict=1
  fi
  spawn_self_test || verdict=1
  [ "$verdict" -eq 0 ] && echo "$count spawn calls in $(grep -c . <<<"$listed") adapter modules; $scanned files scanned"
  return "$verdict"
}

# ── The gates ───────────────────────────────────────────────────────────────
run_gate "gate:1  rustfmt" fmt_g
run_gate "gate:2  clippy (-D warnings, scope, every target)" clippy_g
run_gate "gate:7  cargo-audit" audit_g
run_gate "gate:8  cargo-deny (licenses/bans/sources)" deny_g
run_gate "gate:9  cargo-shear (unused deps)" shear_g
run_gate "gate:10 gitleaks (secrets)" secrets_g
run_gate "gate:11 shellcheck" shellcheck_g
run_gate "gate:12 no-suppressions" no_suppr_g
run_gate "gate:13 source-bans (SAST)" source_bans_g
run_gate "gate:14 docs (rustdoc, warning-free + todos)" docs_g
run_gate "gate:16 zed-ledger" zed_ledger_g
run_gate "gate:17 manifests (cargo-sort + taplo)" manifests_g
run_gate "gate:18 spelling (typos)" typos_g
run_gate "gate:20 semgrep (.semgrep.yml)" semgrep_g
run_gate "gate:21 dylint (Zed's lints, Marley crates)" dylint_g
run_gate "gate:22 spawn sites (adapters only)" spawn_sites_g

# The receipt binds the tree the gates ran on, so a change made during the run
# fails the run instead of being receipted untested.
tree_unchanged_g() {
  [ -n "$GATE_START_HASH" ] || { echo "the fingerprint could not be taken when the run started"; return 1; }
  [ "$(gate_state_hash)" = "$GATE_START_HASH" ] \
    || { echo "the gated files changed during the run; run the gate again on the settled tree"; return 1; }
}
[ "$MODE" = "fast" ] || run_gate "receipt: the tree the gates ran on" tree_unchanged_g

# ── Summary ──────────────────────────────────────────────────────────────────
printf '\n\033[1m══ gate summary (%s) ══\033[0m\n' "$MODE"
for r in "${RESULTS[@]}"; do
  case "$r" in
    PASS*) printf '  \033[32m%s\033[0m\n' "$r" ;;
    FAIL*|BLOCKED*) printf '  \033[31m%s\033[0m\n' "$r" ;;
    *)     printf '  %s\n' "$r" ;;
  esac
done
printf '  %d passed, %d failed\n' "$PASS" "$FAIL"

[ "$FAIL" -eq 0 ] || { echo "GATE RED — fix at source (CONSTITUTION §0: no baselines, no suppressions)."; exit 1; }
echo "GATE GREEN [$MODE]"

# Receipt — bind this DIFF green to the exact worktree it ran on.
# enforce-commit-gate.sh reads it back and blocks `git commit` of Rust source
# unless the fingerprint still matches. FAST never writes one.
if [ "$MODE" != "fast" ]; then
  printf '%s\n' "$GATE_START_HASH" > "$RECEIPT" || { echo "FATAL: the receipt could not be written" >&2; exit 1; }
fi
