# Learnings

## Step 1

- The old guard `MODIFIER_KEYS.includes(event.key)` was an exact-case match on
  `event.key` only. Replaced it with a `MODIFIER_CODES` list checked against
  `event.code` plus a lower-cased `event.key` check, so a lone modifier is
  rejected in any casing and with or without its own flag set.
- The actual webview event shape was not reproduced (no way to drive the
  WKWebView from the test suite); the fix covers every shape consistent with
  the symptom.

## Deferred issues (todo candidates)

- Already-persisted bogus keys (`control`, `shift`, `alt`, `meta` chips in the
  user's `shortcuts` store) are not cleaned up. Basis: plan.md "Trade-offs and
  risks" declares the migration out of scope for this step. Related files:
  `crates/app/src/shortcuts.rs`.
