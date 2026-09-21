# 🔌 Eclipse MCP + MAT — Setup Guide

Read when the `eclipse` MCP server is **missing, broken, or the user is stuck**. For using it,
read `heapdump-mat.md`.

URLs below come from a working installation's p2 registry — not guessed.

## 🔷 Full IDE, never standalone MAT

Standalone MAT is a stripped RCP app. The server needs workspace, JDT, the command and handler
framework, editors, views, preferences. Install MAT **into the Eclipse IDE for Java
Developers**. Bonus: dump and project source share a workspace — pull a structure out of the
heap, reinstate it as a test fixture, continue in code.

## 📥 Install — Help → Install New Software…, restart after each

| # | component | location | feature |
|---|---|---|---|
| 1️⃣ | Eclipse IDE for Java Developers | `https://download.eclipse.org/technology/epp/packages/latest/` | — |
| 2️⃣ | Eclipse MCP Server (vogella) | `https://vogellacompany.github.io/eclipse-mcp-server/` | `com.vogella.eclipse.mcp.feature.feature.group` |
| 3️⃣ | Memory Analyzer | `https://download.eclipse.org/mat/latest/update-site/` | Memory Analyzer (Charts optional) |
| 4️⃣ | Calcite SQL plug-in | `https://vlsi.github.io/mat-calcite-plugin-update-site/stable/` | `MatCalcitePlugin` |
| 5️⃣ | Auspex Mortis | `https://kgyrtkirk.github.io/auspex-mortis/` | `hu.rxd.auspex.mortis.feature.feature.group` |

* **4️⃣ is not optional in practice** — MAT's OQL has no `ORDER BY`, `GROUP BY` or aggregates;
  without it an agent is reduced to bisecting thresholds by hand.
  Home: <https://github.com/vlsi/mat-calcite-plugin>.
* **5️⃣ adds `mat_query`, `mat_object`, `mat_extract`** — MAT's API in-process instead of the
  widget layer. Needs 2️⃣ and 3️⃣, uses 4️⃣ when present. Its tools register at IDE startup only,
  so the restart is not optional. Home: <https://github.com/kgyrtkirk/auspex-mortis>.
* **Unpack somewhere writable, with space** — MAT writes `.index` files next to the `.hprof`.
* **`-Xmx` only when the default is too little** (laptop, container limit): `-Xmx16g` in
  `eclipse.ini` after `-vmargs`. Symptom: the parse fails or never finishes, with no clear error.

## 6️⃣ Enable and wire

**Preferences → General → MCP Server** — enable; note **port** and **auth token**; the **call
timeout** lives here too.

```bash
claude mcp add --transport http eclipse http://127.0.0.1:<port>/mcp \
  --header "Authorization: Bearer <token-from-preferences>"
```

🔒 Token is a secret. Read from Preferences; never into docs, commits or chat.

## 7️⃣ Open the dump

`File → Open File…` → `.hprof` / `.hprof.gz`. Heap editor part id:
`org.eclipse.mat.ui.editors.HeapEditor`.

## ✅ Verify

1. `eclipse_get_installation` → `runtime.agrees: true`; filter `memory`, `calcite`, `mcp`,
   `auspex` to confirm all four features.
2. The tool list carries `mat_query` → Auspex Mortis registered. `mat_query histogram` with no
   dump open answers *"No heap dump is open in the IDE"* — that refusal is the proof it runs.
3. `eclipse_list_editors` → the dump, id `org.eclipse.mat.ui.editors.HeapEditor`.

1 and 2 passing but 3 empty → install is fine, the user just has not opened the dump.

## 🩺 Troubleshooting

* **Keys never arrive** → Wayland. `press_key` uses `Display.post`, which Wayland drops. Check
  `/tmp/.X11-unix/X0`. No X11 → running MAT queries by keystroke will not work.
* **`runtime.agrees: false`** → p2 thinks something is installed that never loaded; `mismatches`
  names it. Restart, reinstall that feature.
* **"pass ids explicitly" / looks like many clients** → a session ends on HTTP DELETE or 60 s of
  silence. One session per client, or pass ids.
* **Calls die after a few seconds** → raise the call timeout in Preferences.
* **Parse fails on a large dump** → `-Xmx` too small (see above), or the dump's directory is full
  or read-only.
* **Query seems stuck** → `eclipse_wait_until_quiet` names the job. It may be the user's own
  query holding the snapshot.
* **`mat_*` tools missing after an install** → the IDE was not restarted; tools register at
  startup only.
* **Old behaviour after upgrading Auspex Mortis** → two bundles contribute the same tool name;
  the registry keeps the first and logs a duplicate warning. Uninstall the old feature (dry run
  first), then restart.
