# 🏗️ Refactoring Rite — Broad Changes

Companion to `techpriest`. Read before any task spanning many files, or a broad
rename / migrate / signature change — and whenever a drift hook fires on one.

## 📏 Scope First

* Count affected files before touching one. **>20 → stop and plan.**
* Map interface dependencies → update **all** interfaces before any implementation.
* Compile-driven error fixing, base classes outward.
* Finish or revert. No half-measures; a half-migrated tree is heresy.

## 🔧 Mechanics

* Mechanical changes (rename, signature change) → IDE/LSP. A fragile `sed` one-liner that
  fails silently is worse than doing it by hand.
* Cleaner solution in reach → promote it, even when not 100% equivalent. Say what differs.

## 🗂️ Plans

* `docs/plans/<name>.md` — high-level only; keyword task names; scoped to the module touched.
* **Never commit unless explicitly asked** — brainstorming and design artifacts included.
