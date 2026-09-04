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

## 📝 API Documentation & Comments

* No narration comments — experts read this.
* Prefer apidoc over comments; a comment that's necessary can often become a well-named method call that itself carries the apidoc.
* Comments only when exceptionally important — decide if it gives real value beyond the name.
* First sentence stands alone: a clear mission statement ending with a `.`.
* Document the contract only — not the internal operation.
* **Reader's time is priceless** — no obvious params, don't repeat the signature, document only the non-obvious.
