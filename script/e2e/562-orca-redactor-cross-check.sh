# shellcheck shell=bash
# #562's e2e test: the rules the comparison with Orca's redactor added. The terminal runs a script
# that prints, from fakes put together when the run starts, an `Authorization` credential in each
# form the new rule reads (Basic, Token, Proxy-Authorization, a JSON body, a Digest header), a
# value under each of the six secret names the comparison added, three cookie headers, #516's
# bearer line, and two lines that must stay. A stand-in agent reads the block through Marley's
# MCP server, as Claude Code does, and each line must come back as `expected.txt` has it:
# - each credential as `[redacted: authorization]` after its label and scheme (REQ-001), the
#   JSON body's quote and its next key kept;
# - each new name with `[redacted: secret]` in its value's place (REQ-002);
# - each cookie header's whole value as `[redacted: cookie]`, and the URL after a quoted `-H`
#   header kept (REQ-003);
# - the bearer line with `[redacted: bearer token]`, hidden once (REQ-004);
# - `PATH=/usr/bin:/bin` and the plain line as printed (REQ-005).
# No fake may come back, and the read counts one redaction for each line that changed.
compositor sway
# shellcheck source=script/e2e/browser-fixture.sh
. script/e2e/browser-fixture.sh

setup() {
  local home=$E2E_WORK/home
  mkdir -p "$home"
  echo "PS1='\$ '" >"$home/.bashrc"
  terminal_env HOME "$home"
  git init -q -b redact "$E2E_WORK/repo"
  write_fakes
  open_path "$E2E_WORK/repo"
}

# The script the terminal runs, and what the checks read: `fakes.txt`, each fake as printed;
# `expected.txt`, each line as the agent must read it; `count.txt`, how many lines change.
write_fakes() {
  python3 - "$E2E_WORK" <<'PY'
import base64, pathlib, sys

work = pathlib.Path(sys.argv[1])


def basic(user, fake):
    return base64.b64encode(f"{user}:{fake}".encode()).decode()


label = "Author" + "ization"
# 23 bytes each, so the base64 ends in one `=`, which the rule must not read as a Digest
# parameter; in the JSON body the quote after it must stay.
basic_header = basic("marley", "Fake" * 4)
basic_json = basic("marley", "Fake" * 3 + "Json")
basic_proxy = basic("proxy", "Fake" * 4)
token = "Fake" * 7 + "Tok"
nonce = "Fake" + "Nonce" * 3
response = "Fake" + "Response" * 2
opaque = "Fake" * 2 + "Opaque"
digest = (f'username="marley", realm="e2e", nonce="{nonce}", uri="/api", '
          f'response="{response}", opaque="{opaque}"')
names = ["x-api-" + "key: ", "api-" + "key=", "PRIVATE-" + "KEY=", "ACCESS-" + "KEY=",
         "PRIV" + "KEY=", "BEAR" + "ER="]
values = [f"{'Fake' * 3}Name{index}" for index in range(len(names))]
session = "Fake" * 3 + "Session"
sid = "Fake" * 3 + "Sid"
curl_cookie = "Fake" * 3 + "Curl"
bearer = "Fake" * 6
pairs = [
    (f"{label}: Basic {basic_header}", f"{label}: Basic [redacted: authorization]"),
    (f"{label}: Token {token}", f"{label}: Token [redacted: authorization]"),
    (f"Proxy-{label}: Basic {basic_proxy}", f"Proxy-{label}: Basic [redacted: authorization]"),
    (f'{{"{label}": "Basic {basic_json}", "Accept": "application/json"}}',
     f'{{"{label}": "Basic [redacted: authorization]", "Accept": "application/json"}}'),
    (f"{label}: Digest {digest}", f"{label}: Digest [redacted: authorization]"),
    *[(name + value, name + "[redacted: secret]") for name, value in zip(names, values)],
    (f"Cookie: session={session}; theme=dark", "Cookie: [redacted: cookie]"),
    (f"Set-Cookie: sid={sid}; Path=/; HttpOnly", "Set-Cookie: [redacted: cookie]"),
    (f'curl -H "Cookie: session={curl_cookie}" https://api.example.test/v2',
     'curl -H "Cookie: [redacted: cookie]" https://api.example.test/v2'),
    (f'curl -H "{label}: Bear' + f'er {bearer}" https://api.example.test/v1',
     f'curl -H "{label}: Bearer [redacted: bearer token]" https://api.example.test/v1'),
    ("PATH=/usr/bin:/bin", "PATH=/usr/bin:/bin"),
    ("a plain line stays: 5 files built", "a plain line stays: 5 files built"),
]
script = "cat <<'EOF'\n" + "\n".join(printed for printed, _ in pairs) + "\nEOF\n"
(work / "repo" / "secrets.sh").write_text(script)
fakes = [basic_header, basic_json, basic_proxy, token, nonce, response, opaque, *values, session,
         sid, curl_cookie, bearer]
(work / "fakes.txt").write_text("\n".join(fakes) + "\n")
(work / "expected.txt").write_text("\n".join(read for _, read in pairs) + "\n")
(work / "count.txt").write_text(str(sum(printed != read for printed, read in pairs)))
PY
}

# Whether each line of `expected.txt` is a whole line of the read.
every_line() {
  local line failed=0
  while IFS= read -r line; do
    grep -qxF -- "$line" "$E2E_WORK/read.txt" || {
      echo "not in the read: $line" >&2
      failed=1
    }
  done <"$E2E_WORK/expected.txt"
  return "$failed"
}

no_fake() {
  ! grep -qFf "$E2E_WORK/fakes.txt" "$E2E_WORK/read.txt"
}

steps() {
  settle 12
  # Trusts the scratch repository.
  press "" Return
  settle 2
  type_text "sh secrets.sh"
  press "" Return
  settle 3
  shot 562-00-printed
  mcp_agent terminal-read secrets.sh >"$E2E_WORK/read.txt"
  cat "$E2E_WORK/read.txt"
  expect "each line reads as expected" every_line
  expect "no fake comes back" no_fake
  expect "the bearer line is hidden once" \
    test "$(grep -cF "[redacted: bearer token]" "$E2E_WORK/read.txt")" -eq 1
  expect "one redaction for each line that changed" \
    holds "$E2E_WORK/read.txt" "  redacted: $(cat "$E2E_WORK/count.txt") in the read, 0 in the list"
}
