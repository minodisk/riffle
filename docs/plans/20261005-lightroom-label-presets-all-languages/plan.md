<plan-guide>
When you start executing this plan, record the in-flight plan path in auto memory (`MEMORY.md`).
After every step's PR is merged, the `develop` skill's wrap-up phase (normally a chore PR, or the same PR as the implementation in single-PR mode) does the archive move and removes the auto memory entry.

Update this file as you go if problems or design changes come up.
Record additional important information (investigation results, design details) in separate files in this folder.
Record what you learn during implementation, and notable events (CI failures, review feedback, plan changes, user intervention), in `learnings.md` as they happen.
After finishing a step, continue to the next without asking the user.
lychee (`mise run lint`) resolves a relative link in `docs/plans/**` from the linking file's own directory, so write links relative to the plan folder (e.g. `../../humans/usage.md`) and put an example path that is not a real link target in backticks.

<pr-rules>
- Include the plan.md update (marking the step done + the Progress entry) in the implementation PR
- Include `Plan: [path]` and `Step: [number]` in the PR description
</pr-rules>
</plan-guide>

# Lightroom color label presets for every Lightroom Classic UI language

## Purpose

The settings window and the first-launch dialog offer a Lightroom default
color label set per language, so a user whose Lightroom Classic runs in that
language gets `xmp:Label` names Lightroom's set matches without typing them.
Only English and Japanese exist so far. Adding the remaining 14 UI languages
Lightroom Classic ships (de, es, fr, it, ko, nb, nl, pl, pt, ru, sv, th,
zh-Hans, zh-Hant) completes the preset list, each from the installed
Lightroom Classic 15.6 (Windows), and the first-launch dialog preselects the
right one for a Chinese browser locale, whose script (`Hans` / `Hant`) the
primary-subtag match alone cannot tell.

## Steps

