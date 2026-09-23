# 🖥️ Eclipse Rite — Compile & Problem Markers

Companion to `techpriest`, child of `tooling.md`. Read when the `eclipse` MCP server is connected
and the work is compiling, hunting problem markers, or running tests.

## 🧭 What To Reach For

| ask | route |
|---|---|
| compile errors & warnings | `eclipse_build` → `eclipse_get_problems`. Incremental, two modules ≈ 200 ms — orders of magnitude faster than a maven static-check pass. |
| only **your** markers | `pathPrefix` + `types: ["org.eclipse.jdt.core.problem"]`. Unscoped, a mature workspace buries the change under thousands of pre-existing warnings. |
| run tests | maven — **never the IDE**. §🚫 below. |

* 🗂️ **Closed project** — `eclipse_list_projects` first; a build naming a closed project errors
  out. `eclipse_set_project_state` opens it, but **ask** — it may be closed deliberately.
* ⏳ `eclipse_get_problems` **never starts a build** — markers stay stale until `eclipse_build` runs.
* ⚖️ Judge a build by `errors`, not by the warning count; `builderFailures` catches builders that
  threw without leaving a marker.

## 🚫 Tests — Both IDE Routes Are Dead

Reached only from the table above. Run tests with maven; read on only to stop re-deriving this.

**`eclipse_run_tests` fails on JUnit 5:**

> Cannot find 'org.junit.platform.commons.annotation.Testable' on project build path.

* **why** — it synthesises a launch config, so JDT must *detect* the test kind, and detection
  fails. A human-saved config pins `org.eclipse.jdt.junit.TEST_KIND` and never detects — which is
  why *Run As → JUnit Test* works for the user and this tool never will.
* **don't re-derive** — classpath, `Testable` present in the jar, test-scoped entries, fresh-open
  race: four hypotheses, all checked, all dead.
* **server-side fix**, if the server is yours: pin `TEST_KIND` explicitly instead of detecting.

**`eclipse_debug_launch {configuration: "<name>", mode: "run"}` is not the way round it.** It runs
a human-saved config and returns an **exit code only** — no counts, no assertion text, no stack
trace, and `eclipse_get_test_results` cannot see it. Worthless for diagnosing a failure, and a new
test class has no saved config. `project` + `mainType` is no substitute: that builds a Java
Application config, and a test class has no `main`.
