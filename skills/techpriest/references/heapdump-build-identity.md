# 🏷️ Heap Dump Rite — Build & JVM Identity

Child of `heapdump-mat.md`. Read when the question is **which build, which JVM flags, which node**
produced the dump. Verified on Auspex Mortis (`mat_query`, `mat_object`); widget route unverified —
same SQL, object walks via `list_objects` (parent §🔍). Every probe here → `show: false`.

## 🧭 Cheapest first

| want | source |
|---|---|
| JVM version, dump time, used heap | `heap_dump_overview` |
| main class + args, classpath, arch | `system_properties` |
| `-Xmx`, `-XX:*`, `--add-opens` | §⚙️ JVM flags |
| per-jar version | §📦 Per-jar versions |
| git hash, no product stamp | §🕵️ 40-hex scan |
| 🐉 Druid process | §🐉 Druid — **before** the 40-hex scan |

## 📜 Overview & system properties

* `heap_dump_overview` → JVM version, dump time, file, used heap, object / class / loader counts.
  Used heap counts **reachable objects only** — hold it against `-Xmx`.
* `system_properties` → one row per key:
  * `sun.java.command` → main class + args. **Never JVM flags.**
  * `java.class.path` → deploy dir often names distribution + build timestamp. Value is cut with
    `...`; the prefix is what counts.
  * `os.arch`, `java.runtime.version`, `user.timezone`, vendor keys.
  * Third-party hashes (`jetty.git.hash`) land here too — not yours.
  * **`-Xmx` is never here** → §⚙️.

## ⚙️ JVM flags — `VMManagementImpl.vmArgs`

JDK caches it lazily: present **only if** something called `RuntimeMXBean.getInputArguments()`
(JMX, a metrics monitor). Probe:

```sql
select getAddress(this)        addr,
       getSize(this['vmArgs']) nArgs
  from "sun.management.VMManagementImpl"
```

No row / null `nArgs` → not cached; no other source known → ask the user for the launch config.
Cached → `mat_object` down the chain:

```
VMManagementImpl
  .vmArgs  java.util.Collections$UnmodifiableRandomAccessList
  .list    java.util.Arrays$ArrayList
  .a       java.lang.String[]          ← arrayLength: nArgs
```

Each element's `refText` = one flag, command-line order.

**⚖️ Verdict** — used heap ≈ `-Xmx` → heap full of live data. Plus `+HeapDumpOnOutOfMemoryError`
→ *consistent with* an OOM-triggered dump. The dump cannot prove its trigger: never say "was".

## 📦 Per-jar versions — `Package$VersionInfo`

```sql
select toString(this['implTitle'])   title,
       toString(this['implVersion']) implVersion,
       count(*)                      n
  from java.lang."Package$VersionInfo"
 group by toString(this['implTitle']), toString(this['implVersion'])
```

Milliseconds. Each jar's `Implementation-Version` — version only, **no hash**. The manifests
themselves are gone: a running server's dump held **zero** `java.util.jar.Manifest`. Don't hunt there.

## 🕵️ 40-hex scan — any stray git hash

```sql
select toString(this) s,
       count(*)       n
  from "java.lang.String"
 where length(this['value']) = 40
   and toString(this) similar to '[0-9a-f]{40}'
 group by toString(this)
```

Seconds per ~1M Strings. `length(this['value']) = 40` is the parent's §💸 prefilter — spares the
regex (compact Latin-1 String: one byte per char). Then the owners:

```sql
select getAddress(this) addr,
       toString(this)   s
  from "java.lang.String"
 where length(this['value']) = 40
   and toString(this) in ('<hash1>', '<hash2>')
```

→ `mat_object` with `inbound: true, fields: false, outbound: false` per address. Owner is a
`java.lang.Class` → static field: `mat_object` on that class lists its constants (ZooKeeper
`Version`: `REVISION_HASH`, `BUILD_DATE`). Expect third-party hashes and all-zero placeholders —
the owner says whose.

## 🔗 Tie the hash to local source

```
git cat-file -t <rev>                             # 'commit' → present locally
git log -1 --format='%h %ad %an%n%s' --date=short <rev>
git merge-base --is-ancestor <rev> HEAD           # exit 0 → branch contains it
git grep -n '<class declaration>' <rev> -- <path> # dump's class shapes exist at <rev>?
```

Class in the dump, absent from the working tree → working tree ≠ build. Read source at `<rev>`
(worktree) before reasoning about the dump from code.

## 🐉 Druid — `DruidNode`

Gate: `sun.java.command` = `org.apache.druid.cli.Main server <type>`. Every `DruidNode` carries
`version` + `buildRevision` (git commit id):

```sql
select getAddress(this)                addr,
       toString(this['serviceName'])   svc,
       toString(this['host'])          host,
       this['plaintextPort']           port,
       toString(this['version'])       ver,
       toString(this['buildRevision']) rev
  from "org.apache.druid.server.DruidNode"
```

Milliseconds. **⚠️ Several rows** — own node plus every discovered peer. Own = the row whose
`serviceName` matches `<type>` (`server historical` → `druid/historical…`). Peers come free:
differing `rev` → mixed-version cluster. Hash in hand → §🔗.
