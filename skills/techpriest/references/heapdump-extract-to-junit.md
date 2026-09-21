# ⚱️ Rite of Exhumation — heap dump object → JUnit fixture

Companion to `heapdump-mat.md`. That rite **answers questions** about a dump through MAT's UI. This one
**takes an object out** of it: a live instance, rebuilt in a plain JVM, interrogable with ordinary Java and
a debugger. Read it when "why is this thing 56 MB" stops being answerable by queries and starts needing
experiments.

## 🔧 With `mat_extract` present, phases 3 and 4 are already built

`mat_extract` (see `heapdump-mat.md`) is the generic half of this rite as a tool: root address plus
filters, a breadth-first walk over field and array references that never enters a class or a class
loader, the off-heap buffer check, limits that say which one stopped the walk, and a per-class report.
**It defaults to a dry run**, which answers "what would travel, and how big" before anything is written.

So reach for it first, and write a bundle only when the walk itself has to be type-aware. What it does
*not* remove is Phase 5: the reader still lives in the repository that owns the classes, because a generic
restorer is the half that fails silently.

## 🧭 Decide first: query or exhume

| you need | instrument |
|---|---|
| sizes, counts, distributions, who-retains-what | `heapdump-mat.md` — Calcite SQL, `dominator_tree`, `list_objects` |
| one array's bytes | MAT UI `Copy → Save Value To File` — **one array at a time**, and it is a JFace action, so an agent cannot click it |
| the object graph, to run real code against | **this rite** |

Do not exhume what a query can answer. Exhumation costs a bundle build and a restore reader; it pays off only
when you will run many experiments against the same data.

## 🪝 Why a MAT query bundle (and not a script)

* MAT's parser resolves its extensions through the Eclipse extension registry, so `SnapshotFactory` from a plain
  `java -cp` program is a dead end. OSGi or nothing.
* The IDE **already holds the dump open with indices built** — a query runs against that snapshot, no second parse.
* `eclipse_run_script` chains MCP tools only; it is not a scripting engine. **A custom query bundle is the back
  door to `ISnapshot`** that the MAT rite correctly says does not exist at the MCP layer — you get the full API,
  in-process, against the open snapshot.
* Neither OQL nor Calcite can stream megabytes to disk, and MAT's CSV/HTML exports are unreachable JFace actions.

## 🔩 Phase 1 — locate and measure

Per `heapdump-mat.md`: find the instance and its address, then read the size story before writing any code.

* `list_objects 0x<address>` is the focusless drill-down; column 0 carries field names, types and addresses.
* **`shallowSize` for array bytes, not `retainedSize`.** A shared array dominates nothing and contributes ~0 to
  retained; its element data lives in shallow size.
* Note which fields are big. That list becomes the extract format.

## 🧪 Phase 2 — triage every field

Three buckets. Getting this wrong is what makes an extract wrong rather than merely large.

| bucket | examples | action |
|---|---|---|
| 🟢 **data** | primitive arrays, `int[][]` payloads, dictionary offsets + bytes, scalars | extract verbatim |
| 🟡 **rebuildable cache** | lazily built reverse-lookup maps, materialized entry arrays | **skip** — they regenerate on first use, and they are often the biggest thing in the dump |
| 🔴 **unrecoverable** | direct/mapped `ByteBuffer`s, native handles, sockets, threads, locks, lambdas, per-request context | cannot travel — **fail loudly** or document the substitute |

A 🟡 cache routinely outweighs the data it indexes. Skipping it shrinks the extract and changes nothing about
correctness — so measure the buckets before believing the retained figure.

## 🏗️ Phase 3 — the extractor bundle

Shape that works, in ~200 lines:

```java
@CommandName("exhume")
public class ExhumeQuery implements IQuery
{
  @Argument public ISnapshot snapshot;          // MAT injects the open snapshot
  public IResult execute(IProgressListener l) { … return new TextResult(report); }
}
```

* `META-INF/MANIFEST.MF` with `Require-Bundle: org.eclipse.mat.api, org.eclipse.mat.report`, and `plugin.xml`
  contributing to extension point **`org.eclipse.mat.report.query`**.
* Compile against the standalone MAT install (`javac -cp '/active/mat/plugins/*'`), which must be the **same MAT
  version** the IDE runs, then `jar cfm`.
* Install with `eclipse_install_bundle` (dry run, then `dryRun: false`). Run with
  `eclipse_run_workbench_command` → `org.eclipse.mat.ui.actions.executeInspection`, `commandName: <your command>`.

### 🎛️ Parameters: a job file, not query arguments

