#!/bin/bash
# bootstrap.sh — SessionStart rite. Mandates the doctrine load, and builds the
# guard on first use.
#
# Claude Code has no install-time hook, so a compiled guard has to provision
# itself. SessionStart is the sanctioned moment: it fires before any tool call,
# so the binary exists by the time the PreToolUse gate must judge one. Cargo
# no-ops once built, making every later session pay nothing.
#
# Only the verdict may reach stdout — the harness parses it as JSON — so the
# build is diverted to stderr, where --debug can still read it.

guard_dir="${CLAUDE_PLUGIN_ROOT}/guard"
guard="${guard_dir}/target/release/heresy-guard"

if [ ! -x "$guard" ] && command -v cargo >/dev/null 2>&1; then
  cargo build --release --manifest-path "${guard_dir}/Cargo.toml" 1>&2
fi

msg="MANDATORY SESSION START: Call ToolSearch immediately before any other action to load deferred tool schemas. Query: select:TaskCreate,LSP,WebSearch. Then load the 'techpriest' skill IN FULL (it carries the sacred directives and hubs to the tooling, shell and code-style rites), plus any other relevant skill. Skipping causes silent failures."

if [ ! -x "$guard" ]; then
  if command -v cargo >/dev/null 2>&1; then
    msg+=" WARNING: heresy-guard failed to build — Bash heresies go unjudged. Run 'cargo build --release' in ${guard_dir} and read the error."
  else
    msg+=" WARNING: heresy-guard is not built and cargo is absent — Bash heresies go unjudged. Install the Rust toolchain, then restart the session."
  fi
fi

jq -n --arg m "$msg" \
  '{hookSpecificOutput:{hookEventName:"SessionStart", additionalContext:$m}}'
