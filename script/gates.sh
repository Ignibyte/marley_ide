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
# THE SCOPE RULE (§0). The Marley-owned surface is crates/marley_*. The static
# gates run over those crates plus every crate the change touched (git status,
# so an untracked new crate counts). The heavy gates run over the Marley crates
# in FULL; in DIFF mode coverage covers the touched Marley crates. Upstream Zed
# code is held to Zed's own bar (fmt, ./script/clippy, its tests), not to the
# Marley floors. Mutation testing is not a gate: script/mutation.sh runs it once
# at the end of a sprint (Chad, 2026-09-22).
#
# Gate numbering follows CONSTITUTION §0. Retired numbers are not reused: gate:5,
# mutation, left the gate on 2026-09-22 for script/mutation.sh, and gate:15, the
# macOS visual/AX harness, retired with the fork.
#   STATIC (always): 1 fmt · 2 clippy · 3 tests · 7 audit · 8 deny · 9 shear
#                    10 gitleaks · 11 shellcheck · 12 no-suppress · 13 SAST · 14 docs
#                    16 zed-ledger · 17 manifests · 18 spelling · 19 empty suites
#                    20 semgrep · 21 dylint
#   HEAVY  (FULL/DIFF): 4 coverage · 6 miri — reported BLOCKED, not run, after a
#                    static red
#
# Modes (one is required; none, or an unknown one, is a usage error: exit 2):
#   script/gates.sh --full  FULL — heavy gates over every Marley crate. Receipt.
#   script/gates.sh --diff  DIFF — heavy gates on what the change touched. Receipt.
#   script/gates.sh --fast  FAST — static gates only (no heavy, no receipt).
# A FULL or DIFF run removes the earlier receipt when it starts and writes a new
# one only when the gated files at the end are the ones it started on. Either a
# FULL or a DIFF green satisfies the commit hook.
#
# Knobs (env): MARLEY_UPSTREAM_BASE names the upstream fork-point commit when
# the `upstream` remote is not fetched.
# =============================================================================

set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2
MANIFEST="Cargo.toml"

usage() {
  cat >&2 <<'USAGE'
Usage: script/gates.sh --full|--diff|--fast
  --full  every gate, the heavy ones over every Marley crate; writes the commit receipt
  --diff  every gate, the heavy ones over the touched Marley crates; writes the commit receipt
  --fast  the static gates only; writes no receipt
USAGE
}

# The mode is settled before any gate or cargo command runs.
MODE=""
if [ "$#" -eq 1 ]; then
  case "$1" in
    --full) MODE="full" ;;
    --diff) MODE="diff" ;;
    --fast) MODE="fast" ;;
  esac
fi
[ -n "$MODE" ] || { usage; exit 2; }

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
. ./.claude/hooks/lib-hook-helpers.sh 2>/dev/null \
  || { echo "FATAL: cannot load .claude/hooks/lib-hook-helpers.sh" >&2; exit 2; }

# §0 baked minimums — env may RAISE (ratchet up); a value below the minimum is
# clamped back up, so a green can never be bought by lowering the bar.
RUST_COV_FLOOR=100
RUST_COV_MIN="${RUST_COV_MIN:-$RUST_COV_FLOOR}"
if awk -v c="$RUST_COV_MIN" -v f="$RUST_COV_FLOOR" 'BEGIN{exit !(c+0 < f+0)}'; then echo "note: RUST_COV_MIN below the §0 minimum $RUST_COV_FLOOR — clamped." >&2; RUST_COV_MIN=$RUST_COV_FLOOR; fi

# A FULL or DIFF run revokes the earlier receipt before any gate runs: a tree that
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
clippy_g() {
  cargo clippy --manifest-path "$MANIFEST" "${SCOPE_PKG_ARGS[@]}" --all-targets --all-features -- -D warnings
}