Put `address=0x…` and `out=…` in a properties file the query reads on every run.

* **A query with no arguments beyond `ISnapshot` executes immediately — no wizard, no dialog.** (The wizard dance
  in `heapdump-mat.md` applies to queries that *declare* arguments; this sidesteps it entirely.)
* Retargeting to another address is then a one-line file edit — no rebuild, no reinstall.

### ⚠️ Bundle traps, each paid for in lost turns

* **Bump `Bundle-Version` before every reinstall.** The framework rejects byte-identical content outright.
* **A hot update leaves the old revision's classloader wired into the registry** — `Invalid class loader from a
  refreshed bundle`, `NoClassDefFoundError: ISnapshot`, and a popup for the user. The run already in flight
  completes; the next may need a higher version or an IDE restart. Do the reporting work *before* iterating.
* **A hot install is invisible to p2** and does not survive a restart. Correct for a tool; reinstall after restart.
* `executeInspection` takes an inspection **name**, never a command line with arguments.
* `openSampleHeapDump` cannot open an arbitrary path — it resolves inside a bundle and throws NPE.
* `eclipse_open` accepts workspace paths only; a dump outside the workspace is opened by the human, or linked in.
* `outcome: "success"` means a handler returned. **Verify by the artifact** — the output file's size and mtime.

## 📐 Phase 4 — the wire format

Flat, positional, `DataOutputStream`. No JSON, no Java serialization (the dumped classes' `serialVersionUID`s are
irrelevant — you are rebuilding through constructors, not deserializing).

* Magic string first; the reader rejects anything else.
* Counts before every array; `-1` for null so null and empty stay distinct.
* Document the layout **in the writer's javadoc** and name the reader class there. The two must move together;
  nothing else enforces it.
* Write a **size report next to the binary** (`out + ".txt"`): retained per component, extracted bytes vs backing
  array, skipped caches. The report is the actual diagnostic — the binary is just fuel.

### 🩸 Rules the extractor obeys

* **Fail loudly, never silently**: wrong class at the address, unexpected null, off-heap buffer → throw. A silently
  wrong extract poisons every experiment that follows.
* **Extract the window, not the backing array.** A `ByteBuffer` view carries `offset`, `position`, `limit`; read
  `[offset+position, offset+limit)` from `hb`. Extracting the whole `hb` is both wrong and huge.
* **`hb == null` means off-heap** — a direct or mapped buffer whose bytes are *not in the dump at all*. Throw, and
  say the capacity and address so the human can decide.
* Huge arrays: `getValueArray(offset, length)` in slices, never whole-array reads you do not need.
* `int[][]` is an `IObjectArray` of addresses → `mapAddressToId` + `getObject` per element. Fine for ~10⁴ rows;
  budget for it at 10⁶.
* fastutil-derived lists (`IntArrayList` and friends) are `a` + `size` — **write `size` elements, not `a.length`**.

## 🧫 Phase 5 — the restore side, in the repo's test sources

The reader belongs with the classes it rebuilds, so it compiles against their real constructors:

* **Rebuild through public constructors**, not reflection. If a constructor is missing, that is a design signal —
  do not reach for `Unsafe` to dodge it.
* Restore what travelled; **document what did not** (headers, request context, bundle references) in the class
  javadoc. Anything driven by the missing state is not evidence about the dumped run.
* Default the extract path through a system property so a second instance needs no edit.
* Guard the test with `Assumptions.assumeTrue(Files.exists(...))` — skipped without the extract, so CI stays green
  while the fixture stays in the repo.
* The test's job is to restore and **log the size breakdown**; assertions stay minimal. It is a harness, not a
  regression test.

## ✅ Phase 6 — verify before believing

Cross-check the restored object against the dump report, the same way the MAT rite demands two independent routes:
sizes computed from the restored object must match the extracted byte counts, row counts must match the source
lengths. A skipped test that *looks* green is the failure mode to watch for — confirm `Skipped: 0`.

## ♻️ Generalising — and where to stop

Generic already: job-file parameterization, field/array reads over `IObject`, the heap-vs-direct check, size
reporting, and the build → install → run loop. Type-specific: the field walk and its matching reader.

A **subtree** extractor is tempting and half-easy: traversal is cheap (`getOutboundReferentIds`, or MAT's retained
set) and a self-describing encoding is mechanical. The hard half is the way back — a generic restorer means
`Unsafe.allocateInstance` plus reflective field writes, which breaks on direct buffers, native handles, lambdas,
and any class whose fields drifted since the dump. **That last one fails silently.** Recommended shape: one generic
traversal with filters, typed readers per restored type.
