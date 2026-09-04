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
| `guard/` | `heresy-guard`, the Rust `PreToolUse` judge — a self-contained crate |
| `hooks/` | `hooks.json` plus the bootstrap, guard and re-anchor scripts |
| `examples/CLAUDE.md` | example user-level `CLAUDE.md` mandating the doctrine load |

Only `techpriest` is a skill. The three rites are reference files it reads on demand, so the
session pays for depth only when the context calls for it.

## ⚙️ Requirements

* **Rust toolchain** — `cargo` on `PATH`; the guard is built, not shipped.
* **`jq`** — the bootstrap and re-anchor hooks speak JSON.

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

Restart the session — hooks load only at session start, so a freshly installed plugin's
hooks take effect from the next one. Confirm the skill is listed and the bootstrap context
carries no build warning.

## 🔨 The guard builds itself

Claude Code has no install-time hook, so the crate provisions itself at `SessionStart`:
`bootstrap.sh` builds it when the binary is missing and `cargo` is present. That moment is
the right one — `SessionStart` fires before any tool call, so the guard is armed before the
first Bash command it must judge, and `cargo` no-ops on every later session.

The first session after install therefore pays one cold build (hence `"timeout": 180` on the
hook); afterwards the check costs milliseconds. Build output goes to stderr — visible under
`claude --debug` — because stdout carries the hook's JSON and nothing else.

Degradation is deliberate rather than silent:

* **`cargo` absent** → no build, no denials, and the bootstrap says so.
* **Build fails** → same, and the bootstrap tells you to run the build by hand and read the error.
* **Binary missing at `PreToolUse`** → the gate exits silently. It never builds; it runs before
  every Bash call and must stay instant, and a hook that blocks the session is worse than an
  unjudged command.

To build or test by hand:

```
cargo build --release --manifest-path guard/Cargo.toml
cargo test --manifest-path guard/Cargo.toml
```

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
incantation. Thirteen rules live in `guard/src/catalogue.rs` — each carries its own
indictment and the test that convicts it, so a rule cannot be half-defined or left unwired.

Denials tally per session and escalate through four rites — Re-Anchoring, Restoration,
Recitation, Excommunication — each demanding a *different* penance, because a rite repeated
verbatim stops being read. Repeating one specific heresy is judged separately: twice is
choice, three times is habit. Tallies live under the system temp dir, keyed by session id.

To add a heresy: add one `Rule` to `guard/src/catalogue.rs`, add its sample command to the
test table, and document it in `skills/techpriest/references/shell.md`. The tests fail if a
rule lacks a sample or a sample fails to convict.

## 🔍 Verify the guard by hand

```
G=guard/target/release/heresy-guard
echo '{"session_id":"smoke","tool_input":{"command":"less f.txt"}}' | $G
```

A sanctioned command prints nothing:

```
echo '{"session_id":"smoke","tool_input":{"command":"git grep -n foo"}}' | $G
```
