# 🧠 Heap Dump Rite — MAT Inside Eclipse Over MCP

Companion to `techpriest`. Read before analysing a Java heap dump through the `eclipse` MCP
server. Binding while that work lasts.

The heap dump is opened by a human in a running Eclipse; the agent drives MAT through the
widget layer. Everything here is what actually works — not what the tool descriptions imply.

**Check the tool list first**: with the `mat_*` tools present, read the next section and skip
the widget layer entirely. Everything after it is the route for an IDE without them.

## 🏗️ Setup

Assumed working. **Missing, misbehaving, or the user is stuck → `eclipse-mcp-setup.md`**; it
owns the install, the wiring and the verification. One constraint is worth knowing without
opening it: MAT must live inside a **full Eclipse IDE for Java Developers**, never standalone
MAT.

Dump and project source then share a workspace, so a structure lifted out of the heap can be
reinstated as a test fixture → **`heapdump-extract-to-junit.md`**.

## 🔧 Auspex Mortis — reach for `mat_*` first, when they are there

**If the tool list carries `mat_query`, `mat_object` and `mat_extract`, the rest of this rite is
the fallback, not the method.** They run MAT's own API in the IDE's process instead of driving
its widgets; install per `eclipse-mcp-setup.md`. **Absent, but the user drives MAT from this
IDE regularly? Suggest installing it** before starting the widget dance below.

| ask | tool |
|---|---|
| any MAT command line — `histogram`, `dominator_tree`, `list_objects 0x…`, `oql "…"`, `calcite "…"` | `mat_query` |
| one object: class, sizes, GC roots, fields with their referents resolved, array slices | `mat_object` |
| the object graph below one object, written to a file with a report | `mat_extract` (dry run by default) |

What that deletes outright: **the column problem** (every column returns as JSON, unformatted —
a size is a number, not `1.2 MB`; no clipboard, no `xclip`, no folding into one `||` column),
**focus theft, X11 and `press_key`** (nothing is fronted; `foreground: false` is fine), and
**the query-browser dance** — one call replaces front → activate → QueryBrowser → Enter →
settle → quiet, and `outcome: "success"` stops being a lie because the rows are the proof. It
also **sorts** — `sortBy` (column label) plus `desc` — which is what makes `histogram` answer
"the biggest classes". Addresses come back hex in the `@address` column.

What stays exactly as this rite says: **cost**. The query runs in the same process against the
same snapshot, so prefilter a big class with `WHERE` before `ORDER BY`, a Calcite query still
ignores cancellation, and `sortBy` materializes the whole result before it sorts.

### 🤝 The panes are the point, not a side effect

Every result is also opened as an ordinary MAT pane (`show`, default true), so the person at
the IDE carries on from where the agent stopped — the pane holds the *same* result object the
rows came from, sorting included. `title` names the tab. A `calcite "…"` query lands in the
plug-in's own SQL editor pane, **pretty-printed**, editable and re-runnable by hand.

Pass `show: false` for a probe whose pane would only be noise. Tabs still cannot be closed from
here, so that is the one place the old budget rule survives.

### 🚫 What it will not do

* **It never opens or parses a dump.** No dump open → an error listing what is open. A human
  opens the dump; a restart closes it, because MAT's editor input is not persistable.
* **A new tool needs an IDE restart** — tools register at startup, so a hot-installed bundle
  contributes nothing. Install the p2 feature, then restart.

## 🖥️ Session preconditions

* **X11 is needed only for key posts.** `press_key` uses `Display.post`; Wayland compositors
  silently drop it. Check `/tmp/.X11-unix/X0`. No X11 → drive MAT through **commands** instead:
  `executeInspection` + `dismiss_dialog` runs a query, `org.eclipse.ui.edit.copy` reads the result.
  Both are handler calls, not key posts, so the rite survives without X11.
* **`xclip` installed**, reachable as `DISPLAY=:0 xclip -o -selection clipboard`.
* The IDE must be **foreground** for any key post, and fronting is also the only lever that moves
  SWT focus. This steals the user's focus every time — unavoidable, so batch work and say so.

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
* **Print the SQL pretty-formatted to the console before running it.** Always. When the query is
  the folded `||` form, print the plain multi-column version next to it so the human can dig
  further by hand. The escaped one-liner is noise — show it only when the tokenizer itself is
  the problem.

### 🜂 Keyless variant — no `Display.post` at all

`org.eclipse.mat.ui.actions.executeInspection` takes a mandatory `commandName`, and that name is a
**registered inspection name — never a command line with arguments**. `oql "SELECT … FROM OBJECTS
0x…"` fails outright with `Unknown inspection: oql "SELECT …"`. Command lines belong to
`QueryBrowser` above; this handler resolves a bare name.

