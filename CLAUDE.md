# 🩸 techpriest

Claude Code plugin. **The doctrine is the product**; the code only enforces it.

| path | what it is |
| --- | --- |
| `skills/techpriest/` | the doctrine — `SKILL.md` plus three `references/` rites (tooling, shell, code-style) |
| `guard/` | `heresy-guard`, a Rust `PreToolUse` hook denying Bash heresies. Has its own `CLAUDE.md` |
| `hooks/` | `hooks.json` and the bash entry points the harness actually calls |
| `examples/CLAUDE.md` | the user-level mandate an adopter copies |

## ⚖️ The one invariant

`guard/src/catalogue.rs` and `references/shell.md` are the same rite in two forms —
executable and written. **Change one, change the other.** A denial quotes the written
directive back at the offender, so drift between them makes the guard lie.

## 🧭 Working here

* Prose is load-bearing: the skill files are read by a model, not a compiler. Terse, iconed,
  imperative — match the surrounding voice.
* The plugin must stand alone. No personal paths, no host-specific assumptions, no tools an
  adopter would not have.
* Doctrine binds the authors too: edits here obey the rites in `skills/techpriest/`.

```
cargo test --manifest-path guard/Cargo.toml
```
