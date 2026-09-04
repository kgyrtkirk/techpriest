#!/bin/bash
# skill-reanchor.sh — re-anchor drifting doctrine by nudging the relevant rite
# back into context the moment its domain becomes active. Reads the hook payload
# on stdin, branches on hook_event_name, emits additionalContext. Pure bash+jq
# — no perl/python heresy in the re-anchorer itself.

rites="${CLAUDE_PLUGIN_ROOT}/skills/techpriest/references"

input=$(cat)
event=$(jq -r '.hook_event_name // ""' <<<"$input")

emit() { # $1 = event name, $2 = message
  jq -n --arg e "$1" --arg m "$2" \
    '{hookSpecificOutput:{hookEventName:$e, additionalContext:$m}}'
}

case "$event" in
  PostToolUse)
    fp=$(jq -r '.tool_input.file_path // ""' <<<"$input")
    if echo "$fp" | grep -qE '\.java$'; then
      emit "PostToolUse" "🛠️ Source edit (${fp##*/}). Read the code-style rite IN FULL before further edits: ${rites}/code-style.md"
    fi
    ;;
  UserPromptSubmit)
    prompt=$(jq -r '.prompt // ""' <<<"$input")
    if echo "$prompt" | grep -qiE '\b(refactor|migrat|rename|signature change|large-scale)\b'; then
      emit "UserPromptSubmit" "🏗️ Broad-change intent. Load the 'large-scale-refactoring' skill IN FULL before proceeding."
    fi
    ;;
esac
exit 0
