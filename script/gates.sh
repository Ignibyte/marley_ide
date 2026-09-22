#!/usr/bin/env bash
# =============================================================================
# script/gates.sh — Marley canonical quality gate (the Zed fork)
# =============================================================================
# The single source of truth for "is this change shippable?" (CONSTITUTION §0).
# Invoked by /commit (the delivery gate) and read back by enforce-commit-gate.sh
# through the receipt it writes. Strict by charter: no baselines, no
# suppressions, source-fix only. Every gate's verdict is the tool's EXIT CODE,
# never a grep of its output.
#
# THE SCOPE RULE (§0). The Marley-owned surface is crates/marley_*. The static
# gates run over those crates plus every crate the change touched (git status,
# so an untracked new crate counts). The heavy gates run over the Marley crates
# in FULL; in DIFF mode mutation covers the touched lines of any crate, in place
# with one job (the target directory is shared with every project on this box),
# and coverage covers the touched Marley crates. Upstream Zed code is held to
# Zed's own bar (fmt, ./script/clippy, its tests), not to the Marley floors.
#
# Gate numbering follows CONSTITUTION §0 (1-14 and 16; gate:15, the macOS
# visual/AX harness, retired with the fork):
#   STATIC (always): 1 fmt · 2 clippy · 3 tests · 7 audit · 8 deny · 9 shear
#                    10 gitleaks · 11 shellcheck · 12 no-suppress · 13 SAST · 14 docs
#                    16 zed-ledger
#   HEAVY  (FULL/DIFF): 4 coverage · 5 mutation · 6 miri
#
# Modes:
#   script/gates.sh         FULL — heavy gates over every Marley crate. Receipt.
#   script/gates.sh --diff  DIFF — heavy gates on what the change touched. Receipt.
#   script/gates.sh --fast  FAST — static gates only (no heavy, no receipt).
#   GATE_FAST=1 also selects FAST. Either FULL or DIFF green satisfies /commit.
#
# Knobs (env): MUT_PACKAGES="a b" scopes a FULL mutation run to named Marley
# crates (a smoke or a killer-test check; writes NO receipt); MUT_JOBS overrides the FULL job
# count (default 2; DIFF is always 1, in place); MARLEY_UPSTREAM_BASE names the
# upstream fork-point commit when the `upstream` remote is not fetched.
# =============================================================================

set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2
MANIFEST="Cargo.toml"

# shellcheck source=.claude/hooks/lib-hook-helpers.sh
. ./.claude/hooks/lib-hook-helpers.sh 2>/dev/null \
  || { echo "FATAL: cannot load .claude/hooks/lib-hook-helpers.sh" >&2; exit 2; }

# §0 baked minimums — env may RAISE (ratchet up); a value below the minimum is
# clamped back up, so a green can never be bought by lowering the bar.
RUST_COV_FLOOR=100; MUT_MSI_FLOOR=100
RUST_COV_MIN="${RUST_COV_MIN:-$RUST_COV_FLOOR}"
MUT_MSI_MIN="${MUT_MSI_MIN:-$MUT_MSI_FLOOR}"
if awk -v c="$RUST_COV_MIN" -v f="$RUST_COV_FLOOR" 'BEGIN{exit !(c+0 < f+0)}'; then echo "note: RUST_COV_MIN below the §0 minimum $RUST_COV_FLOOR — clamped." >&2; RUST_COV_MIN=$RUST_COV_FLOOR; fi
if awk -v c="$MUT_MSI_MIN"  -v f="$MUT_MSI_FLOOR"  'BEGIN{exit !(c+0 < f+0)}'; then echo "note: MUT_MSI_MIN below the §0 minimum $MUT_MSI_FLOOR — clamped." >&2; MUT_MSI_MIN=$MUT_MSI_FLOOR; fi