# ── 3. tests — nextest on the scope + doctests on the Marley crates ──────────
tests_g() {
  need cargo-nextest "cargo install cargo-nextest" || return 1
  cargo nextest run --manifest-path "$MANIFEST" "${SCOPE_PKG_ARGS[@]}" --no-tests=warn || return 1
  cargo test --manifest-path "$MANIFEST" "${MARLEY_PKG_ARGS[@]}" --doc || return 1
  # The upstream crates Marley carries build outside the workspace, so their own tests, the
  # Marley hunks' among them, run standalone against the lockfile each copy keeps
  # (vendor/README.md).
  local vendored
  for vendored in vendor/*/Cargo.toml; do
    [ -f "$vendored" ] || continue
    cargo test --locked --manifest-path "$vendored" || return 1
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
  for d in crates/marley_* .claude script/gates.sh docs/marley docs/planning CONSTITUTION.md CHANGELOG.md; do
    [ -e "$d" ] || continue
    gitleaks dir "$d" --no-banner -c .gitleaks.toml || return 1
  done
  return 0
}

# ── 11. shell scripts (the hooks + this gate) ────────────────────────────────
shellcheck_g() { need shellcheck "install shellcheck" || return 1; shellcheck -S info -e SC1091 .claude/hooks/*.sh script/gates.sh script/mutation.sh script/live-shot.sh crates/marley_terminal/shell_integration/marley.bash; }

# The files gates 12/13 scan wholesale: the Marley crates plus any untracked
# Rust file elsewhere under crates/. Tracked edits to Zed crates are judged on
# their ADDED lines only (added_lines) — upstream's own history is upstream's.
scan_files() {
  local d
  for d in crates/marley_*/; do [ -d "${d}src" ] && printf '%s\n' "${d%/}"; done
  git ls-files --others --exclude-standard -- crates 2>/dev/null | grep -E '\.rs$' | grep -vE '^crates/marley_' || true
}
added_lines() { git diff HEAD -U0 -- crates 2>/dev/null | grep -E '^\+[^+]' | sed 's/^+//' || true; }

