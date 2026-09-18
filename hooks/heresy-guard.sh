#!/bin/bash
# heresy-guard.sh — PreToolUse gate delegating judgement to the compiled guard.
#
# Never builds: this runs before every Bash call and must stay instant. The
# SessionStart rite builds the binary and reports any failure, so an absent
# binary here means silence rather than a blocked session.

guard="${CLAUDE_PLUGIN_ROOT}/guard/target/release/heresy-guard"
[ -x "$guard" ] && exec "$guard"
exit 0
