<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Simplify the Compatibility lists in the READMEs

## Purpose

The "Compatibility" checklists in `README.md` and `README.ja.md` carry
qualifiers (OS, UI language, product version) that belong to the verification
notes rather than to the list of what is supported, and an unchecked
`Capture One` entry that is not a target. Trimming them makes the lists read as
a plain statement of what Riffle supports; the verification details stay in
the "### Lightroom" subsection ("Verified with Lightroom 9.5.1 on Windows.").

With Lightroom, Lightroom Classic and DxO PhotoLab all verified, the user
decided to release this as 1.0.0: the squash merge of this PR carries a
`Release-As: 1.0.0` footer in its commit body (see
`docs/agents/tauri-app.md`, "A `Release-As` footer must be on the squash
commit itself").

## Steps

- [x] Step 1: Simplify the OS and software checklists in both READMEs
  - Done when:
    - `README.md` "### OS": `- macOS` / `  - [x] 26 (Apple Silicon)` becomes
      `- macOS (Apple Silicon)` / `  - [x] 26`; the Windows and Linux lines are
      untouched.
    - `README.md` "### Sidecar formats and software":
      - `- [x] Adobe Lightroom (Windows; not Lightroom Classic)` -> `- [x] Adobe Lightroom`
      - `- [x] Adobe Lightroom Classic (Windows, Japanese UI)` -> `- [x] Adobe Lightroom Classic`
      - the `- [ ] Capture One` line is deleted
      - `- [x] DxO PhotoLab 10` -> `- [x] DxO PhotoLab`
    - `README.ja.md` "### OS": `- macOS` / `  - [x] 26（Apple Silicon）` becomes
      `- macOS（Apple Silicon）` / `  - [x] 26` (full-width parentheses, matching
      the file's existing style, e.g. the download table's `macOS（Apple Silicon）`).
    - `README.ja.md` "### サイドカー形式とソフト":
      - `- [x] Adobe Lightroom（Windows、Lightroom Classic ではないもの）` -> `- [x] Adobe Lightroom`
      - `- [x] Adobe Lightroom Classic（Windows、日本語 UI）` -> `- [x] Adobe Lightroom Classic`
      - the `- [ ] Capture One` line is deleted
      - `- [x] DxO PhotoLab 10` -> `- [x] DxO PhotoLab`
    - Nothing else changes: the "Anything unchecked has not been verified yet"
      paragraph (JA: "チェックのないものはまだ検証できていません。..."), the
      "### Lightroom" subsection's "Verified with Lightroom 9.5.1 on Windows."
      sentence (JA: "Windows 版の Lightroom 9.5.1 で確認しています。"), the RAW
      formats list and the report-thread paragraphs are left as is.
    - Both READMEs change in the same PR; `mise run ci` passes.
  - Implementation approach:
    - Files: `README.md` (lines ~223-224 and ~258-262) and `README.ja.md`
      (lines ~116-117 and ~141-145) only. `crates/core/i18n/README.md` mentions
      "Lightroom Classic 15.5.1 (Windows, Japanese UI)" as the preset's
      verification note; it is out of scope and stays.
    - Commit as `docs(readme): simplify the compatibility lists`.

## Trade-offs and risks

- Moving "(Apple Silicon)" to the macOS heading reads as Intel Macs being out
  of scope, while the download table above still offers an `x64.dmg`. The user
  accepts this reading; no change to the download table is planned.
- The intro paragraph "Anything unchecked has not been verified yet. Reports of
  how it went for you are very welcome." still makes sense after the change:
  it introduces all three checklists (OS, cameras, software) as a whole, and
  the sentence is a general rule rather than a pointer to a specific unchecked
  item. It is kept verbatim.
- Dropping the "(Windows; not Lightroom Classic)" qualifier loses the hint that
  the two Lightroom lines are different products, but the "### Lightroom" and
  "### Lightroom Classic" subsections above already explain the distinction and
  keep the verified version/OS.

## Progress

- (none yet)
