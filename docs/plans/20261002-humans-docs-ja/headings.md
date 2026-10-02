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