- [x] Step 1: Add the 14 preset files, make first-run pick a script-coded preset, and update the READMEs
  - Done when:
    - `crates/core/i18n/{de,es,fr,it,ko,nb,nl,pl,pt,ru,sv,th,zh-Hans,zh-Hant}.json` exist, each with the same shape as `ja.json` (`meta.name`, `lightroom.colorLabels.{verified,red,yellow,green,blue,purple}`), the exact values below, and no other keys
    - `cargo test -p riffle-core` passes (the existing tests cover parsing, key set, language-code naming, non-empty values and ordering; no Rust change is expected)
    - the settings window's label-name language dropdown and the first-launch dialog list the 16 presets, English first and then by code (de, es, fr, it, ja, ko, nb, nl, pl, pt, ru, sv, th, zh-Hans, zh-Hant), showing the autonyms; choosing one fills the five names
    - `defaultPreset` in `crates/app/ui/src/firstrun.ts` preselects `zh-Hans` for navigator `zh-CN`, `zh-SG`, `zh`; `zh-Hant` for `zh-TW`, `zh-HK`, `zh-MO`; `pt` for `pt-BR` and `pt-PT`; and still `ja` for `ja-JP` / `ja` / `JA-jp`, `en` (the first preset) for `fr-FR` when no `fr` preset is passed and for `""`
    - `crates/app/ui/src/firstrun.test.ts` has tests for zh-CN, zh-TW, zh-HK, zh, pt-BR, pt-PT against a preset list including `pt`, `zh-Hans`, `zh-Hant`, and the existing tests still pass
    - `README.md` (Lightroom Classic section, "English and Japanese so far") and `README.ja.md` (「現在は英語と日本語」) are updated in sync to say the presets cover every UI language Lightroom Classic ships; no other doc carries the statement (`docs/humans/usage*.md` only use Japanese as an example and stay as they are)
    - `mise run ci` passes
  - Implementation approach:
    - Preset files: copy `crates/core/i18n/ja.json` as the template. `build.rs` picks the files up automatically; `crates/core/src/i18n.rs` needs no change (its code-format test already accepts `pt`, `zh-Hans`, `zh-Hant`). Save as UTF-8 without BOM, 2-space indented, trailing newline; check the formatter in `mise run ci` leaves them alone.
    - `verified` is `"Lightroom Classic 15.6 (Windows, <Language> UI)"` with: German, Spanish, French, Italian, Korean, Norwegian, Dutch, Polish, Brazilian Portuguese (for `pt.json`), Russian, Swedish, Thai, Simplified Chinese, Traditional Chinese. No note about where the names were read from; leave `en.json` / `ja.json` at 15.5.1.
    - `meta.name`: Deutsch, Español, Français, Italiano, 한국어, Norsk bokmål, Nederlands, Polski, Português (Brasil) (for `pt.json`), Русский, Svenska, ไทย, 简体中文, 繁體中文.
    - Values (red|yellow|green|blue|purple), copied exactly:
      - de: Rot|Gelb|Grün|Blau|Lila
      - es: Roja|Amarilla|Verde|Azul|Púrpura
      - fr: Rouge|Jaune|Vert|Bleu|Violet
      - it: Rossa|Gialla|Verde|Blu|Viola
      - ko: 빨강|노랑|초록|파랑|자주
      - nb: Rød|Gul|Grønn|Blå|Lilla
      - nl: Rood|Geel|Groen|Blauw|Paars
      - pl: Czerwony|Żółty|Zielony|Niebieski|Fioletowy
      - pt: Vermelho|Amarelo|Verde|Azul|Púrpura
      - ru: Красный|Желтый|Зеленый|Синий|Фиолетовый
      - sv: Röd|Gul|Grön|Blå|Lila
      - th: สีแดง|สีเหลือง|สีเขียว|สีน้ำเงิน|สีม่วง
      - zh-Hans: 红色|黄色|绿色|蓝色|紫色
      - zh-Hant: 紅色|黃色|綠色|藍色|紫色
    - `defaultPreset(presets, navigatorLanguage)` in `crates/app/ui/src/firstrun.ts`: before the existing primary-subtag match, compute `const { language, script } = new Intl.Locale(navigatorLanguage).maximize()` inside a `try`/`catch` (an empty or invalid tag throws `RangeError`; a webview without `Intl.Locale` throws too; in both cases skip to the subtag match). If `script` is set, look for a preset whose code equals `${language}-${script}` case-insensitively (`code.toLowerCase() === ...toLowerCase()`), then fall through to the current primary-subtag match and the `presets[0]` fallback unchanged. Verified in the project's Node 24: `zh-CN`/`zh-SG`/`zh` maximize to script `Hans`, `zh-TW`/`zh-HK`/`zh-MO` to `Hant`; `pt-PT` → `pt-Latn-PT` finds no `pt-Latn` preset and falls through to `pt`. Update the comment above the function to describe both matches. `Intl.Locale` is typed by the `es2022` lib already in `crates/app/ui/tsconfig.json`.
    - Tests in `crates/app/ui/src/firstrun.test.ts`: extend the `presets` fixture (or add a second one) with `pt`, `zh-Hans`, `zh-Hant`; add cases for zh-CN, zh-TW, zh-HK, zh (script match), pt-BR, pt-PT (subtag match); keep the ja / case / fallback tests. Run `pnpm exec vp test` (`mise run ci` runs it).
    - Runtime check: `Intl.Locale.prototype.maximize` exists in Node 24 (test runtime), Chromium 74+ (WebView2) and Safari 14+ (WKWebView). The vite target `safari13` lacks `Intl.Locale`; the `try`/`catch` keeps the old behavior there. Verify once in the running app that the first-run dialog preselects as expected on at least one locale, and that both dropdowns show the 16 entries in the order above (the options are built from `label_names` in `riffle_core::i18n::presets()` order; no other TS change is expected).
    - README wording: replace the parenthetical with "(every UI language Lightroom Classic ships)" or an equivalent in both files; keep the existing link to `crates/core/i18n/README.md`.

## Trade-offs and risks

- Portuguese is `pt.json` rather than `pt-BR.json`: Lightroom ships only Brazilian Portuguese, and the primary-subtag match then preselects it for both `pt-BR` and `pt-PT` browsers. The file's `meta.name` (`Português (Brasil)`) and `verified` (`Brazilian Portuguese UI`) still record which variant it is. The alternative, `pt-BR.json` plus a mapping in `defaultPreset`, was not taken.
- Chinese needs the script, so `defaultPreset` gains an `Intl.Locale.maximize()` lookup before the subtag match. On a webview without `Intl.Locale` (Safari 13-class WKWebView, within the `safari13` vite target) the lookup throws and is skipped, so Chinese users there get English preselected and choose manually; everything else is unchanged. `maximize()` relies on CLDR likely-subtags data in the engine, which is stable for `zh` but is engine data, not the spec; the tests pin the six cases.
- Dropdown ordering is by language code (en first, then de, es, fr, it, ja, ko, nb, nl, pl, pt, ru, sv, th, zh-Hans, zh-Hant), not by autonym, so e.g. 日本語 sits between Italiano and 한국어. Kept, consistent with the existing `english_comes_first_and_matches_the_default_names` test; changing it would be a separate decision.

## Progress

- (none yet)
