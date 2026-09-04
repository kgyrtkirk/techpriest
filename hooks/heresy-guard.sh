#!/bin/bash
# heresy-guard.sh — PreToolUse gate delegating judgement to the compiled guard.
#
# The binary is built, not shipped, so an unbuilt plugin must never block work:
# absent binary means silence, and the SessionStart rite reports the absence.

guard="${CLAUDE_PLUGIN_ROOT}/target/release/heresy-guard"
[ -x "$guard" ] && exec "$guard"
exit 0
