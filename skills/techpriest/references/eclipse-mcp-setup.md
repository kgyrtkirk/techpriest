# 🏗️ Eclipse MCP + MAT — Setup Guide

Read when the `eclipse` MCP server is **missing, broken, or the user is stuck**. For using it,
read `heapdump-mat.md`.

URLs below come from a working installation's p2 registry — not guessed.

## 🔷 Full IDE, never standalone MAT

Standalone MAT is a stripped RCP app. The server needs workspace, JDT, the command and handler
framework, editors, views, preferences. Install MAT **into the Eclipse IDE for Java
Developers**. Bonus: dump and project source share a workspace — pull a structure out of the
heap, reinstate it as a test fixture, continue in code.

## 1️⃣ Eclipse IDE for Java Developers

Written against `eclipse-2026-09-R-java` (platform 4.41).

* Download: <https://www.eclipse.org/downloads/packages/>
* p2: `https://download.eclipse.org/technology/epp/packages/latest/`

Unpack somewhere writable. MAT writes `.index` files next to the `.hprof` — that directory needs
space and write permission.

### ⚙️ `-Xmx` — usually unnecessary

Stock EPP ships **no `-Xmx`**, and JDK 25 defaults to **25% of physical RAM**. On a 128 GB box
that is ~30 GB, which parses a 50 GB dump without touching `eclipse.ini`.

Set it only when 25% of RAM is too little (laptop, container limit). Then in `eclipse.ini`, after
`-vmargs`:

```
-Xmx16g
```

Symptom of too little: parse fails or never finishes, with no clear error.

## 2️⃣ Eclipse MCP Server (vogella)

Help → Install New Software… → Add…

Location: `https://vogellacompany.github.io/eclipse-mcp-server/`
Feature: **Eclipse MCP Server** (`com.vogella.eclipse.mcp.feature.feature.group`). Restart.

## 3️⃣ Memory Analyzer

Location: `https://download.eclipse.org/mat/latest/update-site/`
Features: **Memory Analyzer** (required), **Memory Analyzer (Charts)** (optional).

Pinned: `https://download.eclipse.org/mat/1.17.0/update-site/` — snapshots:
`https://download.eclipse.org/mat/snapshots/update-site/`

## 4️⃣ Calcite SQL plug-in — install it

MAT's OQL has **no `ORDER BY`, `GROUP BY` or aggregates**. Without this an agent is reduced to
bisecting thresholds by hand.

Location: `https://vlsi.github.io/mat-calcite-plugin-update-site/stable/`
Installs `MatCalcitePlugin` + wrapped `calcite-core`/`avatica`.
Home: <https://github.com/vlsi/mat-calcite-plugin>

## 5️⃣ Auspex Mortis — MAT through the API, not the widgets

Location: `https://kgyrtkirk.github.io/auspex-mortis/`
Feature: **Auspex Mortis** (`hu.rxd.auspex.mortis.feature.feature.group`). Restart.
Home: <https://github.com/kgyrtkirk/auspex-mortis>

⚠️ The one URL here not yet read back from an installation: it goes live with the repository's
first GitHub Pages deploy. Until then, build it and install from the local
`file:/…/update-site/hu.rxd.auspex.mortis.repository/target/repository`.

Adds `mat_query`, `mat_object` and `mat_extract` to the MCP server. Needs 2️⃣ and 3️⃣; uses 4️⃣
when present, for the Calcite editor pane. With it installed, `heapdump-mat.md`'s widget route
is the fallback. **Its tools register only at IDE startup** — the restart is not optional.

## 6️⃣ Enable and wire

**Preferences → General → MCP Server** — enable; note **port** and **auth token**; the **call
timeout** lives here too.

```bash
claude mcp add --transport http eclipse http://127.0.0.1:<port>/mcp \
  --header "Authorization: Bearer <token-from-preferences>"
```

Lands in `~/.claude.json` under the project:

```json
"mcpServers": {
  "eclipse": {
    "type": "http",
    "url": "http://127.0.0.1:8642/mcp",
    "headers": { "Authorization": "Bearer <token>" }
  }
}
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
* **Calls die after a few seconds** → raise the call timeout in Preferences. `run_script` has its
  own ~30 s budget — poll `eclipse_wait_until_quiet` separately instead.
* **Parse fails on a large dump** → `-Xmx` too small (see 1️⃣), or the dump's directory is full or
  read-only.
* **Query seems stuck** → `eclipse_wait_until_quiet` names the job. It may be the user's own
  query from the Calcite tab holding the snapshot.
* **`mat_*` tools missing after an install** → the IDE was not restarted; tools register at
  startup only.
* **Old behaviour after upgrading Auspex Mortis** → two bundles contribute the same tool name.
  `McpToolRegistry` keeps the first and only logs *"Duplicate MCP tool name … ignoring it"*.
  Uninstall the old feature (dry run first), then restart.
