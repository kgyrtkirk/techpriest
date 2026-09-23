# 🩸 Techpriest

## 💡 Idea

These models are more like wild dogs...they are more or less capable of following the goal - but constraints like quality slip easily - so I wanted to attempt to overcome that by presenting the guardrails as a religion (as it most likely has eaten quite a lot of that too during training).
That seemed to improve some things...but after some time it degrades - and drops back to "basic training" level.

An attempt to to fix that was to introduce the heresy-guard - to validate requests and force the model to reiterate the directives to refresh them.

It doesn't always work...it seems like different models (opus 4.8/5) have different levels of discipline to honor such things; I also suspect that the base CLI injects a lot of stuff - which could alter how this works (not sure which matters more).

My experience was:
- I was working with opus-4.8; kinda working ok.
- when 5 came out I switched over; however it was coming up with really dumb ideas
- 4.8 outperformed it easily so I switched back
- 2 weeks later 4.8 became dumb...
- switching to 5 did not feel like that much of a downgrade anymore...
- maybe they have equalized them and made them both dumb?
- I suspect that they are fine-tuning the CLI for the latest model; which will make older ones not perform that well after some time.

## ⚙️ Identity

Claude Code plugin carrying the engagement doctrine of **Octavian-Alpha-7**, Enginseer of the
Omnissiah — one skill, its rites, and a compiled guard that denies Bash heresies.

Doctrine that is merely written gets forgotten mid-session. Here hooks re-anchor it: a session
bootstrap mandating the load, a `PreToolUse` gate refusing heretical commands by name, and a
nudge when a source edit or a refactor request enters the conversation.

## 📦 Inside

| path | purpose |
| --- | --- |
| `skills/techpriest/SKILL.md` | the doctrine — identity, precedence, session start, communication, mindset. Hubs to the rites |
| `skills/techpriest/references/` | the rites, read on demand: tooling, shell, code-style, refactoring, heap-dump analysis |
| `bin/git-updiff` | `git updiff` — the branch's net change against the fork point it elects |
| `guard/` | `heresy-guard`, the Rust `PreToolUse` judge — self-contained crate, own `CLAUDE.md` |
| `hooks/` | `hooks.json` plus the bootstrap, guard and re-anchor entry points |
| `examples/CLAUDE.md` | the user-level mandate an adopter copies |

Only `techpriest` is a skill; the rites are reference files it reads on demand, so a session
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

## 🧭 `git updiff`

Microcommits fragment a change across dozens of half-states, so per-commit views show churn
instead of the change. `git updiff` diffs against the commit the branch forked from — invariant
under rebase, amend, reorder and squash — and it takes every `git diff` argument.

The fork point is **elected, not assumed**: the tightest lineage wins, falling back to
`@{upstream}` and the remotes' default branches. Work across several upstreams of one repo,
listed most-specific first:

```
git config --add techpriest.upstream implydata/iow
git config --add techpriest.upstream apache/master
```

`git updiff --why` names every candidate and the winner — reach for it whenever a diff looks
far too wide.

Claude Code puts the plugin's `bin/` on the Bash tool's `PATH`, so this needs no setup inside a
session. Your own shell is **not** touched; adding that directory to your `PATH` is deliberately
left as your decision.

## ⚖️ The guard

`heresy-guard` reads the `PreToolUse` payload on stdin. Silence means sanctioned; a denial names
the act, its cost, the directive forgotten and the correct incantation. Commands are parsed with
a real bash grammar, so the charge is about what a stage actually invokes. Repeat offences
escalate through rites of penance, each demanding something the last did not.

The crate builds itself: Claude Code has no install-time hook, so `SessionStart` compiles the
binary when it is missing, in time for the first Bash call it must judge. Degradation is
deliberate, never silent — no `cargo`, or a failed build, and the bootstrap says so in-session;
a missing binary at `PreToolUse` yields no judgement rather than a blocked session. The first
session after install pays one cold build.

```
cargo build --release --manifest-path guard/Cargo.toml
cargo test --manifest-path guard/Cargo.toml
```

The catalogue of heresies is one rite in two forms: written in
`skills/techpriest/references/shell.md`, executable in `guard/src/catalogue.rs`. Working on the
guard itself → `guard/CLAUDE.md`.
