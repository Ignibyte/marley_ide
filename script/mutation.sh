#!/usr/bin/env bash
# =============================================================================
# script/mutation.sh — mutation testing of the Marley crates, run at the end
# =============================================================================
# Not a pipeline phase and not part of script/gates.sh. Chad took mutation
# testing out of the per-ticket workflow on 2026-09-22 because it was too slow
# to run on every change; it runs here once, at the end of a sprint or before a
# release, when the machine is idle. Fix what it finds at the source: a missed
# mutant is a missing test.
#
# It mutates every Marley crate (crates/marley_*) in copy mode with MUT_JOBS
# workers (default 2). Each copy builds in its own target directory: cargo names
# a workspace crate's artifacts without the checkout's path, so copies sharing
# CARGO_TARGET_DIR overwrite each other's test binaries and a verdict can
# describe another worker's mutant (#443). The score is caught plus timed-out
# mutants over all viable ones; a timeout is a hang, and a hang is detection.
#
# Knobs (env): MUT_PACKAGES="a b" narrows the run to named crates; MUT_JOBS sets
# the worker count; MUT_SCRATCH moves the tree copies (default
# ~/.cache/marley-mutants, never /tmp, which is tmpfs on this box).
# Exit: 0 when no viable mutant survived, 1 otherwise or when the run is not a
# valid measurement.
# =============================================================================

set -uo pipefail
cd "$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)" || exit 2

command -v cargo-mutants >/dev/null 2>&1 || { echo "MISSING TOOL: cargo-mutants — cargo install cargo-mutants" >&2; exit 1; }
command -v jq >/dev/null 2>&1 || { echo "MISSING TOOL: jq" >&2; exit 1; }

packages="${MUT_PACKAGES:-}"
if [ -z "$packages" ]; then
  for d in crates/marley_*/; do [ -f "${d}Cargo.toml" ] && packages="$packages $(basename "$d")"; done
fi
[ -n "$packages" ] || { echo "no Marley crates under crates/marley_*" >&2; exit 1; }
echo "mutation: packages [${packages# }] with ${MUT_JOBS:-2} workers"

rm -rf mutants.out
# --no-config: a `.cargo/mutants.toml` could exclude files or stretch timeouts;
# the Marley crates are mutated with no exclusions.
args=( --no-config --no-times --test-tool=nextest --jobs "${MUT_JOBS:-2}" )
for p in $packages; do args+=( -p "$p" ); done

scratch="${MUT_SCRATCH:-${XDG_CACHE_HOME:-$HOME/.cache}/marley-mutants}"
mkdir -p "$scratch"
# Cleared for the run: the variables that could move the outcomes away from
# where this script reads them, stretch the timeout that counts as caught, or
# make the copies share a target directory again.
env -u CARGO_TARGET_DIR -u CARGO_MUTANTS_OUTPUT -u CARGO_MUTANTS_MINIMUM_TEST_TIMEOUT \
    -u CARGO_BUILD_TARGET_DIR -u CARGO_BUILD_BUILD_DIR TMPDIR="$scratch" \
    cargo mutants "${args[@]}"
rc=$?
# cargo-mutants 27.x exits 0 when every mutant was caught, 2 when some were
# missed and 3 when some timed out; all three are complete runs. 1 is a usage
# error and 4 a failed baseline, neither of which measures anything.
case "$rc" in
  0|2|3) : ;;
  *) echo "cargo mutants did not complete a valid run (exit $rc; 1=usage, 4=baseline failed)"; exit 1 ;;
esac
out=mutants.out
jq empty "$out/outcomes.json" 2>/dev/null \
  || { echo "mutation: $out/outcomes.json is missing or not valid JSON"; exit 1; }

caught=$(jq '[.outcomes[]|select(.summary=="CaughtMutant" or .summary=="Timeout")]|length' "$out/outcomes.json")
missed=$(jq '[.outcomes[]|select(.summary=="MissedMutant")]|length' "$out/outcomes.json")
total=$((caught + missed))
[ "$total" -gt 0 ] || { echo "mutation: no viable mutants produced"; exit 1; }
score=$(awk -v c="$caught" -v t="$total" 'BEGIN{printf "%.1f", 100*c/t}')
echo "mutation: ${caught} caught / ${missed} missed → score ${score}%"
if [ "$missed" -gt 0 ]; then
  echo "missed mutants ($out/missed.txt):"
  cat "$out/missed.txt"
  exit 1
fi
