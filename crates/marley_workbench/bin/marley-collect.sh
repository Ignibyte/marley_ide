#!/bin/sh
# Marley's host collector (#610, docs/marley/fleet-contract.md). It prints one marley.host/v1
# document about the machine it runs on and the agent processes there, then exits. Marley runs it
# over SSH, or with sh on its own machine; nothing is installed. It reads /proc and df, needs no
# root and opens no port.
#
# MARLEY_COLLECT_NAMES adds process names, separated by spaces, to claude and codex. A process's
# MARLEY_FLEET_SESSION, when its environment can be read, is the session a store's agent carries.
#
# Rates (the processor, the network, each process's processor) are measured against the counters
# of the previous run, kept in a file under XDG_RUNTIME_DIR, else a folder of its own in /tmp;
# with no recent previous run, it samples twice, a second apart.

LC_ALL=C
export LC_ALL

me=$(id -u)

# Whether a path is this user's own.
mine() {
  [ -n "$(find "$1" -prune -user "$me" 2>/dev/null)" ]
}

state_dir=${XDG_RUNTIME_DIR:-}
if [ -z "$state_dir" ] || [ ! -d "$state_dir" ] || ! mine "$state_dir"; then
  state_dir=/tmp/marley-collect-$me
  mkdir -m 700 "$state_dir" 2>/dev/null
  if [ -h "$state_dir" ] || [ ! -d "$state_dir" ] || ! mine "$state_dir"; then
    state_dir=
  fi
fi
state=
if [ -n "$state_dir" ]; then
  state=$state_dir/marley-collect.state
fi

# The processor's counters since boot: all ticks, and the idle ones (idle and iowait).
cpu_counters() {
  awk '/^cpu / { total = 0; for (i = 2; i <= NF; i++) total += $i; printf "%.0f %.0f\n", total, $5 + $6; exit }' /proc/stat
}

# Bytes in and out since boot, over every interface but the loopback.
net_counters() {
  awk -F'[: ]+' 'NR > 2 && $2 != "lo" { rx += $3; tx += $11 } END { printf "%.0f %.0f\n", rx, tx }' /proc/net/dev
}

now=$(date +%s)
# shellcheck disable=SC2046 # the counters are numbers, split on purpose
set -- $(cpu_counters) $(net_counters)
cpu_total=$1 cpu_idle=$2 net_rx=$3 net_tx=$4

previous=
if [ -n "$state" ] && [ -f "$state" ]; then
  previous=$(head -n 1 "$state")
