# Japanese headings and terms

Headings chosen in the `.ja.md` files and their GitHub slugs (raw Japanese,
lowercased Latin, spaces to `-`, punctuation such as `:` and `、` dropped), so
later steps link to them and reuse the same terms.

## `docs/humans/cameras.ja.md`

| English heading | Japanese heading | Slug |
|---|---|---|
| `# What the camera records` | `# カメラが記録する情報` | `#カメラが記録する情報` |

Camera table columns: カメラ / AF 点 / AF 枠のサイズ / 顔追尾 / 1 秒未満の撮影時刻.
The Fujifilm size table: 埋め込み JPEG のサイズ / Fujifilm のカメラ.

## `docs/humans/raw-formats.ja.md`

| English heading | Japanese heading | Slug |
|---|---|---|
| `# How RAW files differ, and why support is listed per camera` | `# RAW ファイルはどう違うのか、対応状況をカメラごとに載せている理由` | `#raw-ファイルはどう違うのか対応状況をカメラごとに載せている理由` |
| `## The layers` | `## 層の構造` | `#層の構造` |
| `## What Riffle reads for each feature` | `## 各機能で Riffle が読むもの` | `#各機能で-riffle-が読むもの` |
| `## How the six formats differ` | `## 6 つの形式の違い` | `#6-つの形式の違い` |
| `### ARW: no full-size JPEG on older bodies` | `### ARW: 古いカメラにはフルサイズ JPEG がない` | `#arw-古いカメラにはフルサイズ-jpeg-がない` |
| `### NEF: two JPEGs besides the thumbnail, told apart by order` | `### NEF: サムネイル以外に 2 つの JPEG があり、順序で見分ける` | `#nef-サムネイル以外に-2-つの-jpeg-があり順序で見分ける` |
| `### CR3: boxes instead of IFDs, and HEIF files` | `### CR3: IFD の代わりにボックス、そして HEIF ファイル` | `#cr3-ifd-の代わりにボックスそして-heif-ファイル` |
| `### RAF: the Exif rides inside the embedded JPEG` | `### RAF: Exif は埋め込み JPEG の中にある` | `#raf-exif-は埋め込み-jpeg-の中にある` |
| `### ORF: the preview lives in the MakerNote` | `### ORF: プレビューはメーカーノートの中にある` | `#orf-プレビューはメーカーノートの中にある` |
| `## Why the MakerNote varies by body` | `## メーカーノートがカメラごとに異なる理由` | `#メーカーノートがカメラごとに異なる理由` |

## `docs/humans/performance.ja.md`

Nothing links into these headings, so no slugs are listed; all are unique
within the file. Product names, `n=` and the platform stay, in full-width
parentheses.

| English heading | Japanese heading |
|---|---|
| `# Performance` | `# パフォーマンス` |
| `## Sony α7 V ARW (Apple Silicon Mac, n=20)` | `## Sony α7 V ARW（Apple Silicon Mac、n=20）` |
| `## Leica M11-P DNG (Apple Silicon Mac, n=32)` | `## Leica M11-P DNG（Apple Silicon Mac、n=32）` |
| `## Nikon NEF, Canon CR3 and OM System / Olympus ORF (raw.pixls.us samples, Windows)` | `## Nikon NEF、Canon CR3、OM System / Olympus ORF（raw.pixls.us のサンプル、Windows）` |
| `## Fujifilm RAF (raw.pixls.us samples, Windows)` | `## Fujifilm RAF（raw.pixls.us のサンプル、Windows）` |
| `## Per-page preview read` | `## ページごとのプレビュー読み込み` |
| `## Folder scan throughput` | `## フォルダースキャンのスループット` |
| `### Real folders on Windows` | `### Windows の実フォルダー` |
| `### Sharpness scoring cost` | `### シャープネスのスコア計算のコスト` |
| `### Face detection cost` | `### 顔検出のコスト` |
| `#### Real files (Linux WSL2)` | `#### 実ファイル（Linux WSL2）` |
| `#### Focus candidate pass` | `#### ピント候補のパス` |
| `### Which pass carries which cost` | `### どのパスがどのコストを担うか` |
| `## Opening an indexed folder again` | `## インデックス済みフォルダーを再び開く` |
| `### Measuring on your own folder` | `### 自分のフォルダーで計測する` |
| `## The 1:1 focus check path` | `## 等倍ピントチェックの経路` |
| `### End to end, keypress to pixels` | `### エンドツーエンド、キー入力から画素の表示まで` |
| `## What the sidecar pass adds to a folder open` | `## サイドカーのパスがフォルダーを開く処理に加えるもの` |