# ── 12. no inline suppressions (CONSTITUTION §0/§15) ─────────────────────────
no_suppr_g() {
  local targets unjust blanket added_unjust added_blanket masks added_masks
  local mask_re='mutants[[:space:]]*::[[:space:]]*skip'
  targets=$(scan_files)
  # shellcheck disable=SC2086 # the target list is word-split on purpose
  unjust=$(grep -rnE '#!?\[(allow|expect)\(' $targets 2>/dev/null | grep -vE '//[[:space:]]*[^[:space:]]' || true)
  # shellcheck disable=SC2086
  blanket=$(grep -rnE '#!?\[(allow|expect)\((clippy::(all|correctness|suspicious|complexity|perf|style|pedantic|nursery|restriction)|warnings|unused)\b' $targets 2>/dev/null || true)
  added_unjust=$(added_lines | grep -E '#!?\[(allow|expect)\(' | grep -vE '//[[:space:]]*[^[:space:]]' || true)
  added_blanket=$(added_lines | grep -E '#!?\[(allow|expect)\((clippy::(all|correctness|suspicious|complexity|perf|style|pedantic|nursery|restriction)|warnings|unused)\b' || true)
  # A mutation mask would hide code from the end-of-sprint mutation run
  # (script/mutation.sh), so it counts as a suppression. cargo-mutants honours `mutants::skip`
  # inside any `cfg_attr` whatever its condition, so `#[cfg_attr(any(), mutants::skip)]`
  # compiles without the `mutants` crate and hides the item from mutation (#443).
  # shellcheck disable=SC2086
  masks=$(grep -rnE "$mask_re" $targets 2>/dev/null || true)
  added_masks=$(added_lines | grep -E "$mask_re" || true)
  if [ -n "$unjust$blanket$added_unjust$added_blanket$masks$added_masks" ]; then
    echo "unjustified / blanket suppressions (CONSTITUTION §0/§15):"
    [ -n "$unjust$added_unjust" ]   && { echo "— missing a // justification (allow/expect):"; echo "$unjust"; echo "$added_unjust"; }
    [ -n "$blanket$added_blanket" ] && { echo "— blanket group suppression (banned outright):"; echo "$blanket"; echo "$added_blanket"; }
    [ -n "$masks$added_masks" ]     && { echo "— mutation mask (banned outright; mutation runs unmasked):"; echo "$masks"; echo "$added_masks"; }
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
  git diff HEAD -U1 -- crates 2>/dev/null | awk -v re="$UNSAFE_RE" '
    /^(\+\+\+|---) / || /^@@/ { previous = ""; next }
    /^-/ { next }
    {
      line = substr($0, 2)
      if ($0 ~ /^\+/ && line ~ re && line !~ /SAFETY:/ && previous !~ /SAFETY:/) print line
      previous = line
    }
  '
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
      CONSTITUTION.md docs/marley docs/marley_architecture docs/specs docs/zed_architecture docs/decisions docs/tickets .claude 2>/dev/null \
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

# ── 19. empty suites — every Marley test suite but a binary's holds a test ───
# A suite that compiles with no test in it passes gate:3 while testing nothing.
# $1 is the JSON `cargo nextest list --message-format json` prints.
suites_have_tests() {
  local listing="$1" empty
  [ -n "$listing" ] || { echo "cargo nextest list printed nothing"; return 1; }
  jq -e '(."rust-suites" | type == "object" and length > 0)
         and all(."rust-suites"[]; (.testcases | type == "object"))' <<<"$listing" >/dev/null 2>&1 \
    || { echo "cargo nextest list printed no test suites, or a listing this gate cannot read"; return 1; }
  empty=$(jq -r '."rust-suites" | to_entries[]
                 | select((.key | contains("::bin/") | not) and (.value.testcases | length) == 0)
                 | .key' <<<"$listing") \
    || { echo "the nextest listing could not be read for empty suites"; return 1; }
  [ -z "$empty" ] || { echo "test suites with no tests:"; sed 's/^/  /' <<<"$empty"; return 1; }
  return 0
}

empty_suites_g() {
  need cargo-nextest "cargo install cargo-nextest" || return 1
  local listing
  listing=$(cargo nextest list --manifest-path "$MANIFEST" "${MARLEY_PKG_ARGS[@]}" --message-format json) || return 1
  suites_have_tests "$listing"
}

# ── 20. semgrep — the rules in .semgrep.yml that clippy and gitleaks do not cover
# The pin is the engine the rules were proven on. The version check and metrics
# stay off, so the gate makes no network call, and `--no-git-ignore` scans a new
# file before it is tracked (semgrep otherwise reads only what git lists).
SEMGREP_PIN="1.156.0"
semgrep_g() {
  need semgrep "pipx install semgrep==$SEMGREP_PIN" || return 1
  local version
  version=$(SEMGREP_ENABLE_VERSION_CHECK=0 semgrep --version 2>/dev/null | tail -n1)
  [ "$version" = "$SEMGREP_PIN" ] || { echo "semgrep ${version:-(no version)} is not the pinned $SEMGREP_PIN"; return 1; }
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

# ── 4. rust line coverage floor (FULL: the Marley crates; DIFF: the touched ones)
# ACCEPTED-UNTESTABLE (the explicit, documented exclude — §0): the raw PTY shim
# marley_terminal/src/pty_os.rs (four OS calls, exercised end to end by the
# real-PTY integration test) and marley_mcp/src/transport.rs (the loopback
# std::net listener + threads, driven by its loopback tests; the IO-error arms
# a loopback peer cannot provoke keep it here). Every testable line
# of every Marley crate stays in the 100% denominator.
rust_cov() {
  need cargo-llvm-cov "cargo install cargo-llvm-cov" || return 1
  local -a pkgs=()
  if [ "$MODE" = "diff" ]; then
    local p
    for p in $TOUCHED_PKGS; do case " $MARLEY_PKGS " in *" $p "*) pkgs+=( -p "$p" ) ;; esac; done
    if [ "${#pkgs[@]}" -eq 0 ]; then echo "coverage: no Marley crate touched (diff) — skip-clean"; return 0; fi
  else
    pkgs=( "${MARLEY_PKG_ARGS[@]}" )
  fi
  # cargo-llvm-cov reads every workspace test executable in its target directory, but cleans
  # only the packages it runs. One an earlier run left can hold an older build of a crate this
  # run covers, whose line map then reports missed lines (#469). Only the executables go: the
  # run relinks the tests it needs, and every library stays built.
  local cov_deps
  cov_deps="$(cargo metadata --manifest-path "$MANIFEST" --format-version 1 --no-deps 2>/dev/null \
    | jq -r .target_directory)/llvm-cov-target/debug/deps"
  if [ -d "$cov_deps" ]; then
    find "$cov_deps" -maxdepth 1 -type f -perm -u=x ! -name '*.*' -delete
  fi
  cargo llvm-cov nextest --manifest-path "$MANIFEST" "${pkgs[@]}" --no-tests=warn \
    --ignore-filename-regex 'marley_terminal/src/pty_os\.rs|marley_mcp/src/transport\.rs' \
    --fail-under-lines "$RUST_COV_MIN"
}

# ── 6. miri — conditional on unsafe in a Marley crate ────────────────────────
miri_g() {
  local needing="" c
  for c in crates/marley_*/; do
    [ -d "${c}src" ] || continue
    grep -rqE '(^|[^_[:alnum:]])unsafe[^_[:alnum:]]' "${c}src" 2>/dev/null || continue
    grep -qE '^[[:space:]]*miri-exempt[[:space:]]*=' "${c}Cargo.toml" 2>/dev/null && continue
    needing="$needing $(basename "$c")"
  done
  if [ -z "$needing" ]; then
    echo "miri: no Marley crate has non-exempt 'unsafe' — nothing to verify (skip-clean)"
    return 0
  fi
  command -v rustup >/dev/null 2>&1 || { echo "miri: rustup required to verify unsafe crates:$needing"; return 1; }
  rustup toolchain list 2>/dev/null | grep -q '^nightly' \
    || { echo "miri: nightly+miri required for unsafe crates:$needing — rustup toolchain install nightly && rustup +nightly component add miri"; return 1; }
  local pkg
  for pkg in $needing; do
    cargo +nightly miri test --manifest-path "$MANIFEST" -p "$pkg" || return 1
  done
}

# ── STATIC gates (always run) ────────────────────────────────────────────────
run_gate "gate:1  rustfmt" fmt_g
run_gate "gate:2  clippy (-D warnings, scope)" clippy_g
run_gate "gate:3  tests (nextest scope + Marley doctests)" tests_g
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
run_gate "gate:19 empty suites (nextest list)" empty_suites_g
run_gate "gate:20 semgrep (.semgrep.yml)" semgrep_g
run_gate "gate:21 dylint (Zed's lints, Marley crates)" dylint_g

# ── HEAVY gates (FULL + DIFF; FAST skips; a static red blocks them) ──────────
if [ "$MODE" = "fast" ]; then
  RESULTS+=("SKIP  gate:4,6 coverage+miri (--fast) — run the gate with --diff or --full before committing")
elif [ "$FAIL" -gt 0 ]; then
  RESULTS+=("BLOCKED gate:4  rust coverage — a static gate is red")
  RESULTS+=("BLOCKED gate:6  miri — a static gate is red")
else
  run_gate "gate:4  rust coverage (>= ${RUST_COV_MIN}% lines)" rust_cov
  run_gate "gate:6  miri (unsafe crates)" miri_g
fi

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

# Receipt — bind this FULL/DIFF green to the exact worktree it ran on.
# enforce-commit-gate.sh reads it back and blocks `git commit` of Rust source
# unless the fingerprint still matches. FAST never writes one.
if [ "$MODE" != "fast" ]; then
  printf '%s\n' "$GATE_START_HASH" > "$RECEIPT" || { echo "FATAL: the receipt could not be written" >&2; exit 1; }
fi