fi
# shellcheck disable=SC2086 # the previous run's counters, split on purpose
set -- $previous
first=no
if [ $# -ne 5 ] || [ $((now - $1)) -lt 1 ] || [ $((now - $1)) -gt 60 ]; then
  first=yes
  sleep 1
  prev_time=$now prev_total=$cpu_total prev_idle=$cpu_idle prev_rx=$net_rx prev_tx=$net_tx
  now=$(date +%s)
  # shellcheck disable=SC2046
  set -- $(cpu_counters) $(net_counters)
  cpu_total=$1 cpu_idle=$2 net_rx=$3 net_tx=$4
else
  prev_time=$1 prev_total=$2 prev_idle=$3 prev_rx=$4 prev_tx=$5
fi

elapsed=$((now - prev_time))
[ "$elapsed" -ge 1 ] || elapsed=1
total_delta=$((cpu_total - prev_total))
idle_delta=$((cpu_idle - prev_idle))
busy_tenths=0
if [ "$total_delta" -gt 0 ]; then
  busy_tenths=$((1000 * (total_delta - idle_delta) / total_delta))
fi
[ "$busy_tenths" -ge 0 ] || busy_tenths=0
rx_bps=$(((net_rx - prev_rx) / elapsed))
tx_bps=$(((net_tx - prev_tx) / elapsed))
[ "$rx_bps" -ge 0 ] || rx_bps=0
[ "$tx_bps" -ge 0 ] || tx_bps=0

# A JSON string's inside: control characters dropped, backslashes and quotes escaped.
json_text() {
  printf '%s' "$1" | tr -d '\000-\037' | sed 's/\\/\\\\/g; s/"/\\"/g'
}

name=$(uname -n)
os=$(uname -sr)
cores=$(getconf _NPROCESSORS_ONLN 2>/dev/null || grep -c '^processor' /proc/cpuinfo)
uptime_s=$(awk '{ printf "%d\n", $1 }' /proc/uptime)
load=$(awk '{ printf "%s, %s, %s\n", $1, $2, $3 }' /proc/loadavg)
memory=$(awk '/^MemTotal:/ { total = $2 } /^MemAvailable:/ { free = $2 } END { printf "{\"used_bytes\": %.0f, \"total_bytes\": %.0f}\n", (total - free) * 1024, total * 1024 }' /proc/meminfo)
disk=$(df -P -k / 2>/dev/null | awk 'NR == 2 { printf "{\"mount\": \"/\", \"used_bytes\": %.0f, \"total_bytes\": %.0f}\n", $3 * 1024, $2 * 1024 }')

ticks_per_s=$(getconf CLK_TCK 2>/dev/null || echo 100)
boot=$(awk '/^btime/ { print $2 }' /proc/stat)
names=" claude codex ${MARLEY_COLLECT_NAMES:-} "

next_state=
if [ -n "$state" ]; then
  next_state=$state.$$
  printf '%s %s %s %s %s\n' "$now" "$cpu_total" "$cpu_idle" "$net_rx" "$net_tx" >"$next_state"
fi

# Each agent process as a JSON object, one a line, with a comma after all but the last.
agents=$(grep '' /proc/[0-9]*/comm 2>/dev/null | while IFS= read -r line; do
  comm=${line#*:}
  case $names in
    *" $comm "*) ;;
    *) continue ;;
  esac
  pid=${line%%:*}
  pid=${pid#/proc/}
  pid=${pid%/comm}
  stat=$(sed 's/.*) //' "/proc/$pid/stat" 2>/dev/null) || continue
  # shellcheck disable=SC2086 # the fields after the command's name, split on purpose
  set -- $stat
  [ $# -ge 20 ] || continue
  ticks=$((${12} + ${13}))
  started_s=$((boot + ${20} / ticks_per_s))
  if [ -n "$next_state" ]; then
    printf 'pid %s %s\n' "$pid" "$ticks" >>"$next_state"
  fi
  before=
  if [ "$first" = no ] && [ -n "$state" ]; then
    before=$(awk -v pid="$pid" '$1 == "pid" && $2 == pid { print $3; exit }' "$state")
  fi
  if [ -n "$before" ]; then
    cpu_tenths=$((1000 * (ticks - before) / ticks_per_s / elapsed))
  else
    lifetime=$((now - started_s))
    [ "$lifetime" -ge 1 ] || lifetime=1
    cpu_tenths=$((1000 * ticks / ticks_per_s / lifetime))
  fi
  [ "$cpu_tenths" -ge 0 ] || cpu_tenths=0
  rss_kb=$(awk '/^VmRSS:/ { print $2 }' "/proc/$pid/status" 2>/dev/null)
  cwd=$(readlink "/proc/$pid/cwd" 2>/dev/null)
  session=$(tr '\000' '\n' 2>/dev/null <"/proc/$pid/environ" | sed -n 's/^MARLEY_FLEET_SESSION=//p' | head -n 1)
  runtime=$comm
  [ "$comm" = claude ] && runtime=claude-code
  printf '{"pid": %s, "runtime": "%s", "started_ms": %s000, "cpu_percent": %s.%s' \
    "$pid" "$(json_text "$runtime")" "$started_s" "$((cpu_tenths / 10))" "$((cpu_tenths % 10))"
  [ -n "$rss_kb" ] && printf ', "rss_bytes": %s' "$((rss_kb * 1024))"
  [ -n "$cwd" ] && printf ', "cwd": "%s"' "$(json_text "$cwd")"
  [ -n "$session" ] && printf ', "session": "%s"' "$(json_text "$session")"
  printf '}\n'
done | sed '$!s/$/,/')

if [ -n "$next_state" ]; then
  mv -f "$next_state" "$state"
fi

printf '{"contract": "marley.host/v1", "host": {"id": "%s", "name": "%s", "os": "%s", "cores": %s, "uptime_s": %s}, ' \
  "$(json_text "$name")" "$(json_text "$name")" "$(json_text "$os")" "${cores:-0}" "${uptime_s:-0}"
printf '"sampled_ms": %s000, "cpu": {"percent": %s.%s}, "load": [%s], ' \
  "$now" "$((busy_tenths / 10))" "$((busy_tenths % 10))" "$load"
printf '"memory": %s, "disks": [%s], "network": {"rx_bps": %s, "tx_bps": %s}, "agents": [%s]}\n' \
  "$memory" "$disk" "$rx_bps" "$tx_bps" "$agents"
