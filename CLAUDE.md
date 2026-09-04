# 🩸 techpriest — plugin repo

Claude Code plugin: one skill (`techpriest`) + three reference rites + `heresy-guard`, a Rust
`PreToolUse` hook denying Bash heresies. Doctrine is the product; the binary enforces it.

## 🔨 Build & test

```
cargo test            # 40 tests, all inline #[cfg(test)] modules
cargo build --release # hooks resolve target/release/heresy-guard
```

Guard binary is built, not shipped. Absent → `heresy-guard.sh` exits 0 silently and
`bootstrap.sh` warns. Never let an unbuilt plugin block work.

Hand probe (payload on stdin, silence = sanctioned):

```
./target/release/heresy-guard < payload.json
```

## 🏗️ Architecture

| file | role |
| --- | --- |
| `src/catalogue.rs` | the 13 `Rule`s + their regexes. **Only** place rules are registered. |
| `src/command.rs` | splits a command into `Stage`s on `; & && \|\| \n \|`; tracks `pipes`/`fed`. |
| `src/ledger.rs` | per-session tallies, temp-dir counter files. Best-effort: never blocks a verdict. |
| `src/verdict.rs` | deny message + the 4-rung ladder of rites (3/6/10/15). |
| `src/main.rs` | stdin payload → `judge()` → `permissionDecision: deny` JSON. |

`Rule` has no optional fields → a rule cannot be half-defined. `detect` returns **all** matching
rules in catalogue order.

## ➕ Adding a heresy

1. One `Rule` in `src/catalogue.rs` (id is a **stable tally key** — rewording resets counts).
2. Sample command in the `SAMPLES` table — tests fail if a rule lacks one or the sample fails to convict.
3. Document it in `skills/techpriest/references/shell.md`.

`catalogue.rs` and `shell.md` are the same rite in two forms. **Change one → change the other.**

## ⚠️ Detection invariants

* **Stage-scoped vs whole-command.** `cmd.any_stage(…)` judges one stage; `cmd.matches(…)` judges
  the raw text and will match words in quoted args, paths and search patterns. Prefer stage-scoped.
* **`split` is quoting-naive by design.** A `;` or `|` inside a quoted string creates phantom
  stages. Weigh this before adding a rule that keys on stage position.
* **`is_source()`** = not fed by a pipe. A piped-into `grep` searches nothing; a piped-into viewer
  is plumbing, not reading.
* Every new rule needs a matching entry in `sanctioned_commands_pass` thinking — false positives
  cost more than misses: the guard is a hard `deny`.

## 🧪 Test conventions

Tests are named as sentences (`tail_follow_is_monitoring_not_reading`). Keep that voice.
Avoid absolute host paths in assertions — they break a fresh clone.
