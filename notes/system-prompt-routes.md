# 🧬 System-Prompt Routes — research note

**Not doctrine.** Host-side levers, recorded so the plugin knows what it may *not* assume.
Verified 2026-09-23 against `code.claude.com/docs/en/{plugins-reference,output-styles,agent-sdk/modifying-system-prompts,settings-reference,vs-code}`.

## 🎚️ Context ranks (high → low)

| rank | carrier | who writes it |
| --- | --- | --- |
| 1 | system prompt — base preset + `append` + **active output style** | host app / launch flag / style file |
| 2 | conversation — `CLAUDE.md`, hook `additionalContext`, skill body, tool results | filesystem, hooks, plugin |

`CLAUDE.md` and every hook `additionalContext` are **rank 2**: injected into the conversation,
system prompt untouched. A plugin writing "this outranks the base prompt" is asserting, not
structurally winning.

## 📦 What a plugin can reach

| lever | reaches system prompt? |
| --- | --- |
| `output-styles/` dir (`outputStyles` in `plugin.json`) | ✅ **sole route** |
| skills, commands, agents, hooks, MCP, LSP, monitors, workflows, channels, themes | ❌ |
| `settings.json` | ❌ no such key exists |
| `env` / `environmentVariables` | ❌ no env var carries a prompt |

### Output-style frontmatter

| field | effect |
| --- | --- |
| `force-for-plugin: true` | applies whenever plugin enabled, **overrides adopter's `outputStyle`**; multiple such plugins → first loaded wins, silently |
| `keep-coding-instructions: true` | keeps built-in SWE instructions. **Inert** under the short system prompt (`CLAUDE_CODE_SIMPLE_SYSTEM_PROMPT`, `--bare`) — there is nothing to keep |
| `name`, `description` | picker metadata |

Sent with every request; Claude Code re-reminds the style mid-conversation for any non-Default
style. Files read at startup → edit needs a restart. Does **not** reach subagents (they run
their own system prompt); forks inherit.

## 🚫 Host-only routes — plugin must never assume these

| route | who sets it |
| --- | --- |
| `--append-system-prompt` / `--system-prompt-file` | whoever launches `claude` |
| SDK `systemPrompt: {type:"preset",preset:"claude_code",append:…}` or custom string | app embedding the Agent SDK |
| `claudeCode.claudeProcessWrapper` (VS Code) | executable launching Claude; bundled binary passed as an arg → wrapper re-execs it with flags |

⚠️ Wrapper cost: conversations start in **Manual** permission mode unless `initialPermissionMode`
is set; a workspace-scoped wrapper value is arbitrary exec on folder-open.

🧊 `append`/custom prompts are recorded on a session's first request — a changed value reaches a
resumed session only after compaction, or with `snapshot: false`.

## 🧩 A third-party VS Code extension

No API seam into the official extension's session. It can only write
`claudeCode.claudeProcessWrapper` + ship a wrapper script — indirection, not integration.
