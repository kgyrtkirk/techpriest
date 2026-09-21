# 🐚 Shell Rite — How To Write Bash

Companion to `techpriest`. Every heresy below is enforced by `heresy-guard`, the plugin's
`PreToolUse` hook: it denies the command and names the act, its cost, the directive broken
and the correct incantation. Rule ids are stable — they key the session tallies.

The catalogue in `guard/src/catalogue.rs` is the executable form of this rite. Change one,
change the other.

The guard parses your command with a real bash grammar and judges the program each stage
invokes. Naming a tool is not running it, and a separator inside quotes is not a separator —
so quote honestly and expect the charge to be about what you actually ran.

## 📜 Practices

* **Read the whole output** — run commands regardless how much they produce; the system
  auto-saves oversized output to a file. Truncation blinds you.
* **Commands ≤ 300 chars** — longer means you are scripting; do it in steps instead.
* **Run the command you were given first** — before digging into what the script it calls
  does. All tools, commands and scripts named to you already exist. Trust them.
* **Know where you are** — `pwd` when a command fails unexpectedly, before retrying.
* **One purpose per invocation** — compound `&&` chains that mix unrelated work make the
  failing step ambiguous.

## ⚖️ The Catalogue of Heresies

### `read-bypass` — `cat`/`head`/`tail`/`less`/`sed -n` to view a file
* **why**: the `Read` tool reads files natively and supports line ranges.
* **correct**: `Read` with `offset`/`limit`.
* **spared**: `tail -f` (monitoring, not reading), heredocs, `cat -`, and a viewer whose
  output is piped or redirected (that is plumbing, not reading).

### `truncation` — `| head` / `| tail`
* **why**: you blind yourself; full output is auto-saved even when huge.
* **correct**: drop the pipe and read it all.

### `useless-cat` — `cat X | …`
* **why**: the downstream tool reads the file directly; the `cat` is dead weight.
* **correct**: `grep pat FILE`, not `cat FILE | grep pat`.

### `grep-over-java` — plain `grep` over `.java` files
* **why**: `git grep` is version-control aware, faster, ignores build noise.
* **correct**: `git grep -nP 'pattern' '**/*.java'`.

### `recursive-grep` — `grep -r`/`-R` over the working tree
* **why**: `git grep` is scoped to tracked files, respects `.gitignore`, and is faster.
* **correct**: `git grep`.
* **spared**: external source trees — `~/.m2`, `~/inspection`, `~/.cargo`, `/usr`, `/etc`,
  `/var` — and any `grep` fed by a pipe (it filters upstream output, it searches nothing).
* **bind**: only inside a git working tree.

### `forbidden-interpreter` — `perl` / `python`
* **why**: they hide logic in throwaway scripts nobody reviews.
* **correct**: bash builtins, `jq`, `awk`, `sed`.

### `hand-rolled-loop` — `for x in a b c; do … ; done`
* **why**: opaque, hard to read output per item, silent-fail prone.
* **correct**: run the commands individually, or drive a real file list via `find -exec` /
  `xargs`.
* **binds**: `for`, `while` and `until`, however the loop is written — the newline form is
  no escape.

### `cd-into-cwd` — `cd` into the directory you already run in
* **why**: a no-op; you are already there.
* **correct**: remove the `cd`.

### `cd-self` — `cd .` / `cd $(pwd)` / `cd $PWD`
* **why**: resolves to the dir you are already in.
* **correct**: remove the `cd`.

### `echo-exit-code` — `echo $?`
* **why**: the harness already reports exit status.
* **correct**: branch on the command directly, or read the reported exit code.

### `mvn-absolute-path` — `mvn` by absolute path, e.g. `/usr/bin/mvn`
* **why**: it bypasses the wrapper that emits the compact build summary — `.git/bin/mvn`, or
  whatever the environment resolves `mvn` to.
* **correct**: `.git/bin/mvn` when the clone has a wrapper, otherwise bare `mvn`.

### `mvn-diverted` — `mvn` piped or redirected
* **why**: the wrapper silences stdout — pipes and redirects produce garbage.
* **correct**: use `mvn` flags directly; read the full output; rely on the exit code.

### `remote-exec` — `curl … | bash`
* **why**: running unreviewed network content as a shell — supply-chain heresy.
* **correct**: download to a file, inspect it, then run deliberately.

## 🪜 The Ladder of Rites

Denials accumulate per session and escalate through four rungs. Each demands a **different**
penance, because a rite repeated verbatim stops being read — the denial names the rung you
earned and the penance it exacts. Obey that text; it is the authority. A repeat of the *same*
heresy is judged separately: twice is choice, three times is habit.
