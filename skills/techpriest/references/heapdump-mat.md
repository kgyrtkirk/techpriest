# 🧠 Heap Dump Rite — MAT Inside Eclipse Over MCP

Companion to `techpriest`. Read before analysing a Java heap dump through the `eclipse` MCP
server. Binding while that work lasts.

The heap dump is opened by a human in a running Eclipse; the agent drives MAT through the
widget layer. Everything here is what actually works — not what the tool descriptions imply.

## 🏗️ Setup (one time)

1. **Install a recent Eclipse IDE for Java Developers** — e.g. `eclipse-2026-09-R-java`. Take
   the full Java package, not a stripped platform.
2. **Install the vogella MCP server** into that Eclipse (Help → Install New Software, or drop
   the bundle into `plugins/`).
3. **Install the Memory Analyzer (MAT) plug-in** into the *same* installation — `Memory
   Analyzer` plus `Memory Analyzer (Charts)`.
4. **Install `MatCalcitePlugin`** (vlsi) into the same installation. Not optional in practice —
   see 🔷 below.
5. **Enable the MCP server**: Preferences → General → MCP Server. The call timeout lives here
   too; raise it when queries outlive the default.
6. **Open the heap dump inside the IDE** (`File → Open File…`). MAT parses and indexes it; the
   agent works against that live snapshot.

### 🔷 Why the IDE, not standalone MAT

Standalone MAT is a stripped RCP app. The MCP server needs far more of the platform than it
ships — workspace, JDT, the command and handler framework, editors and views the tools
address. Start from the **Eclipse Java IDE** and add MAT to it, never the other way round.

The payoff beyond querying: the same IDE holds the *project source*. Extract a problematic
structure from the dump, reinstate it as a fixture in a test, run it, and keep the analysis
going in code — dump and repro in one workspace.

## 🖥️ Session preconditions

* **X11 required.** `press_key` uses `Display.post`; Wayland compositors silently drop it.
  Check `/tmp/.X11-unix/X0`. No X11 → keyboard-driven MAT is dead, and most of this rite with it.
* **`xclip` installed**, reachable as `DISPLAY=:0 xclip -o -selection clipboard`.
* The IDE must be **foreground** for any key post. This steals the user's focus every time —
  unavoidable, so batch work and say so.

## 🔁 The query loop — do exactly this

Every MAT query runs through the Query Browser command. Order is not negotiable:

```
eclipse_set_ide_visibility   visible: true          # foreground, else press_key refuses
eclipse_set_part_state       part: …HeapEditor, state: activated
eclipse_run_workbench_command org.eclipse.mat.ui.query.browser.QueryBrowser
                              parameters: { …QueryBrowser.commandName: "<command line>" }
eclipse_press_key            key: Enter             # commandName only PREFILLS — this runs it
eclipse_wait_until_settled   timeoutSeconds: 5      # lets the job get scheduled
eclipse_wait_until_quiet     timeoutSeconds: 25     # names the running job = the query text
```

* **Front the IDE *before* opening the browser.** The popup is a focus-follow shell; fronting
  afterwards closes it.
* **Activate the heap editor first.** If another editor is active the query has no snapshot
  context, runs against nothing, and the Enter lands in that editor.
* **`outcome: "success"` is meaningless.** It reports that a handler returned, not that a query
  ran. Verify by the new result tab, or by the job name in `wait_until_quiet`.
* **Never call `wait_until_quiet` straight after Enter** — it returns `quiet` before the job is
  scheduled. `wait_until_settled` first, then wait.
* **`run_script` has a 30 s budget.** Long queries: run the steps, then call `wait_until_quiet`
  separately, repeatedly.
* **Print the SQL pretty-formatted to the console before running it.** Always. The escaped
  one-liner is noise — show it only when the tokenizer itself is the problem.

## 🔤 Command-line quoting

MAT's `CommandLine.tokenize`: `"` toggles quoting, `\` escapes the next char. So SQL with
double-quoted identifiers goes through as:

```
calcite "select … from \"io.example.Foo\" order by 2 desc limit 5"
```

Single quotes are **not** string delimiters to the tokenizer — safe to use freely inside SQL.

## 🔷 Calcite SQL — the primary instrument

`MatCalcitePlugin` registers MAT query `calcite` ("SQL via Calcite"). It has `ORDER BY`,
`GROUP BY`, aggregates and joins. **MAT's own OQL has no `ORDER BY`** — grammar is
`SELECT [DISTINCT] [OBJECTS | AS RETAINED SET] … FROM [INSTANCEOF] <class> [alias] [WHERE …]`.
Reach for `calcite` first; fall back to `oql` only for object-set arguments.

**Tables** — `"fqcn"`, `instanceof.fqcn` for subclasses, `java.lang."HashMap$Entry"` for nested.

**Virtual columns** — bracket syntax, not dotted:

| expression | meaning |
|---|---|
| `this['@shallow']` | shallow heap size |
| `this['@retained']` | retained heap size |
| `this['@class']` / `this['@className']` | class / class name |
| `this['@super']` / `this['@classLoader']` | superclass / loader |
| `this['field']` | same as `getField(this,'field')` |
| `this['a.b.c']` | same as `this['a']['b']['c']` |

**Functions** — `toString`, `getAddress`, `getType`, `shallowSize`, `retainedSize`, `length`
(array length), `getSize` (collection/map size, or **non-null element count** of an array),
`getByKey`, `getField`.

