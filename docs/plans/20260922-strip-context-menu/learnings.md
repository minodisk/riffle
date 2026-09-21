# Learnings

## Step 1

- `keymap.get` returns `string | undefined`, so the listener guards `undefined` before calling `runAction(action: string)`; the old `default: return` path now returns `false`.
