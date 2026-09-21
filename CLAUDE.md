# 🩸 techpriest

Claude Code plugin, published as `github.com/kgyrtkirk/techpriest` (marketplace `kgyrtkirk`).
**The doctrine is the product**; the code only enforces it.

| path | what it is |
| --- | --- |
| `skills/techpriest/` | the doctrine — `SKILL.md`, hubbing to the `references/` rites |
| `guard/` | `heresy-guard`, a Rust `PreToolUse` hook denying Bash heresies. Has its own `CLAUDE.md` |
| `hooks/` | `hooks.json` and the bash entry points the harness actually calls |
| `bin/` | executables Claude Code puts on the Bash tool's `PATH` — `git-updiff`. A top-level `bin/` bars org distribution: weigh that before adding |
| `examples/CLAUDE.md` | the user-level mandate an adopter copies |

## ⚖️ Invariants

* `guard/src/catalogue.rs` and `references/shell.md` are one rite in two forms — executable and
  written. **Change one, change the other.** A denial quotes the written directive back at the
  offender, so drift makes the guard lie.
* **One written form per fact.** Rule ids, counts, thresholds, algorithms, install steps and the
  rite list live with whatever enforces them; every other mention states the contract and points
  at the owner. A second copy is scope creep — it drifts inside one release and then lies.
  Sole sanctioned duplicate: the `SKILL.md` non-negotiables, **names only**, because they must
  bind before `shell.md` is ever loaded.
* Every rite is reachable: listed in the `SKILL.md` hub, and named by whatever hook nudges it.

## 🧭 Working here

* Everything must only cover the essence/specification - absolutely no medical history and other crap!
* these will be read by a model - Terse, use icons to compress information! imperative!
* The plugin stands alone. No personal paths, no host assumptions, no tool an adopter lacks.
* If a skill related file needs changes: the full file must be read - and re-evaluate how to add those details.
* Doctrine binds the authors: edits here obey the rites in `skills/techpriest/`.

```
cargo test --manifest-path guard/Cargo.toml
```
