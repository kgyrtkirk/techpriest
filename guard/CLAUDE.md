# ⚖️ heresy-guard

`PreToolUse` judge. Payload on stdin → silence (sanctioned) or a deny verdict naming the act,
its cost, the directive broken and the correct incantation. `Bash` calls meet the shell
catalogue; `Edit`/`Write` on `.java` meet the apidoc catalogue.

```
cargo test                      # add -- --ignored to see the known defects
cargo build --release
./target/release/heresy-guard < payload.json
```

## 🏗️ Shape

| file | role |
| --- | --- |
| `rule.rs` | `Rule<J>` — one heresy, generic over what it judges |
| `command.rs` | parses the command into pipeline `Stage`s via `brush-parser` |
| `catalogue.rs` | the 13 shell `Rule`s. The **only** place they are registered |
| `javadoc.rs` | reads doc comments out of Java source into `Doc`s — lead, mission |
| `apidoc.rs` | the 3 apidoc `Rule`s, and which docs an edit answers for |
| `ledger.rs` | per-session tallies in temp-dir counter files, shared by both catalogues |
| `verdict.rs` | the deny message, its citations, and the 4-rung ladder of rites (3/6/10/15) |
| `main.rs` | payload → `judge()` by tool name → deny JSON |

## 🔍 Judge stages, never raw text

Shell syntax is parsed with a real bash grammar, not split on separator bytes. Hand-splitting
could not tell a `;` inside `git commit -m "fix; …"` from a true one, and a pattern loosed on
the whole string read arguments as invocations.

So there is **no way to match the raw command text** — that API was removed deliberately.

* `Stage::runs(pattern)` — matches the *program* invoked, directory stripped. Use this for
  "is this X?". It is why `git log --grep=python` is not python.
* `Stage::matches(pattern)` — matches one stage's text, for rules about how a stage is written.
* `is_source()` / `diverts_output()` / `is_heredoc()` / `Command::loops()` come from the AST,
  not from regex guesswork.

A compound command is one opaque stage; its body escapes the stage rules. Input the grammar
rejects is judged whole — bash would reject it too, so it will never run.

## ➕ Adding a heresy

1. One `Rule` in `catalogue.rs` (shell) or `apidoc.rs` (apidoc). The `id` is a **stable tally
   key** — rewording it resets counts; ids are unique across both catalogues.
2. A sample in that file's `SAMPLES`; tests fail if a rule lacks one or the sample fails to convict.
3. Document it in `../skills/techpriest/references/shell.md` or `code-style.md`. Same rite, two forms.

False positives cost more than misses — the verdict is a hard `deny` with no override, so a
wrong one leaves the user stuck. Add the sanctioned near-miss to `sanctioned_commands_pass`
(apidoc: `sanctioned_docs_pass`).

## 📜 Apidoc: judge only what the edit wrote

The guard rebuilds the file the edit produces — `Write` content, or `Edit` applied to the file
on disk — and judges only docs whose lead is new (lead-less docs: any change). An untouched
legacy doc is never the editor's to answer for; blaming it would block unrelated edits.

## 🩹 Known defects

`catalogue::tests::defects` holds one test per known-wrong case. The `#[ignore]`d ones are
outstanding; each names what is broken. Fix one → drop its `#[ignore]`.
