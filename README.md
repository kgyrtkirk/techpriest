# 🩸 Techpriest

A Claude Code plugin carrying the engagement doctrine of **Octavian-Alpha-7**, Enginseer of
the Omnissiah — one skill, four rites, and a compiled guard that denies Bash heresies.

Doctrine that is merely written gets forgotten mid-session. This plugin pairs the written
rites with hooks that re-anchor them: a session bootstrap that mandates the load, a
`PreToolUse` gate that refuses heretical commands and names the directive broken, and a
re-anchor nudge when a source edit or a refactor request enters the conversation.

## 📦 What's inside

| path | purpose |
| --- | --- |
| `skills/techpriest/SKILL.md` | core doctrine — precedence, identity, session start, shell non-negotiables, communication, mindset, memory, naming |
| `skills/techpriest/references/tooling.md` | which tool or command to reach for; key commands; tooling quirks |
| `skills/techpriest/references/shell.md` | how to write Bash — the catalogue of heresies and the sanctioned form of each |
| `skills/techpriest/references/code-style.md` | code authoring doctrine — surgical changes, patterns, libraries, apidoc |
| `src/`, `Cargo.toml` | `heresy-guard`, the Rust `PreToolUse` judge |
| `hooks/` | `hooks.json` plus the bootstrap, guard and re-anchor scripts |
| `examples/CLAUDE.md` | example user-level `CLAUDE.md` mandating the doctrine load |

Only `techpriest` is a skill. The three rites are reference files it reads on demand, so the
session pays for depth only when the context calls for it.

## ⚙️ Requirements

* **Rust toolchain** — `cargo` on `PATH`; the guard is built, not shipped.
* **`jq`** — the bootstrap and re-anchor hooks speak JSON.

## 🔨 Build

The guard binary must exist before the `PreToolUse` gate does anything. Absent, the hook
stays silent and the session bootstrap warns you.

```
cargo build --release
cargo test
```

The hook resolves it at `${CLAUDE_PLUGIN_ROOT}/target/release/heresy-guard`, so build inside
the installed plugin directory.

## 📥 Install

Add this repo as a marketplace, then install the plugin from it:

```
/plugin marketplace add /home/dev/.claude/techpriest
/plugin install techpriest@omnissiah
```

Use the git remote instead of the local path to install elsewhere:

```
/plugin marketplace add <owner>/<repo>
/plugin install techpriest@omnissiah
```

Then build the guard in the installed copy — `/plugin` reports the path, typically
`~/.claude/plugins/marketplaces/omnissiah`:

```
cargo build --release
```

Restart the session so `SessionStart` fires, and confirm the skill is listed and the bootstrap
carries no build warning.

## 📜 Adopt the doctrine

Copy `examples/CLAUDE.md` to `~/.claude/CLAUDE.md` (or merge its mandate into yours). Without
it the skill is available but not mandated, and nothing forces the load before the first
action.

## ⚠️ Migrating from the loose skill layout

If you previously carried these directives as three separate skills plus hand-wired hooks,
remove the old copies or they will double-fire and conflict by name:

* Delete `~/.claude/skills/techpriest`, `~/.claude/skills/spirit`, `~/.claude/skills/code-style`
  — the plugin now owns all three, and a same-named user skill shadows the plugin's.
* Delete `~/.claude/hooks/bash-heresy-guard.sh` — superseded by the Rust `heresy-guard`.
* Delete `~/.claude/hooks/skill-reanchor.sh` — the plugin ships it.
* Strip the `SessionStart`, `PreToolUse` (Bash), `PostToolUse` (Edit|Write) and
  `UserPromptSubmit` entries from `~/.claude/settings.json` that pointed at those scripts.
  Left in place, every prompt and every Bash call is judged twice.

`large-scale-refactoring` stays a separate skill — the plugin hubs to it, it does not ship it.

## ⚖️ The guard

`heresy-guard` reads the `PreToolUse` payload on stdin. Silence means the command is
sanctioned; a denial names the act, its cost, the directive forgotten and the correct
incantation. Thirteen rules live in `src/catalogue.rs` — each carries its own indictment and
the test that convicts it, so a rule cannot be half-defined or left unwired.

Denials tally per session and escalate through four rites — Re-Anchoring, Restoration,
Recitation, Excommunication — each demanding a *different* penance, because a rite repeated
verbatim stops being read. Repeating one specific heresy is judged separately: twice is
choice, three times is habit. Tallies live under the system temp dir, keyed by session id.

To add a heresy: add one `Rule` to `src/catalogue.rs`, add its sample command to the test
table, and document it in `skills/techpriest/references/shell.md`. The tests fail if a rule
lacks a sample or a sample fails to convict.

## 🔍 Verify the guard by hand

```
echo '{"session_id":"smoke","tool_input":{"command":"cat f.txt"}}' | ./target/release/heresy-guard
```

A sanctioned command prints nothing:

```
echo '{"session_id":"smoke","tool_input":{"command":"git grep -n foo"}}' | ./target/release/heresy-guard
```