## Terms

In addition to the terms listed in `plan.md`'s conventions:

| English | Japanese |
|---|---|
| body (camera) | カメラ (a series: `Z シリーズ`, `X シリーズ`, `GFX シリーズ`) |
| AF point | AF 点 |
| AF frame | AF 枠 |
| eye-AF frame | 瞳 AF 枠 |
| face tracking | 顔追尾 |
| focus mark | フォーカスマーク |
| 1:1 view / 1:1 focus check | 等倍表示 / 等倍ピントチェック |
| sub-second capture time | 1 秒未満の撮影時刻 |
| embedded JPEG | 埋め込み JPEG |
| full-size JPEG | フルサイズ JPEG |
| preview / thumbnail | プレビュー / サムネイル |
| MakerNote (prose) | メーカーノート (the meta pane group stays `Maker note`) |
| sensor data / sensor's pixels | センサーデータ / センサーの画素 |
| manual focus | マニュアルフォーカス |
| sharpness score | シャープネスのスコア |
| unconfirmed | 未確認 |
| little- / big-endian | リトルエンディアン / ビッグエンディアン |
| meta pane rows | メタペインの `...` の行 |
| pass (scan) / first pass / second pass | パス / 1 回目のパス / 2 回目のパス |
| focus candidate cue | ピント候補の判定 |
| warm / cold page cache | ウォームな / コールドなページキャッシュ |
| bounded (1MiB prefix) read | 先頭部分に限った読み込み |
| ranged read | 範囲読み込み |
| before / after (a change) | 変更前 / 変更後 |
| wall (time) | 全体時間 |
| keypress to pixels | キー入力から画素の表示まで |
| page turn | ページ送り |
| decode / partial decode / full decode | デコード / 部分デコード / 全体デコード |
| crop (noun) | 切り出し |

## `docs/humans/usage.ja.md`

| English heading | Japanese heading | Slug |
|---|---|---|
| `# Using Riffle` | `# Riffle の使い方` | `#riffle-の使い方` |
| `## Features` | `## 機能` | `#機能` |
| `## Keys` | `## キー` | `#キー` |
| `## Ratings and sidecars` | `## レーティングとサイドカー` | `#レーティングとサイドカー` |
| `## MCP companion` | `## MCP コンパニオン` | `#mcp-コンパニオン` |
| `## Installing` | `## インストール` | `#インストール` |

Feature bullet names: a name that is a menu item or button label stays in
English (**Open Folder…**, **Reload Folder**, **Move Rejected to Trash…**,
**Sequence JPEG Timestamps…**, **Undo**, **Redo**, **Open Log Folder**,
**Clear Cache**), so the in-text "see **...**" references still match the UI;
the others are translated (フォルダー, フィルムストリップ, パネル,
フォーカスマーク, 等倍ピントチェック, グレースケールプレビュー, 比較, 判定,
メタペイン, フィルターメニュー, 並べ替えメニュー, JPEG だけのフォルダー,
自動送り, シャープネス表示, 連写).

| English | Japanese |
|---|---|
| status line | ステータス行 |
| Trash | ゴミ箱 |
| sort (menu) | 並べ替え（メニュー） |
| accelerator | アクセラレーター |
| rebindable / rebind | 割り当てを変更できる / 割り当て直す |
| type-ahead | タイプアヘッド（名前の入力） |
| anchor (selection) | アンカー |
| Compare | 比較 |
| auto-advance | 自動送り |
| label colors in the Keys table | レッド, オレンジ, イエロー, グリーン, ブルー, ピンク, パープル (Lightroom's Japanese names) |