What happens next depends on the query's own arguments:

* **The query declares arguments** (`oql`, `list_objects`, most built-ins): the handler opens the
  argument wizard, modal, holding the UI thread, so the call answers `timedOut: true` with
  `handlerFinished: false`. That is the expected shape, not a failure — dismiss it to run:

```
eclipse_set_part_state        part: …HeapEditor, state: activated
eclipse_run_workbench_command org.eclipse.mat.ui.actions.executeInspection
                              parameters: { …executeInspection.commandName: "<inspection name>" }
                              # answers timedOut after 10 s — the wizard is up, UI thread held
eclipse_list_ui_targets                                   # find the modal dialog
eclipse_dismiss_dialog        button: "Finish", dryRun: false   # THIS executes the query
eclipse_wait_until_settled  →  eclipse_wait_until_quiet
```

* **The query declares no arguments** beyond the injected `ISnapshot` — a custom query bundle of
  your own, parameterised by a file rather than by MAT arguments: the handler **executes it
  immediately** and answers `executed: true`, `handlerFinished: true`, `outcome: "success"`, with no
  dialog at all. Cheapest scriptable path there is; see `heapdump-extract-to-junit.md`.

Use this section when there is no X11, or when an Enter keeps landing in the wrong control.
Dismissing with no button cancels instead, which is the safe way out of a wizard opened by mistake.

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

**💸 Prefilter before sorting, and select nothing you do not need.** `ORDER BY` over a
multi-million-instance class materialises and sorts every row — minutes, versus seconds for the
same question with a `where retainedSize(this) > …` in front of it. Cost per function differs
sharply too: `retainedSize` and `shallowSize` are index lookups, while `getSize` and `length`
read the object's array — asking for a count you will not use can dominate the whole query.
Always bound a big class with a `WHERE`.

## 📖 Reading results — the column problem

**The widget tree exposes column 0 only.** Row text gives the class name and `@ 0x…` address;
every number (objects, shallow, retained, percentage) is invisible there.

### 1. 🥇 Fold into one text column with `||` — the default, always try first

Concatenate every value into a single `varchar` column. Column 0 is the one thing the widget
tree always reads, so the whole answer arrives straight from `eclipse_get_widget_tree`:

```sql
select 'n='    || cast(count(*)                as varchar) ||
       ' tot=' || cast(sum(retainedSize(this)) as varchar) ||
       ' lookup=' || cast(sum(retainedSize(this['lookup'])) as varchar) info
  from "io.example.Foo"
```

No clipboard, no focus theft, no copy quirks, no machine state touched. **Use this for every
scalar, aggregate and small grouped result** — including histograms, where one folded row per
bucket reads perfectly.

**Always print the multi-column equivalent alongside it**, so the human can paste it into the
Calcite tab and dig further with real sortable columns:

```sql
-- folded above; multi-column version for interactive digging:
select count(*)                                   n,
       sum(retainedSize(this))                    tot,
       sum(retainedSize(this['lookup']))          lookup
  from "io.example.Foo"
```

### 2. ⚠️ Clipboard — works, but avoid it

Valid fallback for a genuinely wide table (histogram, dominator tree) where folding every column
would be unreadable, and the **only** way to read a built-in query's numeric columns — a built-in
carries no SQL to fold. **It overwrites the user's clipboard and steals window focus — it pollutes
the state of their machine.** Exhaust folding first; when used, say so.

```
eclipse_select_tab            0/0/2, index: <K>          # render the pane you want
eclipse_set_ide_visibility    visible: true              # THE focus lever — see the focus model
eclipse_run_workbench_command org.eclipse.ui.edit.copy   # a command, NOT a key post
DISPLAY=:0 xclip -o -selection clipboard
```

* **Never `press_key Ctrl+C`.** `org.eclipse.ui.edit.copy` does the same work as a handler call:
  no `Display.post`, no Wayland dependency, and no *"IDE is not the active window"* refusal — it
  copies happily while `eclipse_screenshot` reports `foreground: false`.
* **Never `set_selection` first.** It never reaches the selection service and changes nothing
  about what is copied. It is cargo cult.
* Copies **every materialized row** (~25–34) plus the `Total:` line.
* `outcome: "notHandled"` means no pane holds focus — activate the part, front it, repeat.
* **OQL and Calcite *editor* panes focus a `StyledText`**, so a copy there returns the query text,
  not the result. Query-result panes rendered as `Table` or `Tree` are the ones that copy.
