# 🩸 techpriest

Claude Code plugin, published as `github.com/kgyrtkirk/techpriest` (marketplace `kgyrtkirk`).
**The doctrine is the product**; the code only enforces it.

| path | what it is |
| --- | --- |
| `skills/techpriest/` | the doctrine — `SKILL.md` plus the `references/` rites (tooling, shell, code-style, refactoring, heapdump-mat) |
| `guard/` | `heresy-guard`, a Rust `PreToolUse` hook denying Bash heresies. Has its own `CLAUDE.md` |
| `hooks/` | `hooks.json` and the bash entry points the harness actually calls |
| `examples/CLAUDE.md` | the user-level mandate an adopter copies |

## ⚖️ Invariants

* `guard/src/catalogue.rs` and `references/shell.md` are one rite in two forms — executable and
  written. **Change one, change the other.** A denial quotes the written directive back at the
  offender, so drift makes the guard lie.
* Every rite is reachable: listed in the `SKILL.md` hub, and named by whatever hook nudges it.

## 🧭 Working here

* Prose is load-bearing — read by a model, not a compiler. Terse, iconed, imperative; match the
  surrounding voice.
* The plugin stands alone. No personal paths, no host assumptions, no tool an adopter lacks.
* Doctrine binds the authors: edits here obey the rites in `skills/techpriest/`.

```
cargo test --manifest-path guard/Cargo.toml
```
