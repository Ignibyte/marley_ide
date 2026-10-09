# shellcheck shell=bash
# #722's visual check: the guide page caught up. `marley: open guide` opens it in a Browser tab;
# the contents filter takes a word, and a click on its first match shows the article.
# - `722-01-home`: "an agent, a project", Home's page (#701, REQ-001);
# - `722-02-agent-control`: "Agent Activity and the kill switch" (#703, REQ-002);
# - `722-03-modes`: "How far agents go" (#704 to #707, #711, REQ-002);
# - `722-04-marley-agent`: "claude-acp marley", the Marley and Rusty agents (#696, REQ-003).
# The filter matches every word anywhere in an article, so each query was checked to hit its
# article first.
compositor sway

# The page's contents filter and its first match, from #599's shots.
FILTER_X=${FILTER_X:-397}
FILTER_Y=${FILTER_Y:-212}
MATCH_X=${MATCH_X:-360}
MATCH_Y=${MATCH_Y:-291}

setup() {
  mkdir -p "$E2E_WORK/project"
  printf '# A project\n' >"$E2E_WORK/project/README.md"
  open_path "$E2E_WORK/project"
}

palette() {
  press "CTRL SHIFT" p
  settle 1
  type_text "$1"
  settle 1
  press "" Return
}

# Filters the contents for `$1`, opens its first match and shoots `$2`.
article() {
  click "$FILTER_X" "$FILTER_Y"
  settle 1
  press "CTRL" a
  type_text "$1"
  settle 2
  click "$MATCH_X" "$MATCH_Y"
  settle 2
  shot "$2"
}

steps() {
  settle 12
  # Trusts the scratch project.
  press "" Return
  settle 3

  echo "== the guide"
  palette "marley: open guide"
  settle 12

  echo "== the new articles"
  article "an agent, a project" 722-01-home
  article "Agent Activity and the kill switch" 722-02-agent-control
  article "How far agents go" 722-03-modes
  article "claude-acp marley" 722-04-marley-agent
}
