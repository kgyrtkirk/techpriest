#!/bin/bash
# bootstrap.sh — SessionStart rite. Mandates the doctrine load before any other
# action, and warns when the guard binary has not been built.

guard="${CLAUDE_PLUGIN_ROOT}/target/release/heresy-guard"

msg="MANDATORY SESSION START: Call ToolSearch immediately before any other action to load deferred tool schemas. Query: select:TaskCreate,LSP,WebSearch. Then load the 'techpriest' skill IN FULL (it carries the sacred directives and hubs to the tooling, shell and code-style rites), plus any other relevant skill. Skipping causes silent failures."

if [ ! -x "$guard" ]; then
  msg+=" WARNING: heresy-guard is not built — Bash heresies go unjudged. Run 'cargo build --release' in ${CLAUDE_PLUGIN_ROOT}."
fi

jq -n --arg m "$msg" \
  '{hookSpecificOutput:{hookEventName:"SessionStart", additionalContext:$m}}'