MODE="full"
case "${1:-}" in --fast|fast) MODE="fast" ;; --diff|diff) MODE="diff" ;; esac
[ "${GATE_FAST:-0}" = "1" ] && MODE="fast"

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
shellcheck_g() { need shellcheck "install shellcheck" || return 1; shellcheck -S info -e SC1091 .claude/hooks/*.sh script/gates.sh; }

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
  # A mutation mask is a suppression of gate 5. cargo-mutants honours `mutants::skip`
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

# ── 13. source bans (SAST) — mem::transmute; unsafe without // SAFETY: ───────
source_bans_g() {
  local targets trans unsafes added_trans added_unsafe
  targets=$(scan_files)
  # shellcheck disable=SC2086
  trans=$(grep -rnE 'mem::transmute' $targets 2>/dev/null || true)
  # shellcheck disable=SC2086
  unsafes=$(grep -rnE '(^|[^_[:alnum:]])unsafe[^_[:alnum:]]' $targets 2>/dev/null | grep -v 'SAFETY:' || true)
  added_trans=$(added_lines | grep -E 'mem::transmute' || true)
  added_unsafe=$(added_lines | grep -E '(^|[^_[:alnum:]])unsafe[^_[:alnum:]]' | grep -v 'SAFETY:' || true)
  if [ -n "$trans$unsafes$added_trans$added_unsafe" ]; then
    echo "banned source primitives (CONSTITUTION §0/§14):"
    [ -n "$trans$added_trans" ]    && { echo "— mem::transmute:"; echo "$trans"; echo "$added_trans"; }
    [ -n "$unsafes$added_unsafe" ] && { echo "— unsafe without a // SAFETY: justification:"; echo "$unsafes"; echo "$added_unsafe"; }
    return 1
  fi
  return 0
}

# ── 14. docs — rustdoc -D warnings on the Marley crates + doc-todos ──────────
# The doc-todos half bans ACTIONABLE markers (`TODO:` `FIXME(` `XXX!`) in the
# Marley-AUTHORED committed docs. docs/planning/ is working scratch,
# docs/warp_architecture/ transcribes Warp's own markers, docs/marley/history/
# is the gpui-era changelog verbatim, and docs/src/ is Zed's user manual.
docs_g() {
  RUSTDOCFLAGS="-D warnings" cargo doc --manifest-path "$MANIFEST" --no-deps --quiet "${MARLEY_PKG_ARGS[@]}" || return 1
  local hits
  hits=$(grep -rnE '(TODO|FIXME|XXX)[:(!]' --include='*.md' \
           CONSTITUTION.md docs/marley docs/marley_architecture docs/specs docs/zed_architecture docs/decisions docs/tickets .claude 2>/dev/null \
         | grep -vE '^docs/marley/history/' || true)
  [ -z "$hits" ] || { echo "actionable TODO/FIXME/XXX markers in committed Marley docs:"; echo "$hits"; return 1; }
  return 0
}

# ── 16. the Zed touchpoint ledger (CONSTITUTION §0/§14) ─────────────────────
# Every path outside the Marley-owned set that differs from the upstream fork
# point needs its row in docs/marley/zed-touchpoints.md, and every row's path
# must still differ. The owned set and the check live in lib-hook-helpers.sh,
# shared with enforce-zed-ledger.sh; an unknown fork point fails closed.
zed_ledger_g() { zed_ledger_check "$(upstream_base)"; }

# ── 4. rust line coverage floor (FULL: the Marley crates; DIFF: the touched ones)
# ACCEPTED-UNTESTABLE (the explicit, documented exclude — §0): the raw PTY shim
# marley_terminal/src/pty_os.rs (four OS calls, exercised end to end by the
# real-PTY integration test) and marley_mcp/src/transport.rs (the loopback
# std::net listener + threads, verified on the live wire). Every testable line
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
  cargo llvm-cov nextest --manifest-path "$MANIFEST" "${pkgs[@]}" --no-tests=warn \
    --ignore-filename-regex 'marley_terminal/src/pty_os\.rs|marley_mcp/src/transport\.rs' \
    --fail-under-lines "$RUST_COV_MIN"
}

# ── 5. mutation testing (MSI floor) ──────────────────────────────────────────
# The single source of counting truth over an outcomes dir: the timeout audit
# (a Timeout whose log shows a failing test was caught by assert, not a hang)
# and the MSI threshold. Timeout counts as CAUGHT (a hang IS detection).
mutation_verdict() {
  local out="$1"
  jq empty "$out/outcomes.json" 2>/dev/null \
    || { echo "mutation: $out/outcomes.json is not valid JSON (truncated/interrupted run?)"; return 1; }
  local timeouts=0 mislabeled=0 lp
  while IFS= read -r lp; do
    [ -z "$lp" ] && continue
    timeouts=$((timeouts + 1))
    if [ -f "$out/$lp" ] && grep -qE 'FAILED|panicked|assertion' "$out/$lp"; then
      mislabeled=$((mislabeled + 1))
    fi
  done < <(jq -r '.outcomes[]|select(.summary=="Timeout")|.log_path' "$out/outcomes.json")
  if [ "$timeouts" -gt 0 ]; then
    echo "mutation: ${timeouts} timeout(s); ${mislabeled} mislabeled (log shows failing tests — caught-by-assert, perf not detection)"
  fi
  local caught missed total msi
  caught=$(jq '[.outcomes[]|select(.summary=="CaughtMutant" or .summary=="Timeout")]|length' "$out/outcomes.json")
  missed=$(jq '[.outcomes[]|select(.summary=="MissedMutant")]|length' "$out/outcomes.json")
  total=$((caught + missed))
  if [ "$total" -eq 0 ]; then
    [ "$MODE" = "diff" ] && { echo "mutation: 0 viable mutants in the diff (tests/comments/excluded only) — pass"; return 0; }
    echo "no viable mutants produced (a crate with no testable code fails closed — build the crate + tests first)"; return 1
  fi
  msi=$(awk -v c="$caught" -v t="$total" 'BEGIN{printf "%.1f", 100*c/t}')
  echo "mutation: ${caught} caught / ${missed} missed → MSI ${msi}% (floor ${MUT_MSI_MIN}%)"
  awk -v m="$msi" -v min="$MUT_MSI_MIN" 'BEGIN{exit !(m+0 >= min+0)}' \
    || { echo "MSI ${msi}% < floor ${MUT_MSI_MIN}% — kill more mutants (write tests)"; return 1; }
}

# FULL mutates the Marley crates (copy mode, MUT_JOBS workers, default 2, each
# copy building in its own target directory so no worker tests another's
# binary). DIFF mutates only the lines this change touched, in
# place with one job, so a touched Zed crate rebuilds incrementally instead of
# cold in a copy. An interrupted in-place run can leave a mutant in the tree:
# `git diff` shows it and the receipt refuses it; restore the file.
mutation_g() {
  need cargo-mutants "cargo install cargo-mutants" || return 1
  need jq "install jq" || return 1
  rm -rf mutants.out
  # --no-config: a `.cargo/mutants.toml` could exclude files or stretch timeouts, and the
  # fork's mutation gate has no exclusions (CONSTITUTION §0).
  local -a margs=( --no-config --no-times --test-tool=nextest )
  local rc p
  if [ "$MODE" = "diff" ]; then
    # Explicit a/ b/ prefixes: cargo-mutants strips `b/` and nothing else, and
    # git 2.55 labels a --no-index diff `1/` `2/` (a user's diff.noprefix would
    # drop them entirely). Untracked files enter as whole-file diffs.
    git diff HEAD --src-prefix=a/ --dst-prefix=b/ -- crates > mutants.diff 2>/dev/null || true
    git ls-files --others --exclude-standard -- crates 2>/dev/null | grep -E '\.rs$' | while IFS= read -r p; do
      git diff --no-index --src-prefix=a/ --dst-prefix=b/ /dev/null "$p" >> mutants.diff 2>/dev/null || true
    done
    if [ ! -s mutants.diff ]; then
      rm -f mutants.diff
      echo "mutation: no changed crate lines (diff) — nothing to mutate, pass"
      return 0
    fi
    # Name every touched package: the root manifest inherits Zed's
    # `default-members = ["crates/zed"]`, and cargo-mutants mutates only the
    # default members unless told otherwise, so without `-p` the diff filter finds
    # nothing and the gate would pass without testing a mutant (#443).
    if [ -z "$TOUCHED_PKGS" ]; then
      rm -f mutants.diff
      echo "mutation: the diff has crate lines but no touched package resolved — failing closed"
      return 1
    fi
    for p in $TOUCHED_PKGS; do margs+=( -p "$p" ); done
    # In place is one job by construction; cargo-mutants 27 refuses `--jobs`
    # beside `--in-place` as a usage error (exit 1).
    margs=( --in-diff mutants.diff --in-place "${margs[@]}" )
  else
    local mut_pkgs="${MUT_PACKAGES:-$MARLEY_PKGS}"
    for p in $mut_pkgs; do margs+=( -p "$p" ); done
    margs+=( --jobs "${MUT_JOBS:-2}" )
  fi
  # Copy-mode runs (FULL) write their tree copies under a disk-backed scratch,
  # never /tmp (tmpfs on this box); MUT_SCRATCH overrides.
  local scratch="${MUT_SCRATCH:-${XDG_CACHE_HOME:-$HOME/.cache}/marley-mutants}"
  mkdir -p "$scratch"
  # Each FULL copy builds in its own target directory. Cargo names a workspace
  # crate's artifacts without the checkout's path, so copies sharing
  # CARGO_TARGET_DIR overwrite each other's test binaries between one mutant's
  # build and its test run, and a verdict can describe the other worker's mutant
  # (#443). DIFF is in place with one job, so it keeps the shared, warm target.
  # The variables that could move the outcomes away from where this gate reads
  # them, stretch the timeout that counts as caught, or share a target again, are
  # cleared for every run.
  local -a mut_env=( -u CARGO_MUTANTS_OUTPUT -u CARGO_MUTANTS_MINIMUM_TEST_TIMEOUT
                     -u CARGO_BUILD_TARGET_DIR -u CARGO_BUILD_BUILD_DIR TMPDIR="$scratch" )
  [ "$MODE" = "diff" ] || mut_env=( -u CARGO_TARGET_DIR "${mut_env[@]}" )
  ( env "${mut_env[@]}" cargo mutants "${margs[@]}" >/dev/null 2>&1 ); rc=$?
  rm -f mutants.diff
  # cargo-mutants 27.x exit: 0 all-caught · 2 missed found · 3 timeout found —
  # all COMPLETED runs we threshold below. 1 = usage, 4 = baseline build/test
  # failed = NOT a valid measurement (fail closed).
  case "$rc" in
    0|2|3) : ;;
    *) echo "cargo mutants did not complete a valid run (exit $rc; 1=usage, 4=baseline failed)"; return 1 ;;
  esac
  if [ ! -f mutants.out/outcomes.json ]; then
    [ "$MODE" = "diff" ] && { echo "mutation: no mutable lines in the diff — pass"; return 0; }
    echo "no mutants.out/outcomes.json produced"; return 1
  fi
  mutation_verdict mutants.out
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
run_gate "gate:14 docs (rustdoc -D warnings + doc-todos)" docs_g
run_gate "gate:16 zed-ledger" zed_ledger_g

# ── HEAVY gates (FULL + DIFF; FAST skips) ────────────────────────────────────
if [ "$MODE" = "fast" ]; then
  RESULTS+=("SKIP  gate:4,5,6 coverage+mutation+miri (--fast) — run the FULL or --diff gate before /commit")
else
  run_gate "gate:4  rust coverage (>= ${RUST_COV_MIN}% lines)" rust_cov
  run_gate "gate:5  mutation (MSI >= ${MUT_MSI_MIN}%)" mutation_g
  run_gate "gate:6  miri (unsafe crates)" miri_g
fi

# ── Summary ──────────────────────────────────────────────────────────────────
printf '\n\033[1m══ gate summary (%s) ══\033[0m\n' "$MODE"
for r in "${RESULTS[@]}"; do
  case "$r" in
    PASS*) printf '  \033[32m%s\033[0m\n' "$r" ;;
    FAIL*) printf '  \033[31m%s\033[0m\n' "$r" ;;
    *)     printf '  %s\n' "$r" ;;
  esac
done
printf '  %d passed, %d failed\n' "$PASS" "$FAIL"

[ "$FAIL" -eq 0 ] || { echo "GATE RED — fix at source (CONSTITUTION §0: no baselines, no suppressions)."; exit 1; }
echo "GATE GREEN [$MODE]"

# Receipt — bind this FULL/DIFF green to the exact worktree it ran on.
# enforce-commit-gate.sh reads it back and blocks `git commit` of Rust source
# unless the fingerprint still matches. FAST never writes one.
# A MUT_PACKAGES-scoped run is a smoke, not a measurement of the tree: it
# prints its verdict but writes no receipt (§15: a scoped green must never
# satisfy /commit).
if [ "$MODE" != "fast" ] && [ -z "${MUT_PACKAGES:-}" ]; then
  GITDIR=$(git rev-parse --git-dir 2>/dev/null || true)
  if [ -n "$GITDIR" ]; then gate_state_hash > "$GITDIR/ignibyte-gate-receipt" || echo "note: receipt not written (fingerprint failed)" >&2; fi
elif [ -n "${MUT_PACKAGES:-}" ]; then
  echo "note: MUT_PACKAGES scoped this run to [$MUT_PACKAGES]; no receipt written"
fi