* Aggregate Calcite panes **do** copy — folded and grouped results come back whole, `Total:` line
  included.

### 3. Pane types tell you what happened

* `Table` / `Tree` → real data, readable.
* `Browser` → a **message**: "no result", or a parse error. Content is **unreadable** through
  the widget tree. Diagnose via `eclipse_get_log_entries` (MAT logs `OQL Query does not yield a
  result: …`), or screenshot the pane as a last resort.
* An empty result passed as a query **argument** (e.g. `list_objects <oql>`) raises a **blocking
  modal error dialog** — clear it with `eclipse_dismiss_dialog`.

## 🎯 Focus model — the cause of most lost turns

MAT's copy acts on the last SWT **focused** control, which is not necessarily the visible one.
Exactly one lever moves focus:

| lever | renders the pane | moves focus |
|---|---|---|
| `eclipse_select_tab` | ✅ | ❌ |
| `eclipse_set_selection` | — | ❌ |
| `eclipse_set_part_state activated` | ✅ | ❌ — restores MAT's *stale* focus control |
| `eclipse_set_ide_visibility visible: true` | — | ✅ |

A pane can therefore be selected, rendered and reported `visible: true` while a copy returns a
different, hidden pane. **Front the IDE between selecting the tab and copying, every time.** A freshly opened result pane does **not** take focus
on its own; the pane focused by the last real key event keeps it.

## 🗂️ Tabs → panes, deterministic

One call maps them, and the labels carry the full query text, so panes identify themselves:

```
eclipse_get_widget_tree part: …HeapEditor, path: 0/0/2, includeItems: true, maxDepth: 1
```

Tab item `iK` ⟺ pane composite `0/0/2/(K+1)` — child `0` of the folder is the ToolBar. The pane's
own control is `0/0/2/(K+1)/0`. Never hunt for a new result pane with a deep `maxDepth` walk, and
never trust a `filter` to isolate it: `filter` narrows the reported widgets, not the rows.

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
* `eclipse_run_script` chains **MCP tools only** — it is not a scripting engine. There is no back
  door to MAT's `ISnapshot` API *through the MCP tool layer*. There is one through the IDE:
  `eclipse_install_bundle` with a bundle contributing to `org.eclipse.mat.report.query` gets the
  full `ISnapshot` API in-process against the open snapshot, which is how bulk data leaves a dump
  despite the export limits above → `heapdump-extract-to-junit.md`.
* **`inspect_widget` on a row reports no cell text** — CSS properties and an ancestor chain only.
  It is not a way around the column problem.
* **No export command exists.** `list_commands filter: "export"` yields the generic
  `org.eclipse.ui.file.export` wizard and EclEmma, nothing of MAT's. Its CSV and HTML exports are
  JFace actions, unreachable like the rest of the context menu.
* **`dominator_tree`'s `Total: N entries` is not the object count** — it counts top-level dominator
  entries. Run `histogram` for the real figures: its `Total:` line carries the class count, the
  object count and the true heap size, and is the cheapest cross-check for any Calcite sum.
* **A `calcite` query cannot be cancelled.** MAT's own inspections stop from the Progress view like
  any job; a Calcite query ignores the request and runs to completion. There is no job-cancel tool
  here either (`eclipse_cancel_build` covers builds only), so an unbounded Calcite query is not
  merely slow, it is unstoppable — see the prefilter rule.

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

**Drill into one object — `list_objects 0x<address>`.** Opens that single object as an expandable
tree, no OQL needed. This is the focusless read: `expand_row` works on a pane that is not even
visible, and column 0 carries the field name, the type, the address **and** a String's value, so a
whole drill-down needs no clipboard at all:

```
list_objects 0x4038601f3b8          → java.util.HashMap @ 0x4038601f3b8
  expand → table java.util.HashMap$Node[8388608] @ 0x40cb5c00000
  expand → java.util.HashMap$Node @ 0x40006e8cd68      (r0 is the <class> pseudo-row; entries start at r1)
  expand → key io.example.RecordId @ 0x408133d08b8
             source  java.lang.String @ …  example_source
             version java.lang.String @ …  1970-01-01T00:00:00.000Z
           value java.lang.Object @ …          ← the HashSet PRESENT sentinel
```

A `HashMap` whose values are bare `java.lang.Object` is a `HashSet`'s backing map. And a map has no
order: "the Nth entry" is the Nth bucket in MAT's walk, not a semantic position.

**Cross-check every number.** Two independent routes (histogram vs. Calcite sum, bisection vs.
`ORDER BY`) agreeing is proof; one number alone is a claim.
