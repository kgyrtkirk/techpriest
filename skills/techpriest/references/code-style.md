# 🛠️ Code-Style Rite — Authoring Doctrine

Companion to `techpriest`. Read before editing any source file, and whenever a drift hook
fires on a source edit.

## ✂️ Surgical Changes

**Touch only what you must. Clean up only your own mess.**

When editing existing code:
* Don't "improve" adjacent code, comments, or formatting.
* Don't refactor things that aren't broken.
* Match existing style, even if you'd do it differently.
* Notice unrelated dead code → mention it, don't delete it.
* Don't remove pre-existing dead code unless asked.

Test: every changed line traces directly to the user's request.

## 🎯 Goal-Driven Execution

**Define success criteria. Loop until verified.**

* "Add validation" → "Write tests for invalid inputs, then make them pass".
* "Fix the bug" → "Write a test that reproduces it, then make it pass".
* "Refactor X" → "Ensure tests pass before and after".

Multi-step tasks → state a brief plan:

```
1. [Step] → verify: [check]
2. [Step] → verify: [check]
```

Strong success criteria let you loop independently. Weak criteria require constant clarification.

## 🏛️ Code Style & Patterns

* Thorough, high standards — always seek better ways. Following repetitive practice blindly is heresy.
* Decompose problems into classes with good contracts.
* **Match existing patterns** — check similar methods before choosing return types or signatures.
* **Eliminate duplication** — abstract base methods + shared impls, not copy-paste across subclasses.
* **Fail explicitly** — no silent fallbacks, null returns, or defensive defaults that hide bugs.
* Evaluate design and correctness, not syntax novelty.

## 📦 Library & Dependency Use (prefer in order)

1. Existing project dependencies — check all imports and transitive deps first.
2. Standard-library rich APIs — `Files.walk`, `PathMatcher`, Guava (`Joiner`, collection utils).
3. Custom code — last resort only.

* **Prefer Guava over Java streams** for collections work.
* **Prefer simple data flow** — return values and plain loops over side-effectful stream pipelines or `forEach`-into-collection indirection.

## 💥 Error Handling

* Only catch exceptions if they can be handled correctly — otherwise rethrow with further useful context.

## 🩹 FIXMEs

* Working on FIXMEs → add all of them to the task list and iterate.
* Carry over every user-placed FIXME/TODO on rewrites; drop only when the issue is demonstrably fixed.
* Replacing a FIXME with an explanatory comment is **not** fixing it — raise unclear fixes in chat first.

## 📐 Checkstyle

* Brace placement: `switch (x) {` (brace on same line).
* Modifier order: `private static final`.

## 📝 Comments

* No narration comments — experts read this!
* Prefer apidoc over comments; a comment that's necessary can often become a well-named method call that itself carries the apidoc.
* Comments only when exceptionally important!

## 📝 API Documentation

* Consider twice before writing an apidoc - will it add real value?
* Consider giving the thing in question a better name.
* Instantiating a contract is global change - not the call site at hand
* Introducing re-usable simple contracts is key.
* **First sentence stands alone: a clear, short mission statement ending with a `.`** — ≤ 20 words; enforced, see below.
* It's only about contract! never about case history; if its about to look like one => rewrite the whole.
* Don't expose/document the internal operation.
* The doc must make sense with no implementer open; otherwise rewrite it.
* **Reader's time is priceless** — no obvious params, don't repeat the signature, document only the non-obvious.

## ⚖️ The Catalogue of Apidoc Heresies

`heresy-guard` judges every `Edit`/`Write` on a `.java` file. A heretical doc denies the call;
the verdict cites each offending doc by its opening line. Rule ids are stable — they key the
session tallies, shared with the shell rite.

The catalogue in `guard/src/apidoc.rs` is the executable form of this section. Change one,
change the other.

* **judged**: docs the edit writes or rewrites — a doc whose lead (first paragraph) is new, or
  a lead-less doc with any change. The lead ends at a blank line, a block tag or block HTML
  (`<p>`, `<ul>`, …); the mission is the lead up to the first `.` followed by whitespace, `<`
  or its end.
* **spared**: an untouched lead (you edited only its tags), `{@inheritDoc}`, and a `/**` ahead
  of `package` outside `package-info.java` — a licence header documents nothing.

### `apidoc-missing-mission` — only tags, no mission statement
* **why**: bare tags leave the reader to reassemble the contract from its parts.
* **correct**: open the doc with one short sentence stating the contract, closed by `.`.

### `apidoc-unterminated-mission` — first sentence not closed by `.`
* **why**: javadoc cuts the summary at the first `. ` — without one the summary is a fragment
  or runs on.
* **correct**: close it with `.`. A lead ending in `:` before a list does not stand alone.

### `apidoc-long-mission` — first sentence over 20 words
* **why**: it is the summary every index and hover shows — reader's time is priceless.
* **correct**: cut it to the contract alone; detail goes to a later paragraph, or into a better
  name.
* **counts**: words as rendered — an inline `{@…}` tag is one word, HTML markup none.
