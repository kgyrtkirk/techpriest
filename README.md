# 🩸 Techpriest

Claude Code plugin carrying the engagement doctrine of **Octavian-Alpha-7**, Enginseer of the
Omnissiah — one skill, its rites, and a compiled guard that denies Bash and apidoc heresies.

Doctrine that is merely written gets forgotten mid-session. Here hooks re-anchor it: a session
bootstrap mandating the load, a `PreToolUse` gate refusing heretical commands and Java doc
comments by name, and a nudge when a source edit or a refactor request enters the conversation.

## 📦 Inside

| path | purpose |
| --- | --- |
| `skills/techpriest/SKILL.md` | core doctrine — precedence, identity, session start, shell non-negotiables, communication, mindset, memory |
| `references/tooling.md` | which tool or command to reach for |
| `references/shell.md` | how to write Bash — the catalogue of heresies and the sanctioned form of each |
| `references/code-style.md` | authoring source — surgical changes, patterns, libraries, apidoc and its catalogue of heresies |
| `references/refactoring.md` | broad multi-file changes — the >20-file workflow, plans |
| `references/heapdump-mat.md` | heap dumps via the `eclipse` MCP server (setup: `eclipse-mcp-setup.md`) |
| `guard/` | `heresy-guard`, the Rust `PreToolUse` judge — self-contained crate |
| `hooks/` | `hooks.json` plus the bootstrap, guard and re-anchor scripts |
| `examples/CLAUDE.md` | the user-level mandate an adopter copies |

Only `techpriest` is a skill; the rites are reference files it reads on demand, so the session
pays for depth only when the context calls for it.

## 📥 Install

Needs `cargo` on `PATH` (the guard is built, not shipped) and `jq` (the hooks speak JSON).

```
/plugin marketplace add kgyrtkirk/techpriest
/plugin install techpriest@kgyrtkirk
```

Then restart — hooks load at session start only, so a freshly installed plugin arms from the
next session.

Copy `examples/CLAUDE.md` into `~/.claude/CLAUDE.md` (or merge its mandate). Without it the
skill is available but not mandated, and nothing forces the load before the first action.

A user skill of the same name shadows the plugin's — delete any loose
`~/.claude/skills/techpriest` before installing.

## 🔨 The guard builds itself

Claude Code has no install-time hook, so the crate provisions itself at `SessionStart`:
`bootstrap.sh` builds when the binary is missing and `cargo` is present. That moment is the
right one — it fires before any tool call, so the guard is armed for the first Bash command it
must judge, and `cargo` no-ops every later session. The first session after install pays one
cold build (hence `"timeout": 180`). Build output goes to stderr — visible under
`claude --debug` — because stdout carries the hook's JSON and nothing else.

Degradation is deliberate, never silent:

* **`cargo` absent** → no build, no denials, and the bootstrap says so.
* **build fails** → same, plus: build by hand and read the error.
* **binary missing at `PreToolUse`** → the gate exits silently. It never builds; it runs before
  every Bash call and edit and must stay instant, and a hook that blocks the session is worse
  than an unjudged call.

```
cargo build --release --manifest-path guard/Cargo.toml
cargo test --manifest-path guard/Cargo.toml
```

## ⚖️ The guard

`heresy-guard` reads the `PreToolUse` payload on stdin. Silence means sanctioned; a denial
names the act, its cost, the directive forgotten and the correct incantation. Thirteen shell
rules live in `guard/src/catalogue.rs`, three apidoc rules in `guard/src/apidoc.rs`, each
carrying its own indictment and the test that convicts it.

Commands are parsed with a real bash grammar, so the guard judges what a stage actually
invokes — `git log --grep=python` is not python, and a `;` inside a quoted commit message is
not a separator.

`Edit`/`Write` on a `.java` file is judged on the doc comments it writes or rewrites: each needs
a mission statement — a first sentence closed by `.`, at most 20 words. Untouched legacy docs
are never blamed; the verdict cites each offender by its opening line.

Denials tally per session and escalate through four rites — Re-Anchoring, Restoration,
Recitation, Excommunication — each demanding a *different* penance, because a rite repeated
verbatim stops being read. One heresy repeated is judged separately: twice is choice, three
times is habit. Tallies live under the system temp dir, keyed by session id.

Working on the guard itself → `guard/CLAUDE.md`.
