# Learnings

- Step 1: the scan-end refresh reuses the existing `scan-state` listener; the
  true->false transition is computed before `scanRunning` is overwritten, and
  `clearInFlight` is checked both before the fetch and in its `.then`.