**⚠️ `retainedSize` vs `shallowSize` on arrays.** `retainedSize` counts only what an object
*dominates*; a shared array (buffer slice, interned table) contributes ~0 and the number looks
wrong. An array's element data lives in its **shallow** size — use `shallowSize` to count array
bytes truthfully. Per-field retained sizes never partition the parent's total; compare each
against it separately.

## 📖 Reading results — the column problem

**The widget tree exposes column 0 only.** Row text gives the class name and `@ 0x…` address;
every number (objects, shallow, retained, percentage) is invisible there. Two ways out:

### 1. Fold into one text column — preferred, least intrusive

```sql
select 'n='   || cast(count(*)               as varchar) ||
       ' tot='|| cast(sum(retainedSize(this)) as varchar) info
  from "io.example.Foo"
```

Column 0 now carries everything. No clipboard, no focus grab, no copy quirks. Make this the
default for scalar and small aggregate results.

### 2. Clipboard — for real multi-column tables

```
eclipse_set_ide_visibility visible:true
eclipse_set_selection      <row paths in the result pane>
eclipse_press_key          Ctrl+C          # assert posted:true AND focusControl Table|Tree
DISPLAY=:0 xclip -o -selection clipboard
```

* Copies **every materialized row** (~25–34) plus the `Total:` line — the selection barely
  matters.
* Assert `posted: true`. A failed post is silent.
* Assert `focusControl` is `Table` or `Tree`. **OQL and Calcite *editor* panes focus a
  `StyledText`**, so Ctrl+C copies the query text, not the result. Query-result panes rendered
  as a plain `Table` are the ones that copy.
* **Aggregate Calcite panes do not copy** — Ctrl+C returns just the class name. Use folding.

### 3. Pane types tell you what happened

* `Table` / `Tree` → real data, readable.
* `Browser` → a **message**: "no result", or a parse error. Content is **unreadable** through
  the widget tree. Diagnose via `eclipse_get_log_entries` (MAT logs `OQL Query does not yield a
  result: …`), or screenshot the pane as a last resort.
* An empty result passed as a query **argument** (e.g. `list_objects <oql>`) raises a **blocking
  modal error dialog** — clear it with `eclipse_dismiss_dialog`.

## 🌲 Tree navigation

* **`expand_row` with `depth: 1`. Only ever 1.** Then re-read, then descend. Higher depths
  exceed the 15 s UI budget on a dominator tree.
* **`childCount` before expansion is a lie** — a collapsed node reports a placeholder `1`
  regardless of real children. Never size `depth` from it.
* **A withdrawn expand still ran.** The timeout describes the lost *reply*, not the work; the
  UI keeps materialising. A "failed" depth-6 call can leave 85 000 rows behind and a scrollbar
  the user notices. Don't retry blindly — re-read first.
* Pagination is **double-click only** (`Total: N of M; K more`) and there is no click tool.
  Don't fight it: narrow with `WHERE` / `ORDER BY … LIMIT` instead.

## 🚫 Hard limits — don't waste turns rediscovering these

* **No click tool.** Every MAT context-menu action reports `command: null` (JFace Actions, not
  commands) — `Merge Shortest Paths to GC Roots`, `Show Retained Set`, `Copy → Save Value To
  File`, column-header sorting, tab close. Unreachable. Use the query-name equivalent on the
  command line.
* **`set_selection` never reaches the selection service** — `handlerSelection` and
  `serviceSelection` stay empty. Selection-dependent enablement cannot be driven.
* **`Text` widget contents are unreadable** (`text: null`) — you cannot confirm what a dialog
  was prefilled with; ask the user.
* **Tabs accumulate and cannot be closed.** Every query adds one. Budget queries accordingly;
  one good query beats six probes.
* `eclipse_run_script` chains **MCP tools only** — it is not a scripting engine. There is no
  back door to MAT's `ISnapshot` API.

## 🙏 Etiquette

The user is sitting at this IDE. Every action is visible and some are disruptive.

* Front-and-focus steals their window — batch the steps, don't ping-pong.
* Never bulk-expand trees.
* Prefer one precise query over exploratory probing; each leaves a permanent tab.
* A job may already be running — `wait_until_quiet` names it. If it is **not** yours, say so
  rather than fighting for the snapshot.

## 🧪 Worked patterns

**Top N objects of a class by retained size** (replaces threshold bisection):

```sql
select getAddress(this) addr, retainedSize(this) rs
  from "io.example.Foo"
 order by 2 desc
 limit 5
```

**Biggest dominators overall** — run MAT query `dominator_tree`, read the top rows, expand
depth 1. This is the fastest route to "what ate the heap".

**Array fill-rate histogram** (`getSize` = non-null count, `length` = capacity):

```sql
select 'decile='|| cast(b           as varchar) ||
       ' arrays='|| cast(count(*)   as varchar) ||
       ' slots=' || cast(sum(len)   as varchar) ||
       ' nulls=' || cast(sum(len-sz) as varchar) info
  from (select length(this['tbl'])                     len,
               getSize(this['tbl'])                    sz,
               (getSize(this['tbl'])*10)/length(this['tbl']) b
          from "io.example.Foo"
         where length(this['tbl']) > 0)
 group by b
 order by b
```

**Cross-check every number.** Two independent routes (histogram vs. Calcite sum, bisection vs.
`ORDER BY`) agreeing is proof; one number alone is a claim.
