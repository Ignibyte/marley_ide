# shellcheck shell=bash
# Fixtures for the browser's e2e scenarios (#488 on), sourced by a scenario at its top level.
#
# - `serve_site <dir>` serves $E2E_WORK/<dir> on 127.0.0.1 at a free port and prints the port.
#   A page served at 127.0.0.1 that embeds one served at `localhost` embeds another site, so
#   Chromium draws it as a cross-site iframe, in a process of its own.
# - `agent <command> ...` runs a stand-in agent that attaches to the run's Chromium the way any
#   CDP client can, through the `DevToolsActivePort` in Marley's profile, and drives the page the
#   Browser tab shows: `navigate <url>`, and `highlight <selector> <seconds>`, which keeps the
#   highlight, drawn for its own session, for that long.
# - `browser_profile` and `browser_unit` name the run's Chromium profile and its user unit;
#   `browser_teardown`, for the scenario's `teardown`, stops the unit and the servers.

browser_profile() {
  printf '%s/browser/profile' "$(realpath "$E2E_PROFILE")"
}

# The unit's name, as `marley_browser::service::unit_name` makes it.
browser_unit() {
  printf 'marley-browser-%s' "$(printf '%s' "$(browser_profile)" | sha256sum | cut -c1-12)"
}

serve_site() {
  local site=$1 port=
  python3 -u -m http.server 0 --bind 127.0.0.1 --directory "$E2E_WORK/$site" \
    >"$E2E_WORK/$site.log" 2>&1 &
  echo "$!" >>"$E2E_WORK/servers"
  for _ in $(seq 50); do
    port=$(grep -oE 'port [0-9]+' "$E2E_WORK/$site.log" | head -1 | cut -d' ' -f2)
    [[ -n $port ]] && break
    sleep 0.1
  done
  [[ -n $port ]] || { echo "serve_site: $site did not start" >&2; return 1; }
  echo "$port"
}

agent() {
  [[ -f $E2E_WORK/agent.mjs ]] || write_agent
  node "$E2E_WORK/agent.mjs" "$(browser_profile)" "$@"
}

browser_teardown() {
  systemctl --user stop "$(browser_unit)" 2>/dev/null || true
  if [[ -f $E2E_WORK/servers ]]; then
    xargs kill <"$E2E_WORK/servers" 2>/dev/null || true
  fi
}

write_agent() {
  cat >"$E2E_WORK/agent.mjs" <<'JS'
// A stand-in agent for the browser's e2e scenarios: a CDP client of the run's Chromium.
import { readFileSync } from 'node:fs';

const [, , profile, command, ...args] = process.argv;
const [port, path] = readFileSync(`${profile}/DevToolsActivePort`, 'utf8').trim().split('\n');
const socket = new WebSocket(`ws://127.0.0.1:${port}${path}`);
let nextId = 0;
const waiting = new Map();
const listeners = [];
socket.onmessage = (message) => {
  const data = JSON.parse(message.data);
  if (data.id !== undefined && waiting.has(data.id)) {
    const { resolve, reject } = waiting.get(data.id);
    waiting.delete(data.id);
    if (data.error) reject(new Error(`${data.error.message}`));
    else resolve(data.result);
  } else if (data.method) {
    for (const listener of listeners) listener(data);
  }
};
const send = (method, params = {}, sessionId) =>
  new Promise((resolve, reject) => {
    const id = ++nextId;
    waiting.set(id, { resolve, reject });
    socket.send(JSON.stringify({ id, method, params, ...(sessionId ? { sessionId } : {}) }));
  });
const once = (method) => new Promise((resolve) => listeners.push((data) => data.method === method && resolve(data)));
const sleep = (ms) => new Promise((resolve) => setTimeout(resolve, ms));
await new Promise((resolve, reject) => {
  socket.onopen = resolve;
  socket.onerror = () => reject(new Error('no DevTools socket'));
});

const { targetInfos } = await send('Target.getTargets');
const page = targetInfos.find((target) => target.type === 'page');
if (!page) throw new Error('the browser has no page');
const { sessionId } = await send('Target.attachToTarget', { targetId: page.targetId, flatten: true });

if (command === 'navigate') {
  await send('Page.enable', {}, sessionId);
  const loaded = once('Page.loadEventFired');
  await send('Page.navigate', { url: args[0] }, sessionId);
  await loaded;
  console.log(`agent: navigated to ${args[0]}`);
} else if (command === 'highlight') {
  await send('DOM.enable', {}, sessionId);
  await send('Overlay.enable', {}, sessionId);
  const { root } = await send('DOM.getDocument', {}, sessionId);
  const { nodeId } = await send('DOM.querySelector', { nodeId: root.nodeId, selector: args[0] }, sessionId);
  await send('Overlay.highlightNode', {
    nodeId,
    highlightConfig: {
      showInfo: true,
      contentColor: { r: 111, g: 168, b: 220, a: 0.66 },
      paddingColor: { r: 147, g: 196, b: 125, a: 0.55 },
      borderColor: { r: 255, g: 229, b: 153, a: 0.66 },
      marginColor: { r: 246, g: 178, b: 107, a: 0.66 },
    },
  }, sessionId);
  console.log(`agent: highlighted ${args[0]}`);
  await sleep(Number(args[1] ?? 5) * 1000);
} else {
  throw new Error(`agent: no command ${command}`);
}
socket.close();
JS
}
