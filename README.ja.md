<p align="center"><img src="./crates/app/icons/128x128@2x.png" alt="Riffle" width="128"></p>

<p align="center"><a href="./README.md">English</a> | 日本語</p>

# Riffle

撮った RAW をすばやく見ていき、残すものと捨てるものを決めるためのセレクトアプリです。

- RAW に埋め込まれた JPEG プレビューだけを読むので、数千枚のフォルダーでも、写真を切り替えるたびに待たされることがありません。
- スター、採用 / 不採用、カラーラベルは、Lightroom など XMP サイドカーを読むソフトや DxO PhotoLab が読めるサイドカーファイルに書き込みます。RAW ファイル自体には一切書き込みません。
- 現像も編集もしません。やることは「選ぶ」ことだけです。

対応カメラと形式は[対応状況](#対応状況)を参照してください。

## はじめに

### インストール

[Releases ページ](https://github.com/minodisk/riffle/releases)の最新リリースから、OS に合ったファイルをダウンロードします。

| OS | ファイル |
|----|------|
| macOS（Apple Silicon） | `Riffle_<version>_aarch64.dmg` |
| macOS（Intel） | `Riffle_<version>_x64.dmg` |
| Windows | `Riffle_<version>_x64-setup.exe` |
| Linux | `Riffle_<version>_amd64.AppImage` |

アプリは OS ベンダーの署名を受けていないため、初回起動時に一度だけ許可が必要です。

- **macOS**: `Riffle.app` を右クリック → 開く。
- **Windows**: SmartScreen の警告で「詳細情報」→「実行」。

以降のアップデートは起動時に自動でダウンロードされ、次回起動時に反映されます。その他のパッケージや詳細は [docs/usage.md](./docs/usage.md#installing)（英語）を参照してください。

### 最初の一歩

1. 初回起動時に表示されるダイアログで、使っている現像ソフト（Lightroom、DxO PhotoLab、またはその両方）を選びます。あとから設定で変更できます。
2. フォルダーをウィンドウにドロップします（`Cmd+O` / `Ctrl+O` でも開けます）。
3. `←` `→` で前後の写真に切り替え、`1`-`5` でスターを付け、`x` で不採用にします。
4. ピントが怪しいときは `z` で等倍表示にして、フォーカス位置を確認します。

判定はその場でサイドカーに保存されるので、保存操作はいりません。終わったら Lightroom など XMP サイドカーを読むソフトや DxO PhotoLab がそのまま取り込みます。

## 主な機能

- **フォルダーツリー**: 左ペインでホームとマウント中のボリュームをたどり、フォルダーをクリックすると開きます。クリックするとツリーがキーボードを受け取り、矢印キー、`Home` / `End`、`Enter`、名前の入力でフォルダーを移動して開けます。`Escape` を押すまでカリングのキーは効きません。フォルダーを右クリックすると、Finder / エクスプローラー / ファイルマネージャーでそのフォルダーを表示したり、パスや名前をコピーしたり、`Rename…` で名前を変えたり、不採用の写真をサブフォルダーごと、またはそのフォルダーだけゴミ箱に移したりできます。`Cmd+クリック` / `Ctrl+クリック` と `Shift+クリック` で複数のフォルダーを選ぶと、右クリックからそれらの不採用の写真をまとめて移せます。開いているフォルダーの名前をゆっくり 2 回クリックしても名前を変えられます。名前はその場で編集し、`Enter` かほかの場所をクリックすると確定、`Escape` で取り消します。インデックス、レーティング、最後に見ていた位置もフォルダーについていき、開いていたフォルダーなら新しい名前で開き直します。
- **フィルムストリップ**: 下端にサムネイルが並び、クリックで表示します。`Cmd+クリック` / `Ctrl+クリック` で 1 枚ずつ追加・解除、`Shift+クリック` や `Shift+←` / `Shift+→` で範囲選択、`Cmd+A` / `Ctrl+A` でフィルターが表示しているすべてを選択でき、判定は選択中のすべてに適用されます。ファイルを右クリックして `Rename…` を選ぶか、表示中のファイルの名前を少し間をおいてもう一度クリックすると、名前をその場で変えられます。`Enter` かほかの場所をクリックすると確定、`Escape` で取り消し、XMP と `.dop` のサイドカーとレーティングもファイルについていきます。`F6` でフィルムストリップ、`F7` でフォルダーツリー、`F8` でメタペイン、`Tab` で左右両方のペインを隠し、ビューアを広げられます（`View` メニューからも操作でき、フォーカスマーク、1:1、Compare も並びます）。メタペインの `Maker note` グループには、Sony の AF、ドライブ、手ブレ補正、画作りの設定を表示します。
- **等倍ピントチェック**: `z` でフォーカス位置を中心に等倍表示します。別の写真に切り替えてもズームは保たれます。
- **フォーカスマーク**: `f` で、AF 枠を記録するカメラではカメラが使った AF 枠と、そのフォーカス位置の十字を表示します。位置だけを記録するカメラでは十字のみ、AF 位置のないカメラやマニュアルフォーカスの写真では何も表示しません（[カメラが記録する情報](./docs/cameras.md)（英語））。AF 点に最も近い顔の目にピントが合っている可能性が高い（目のシャープさとエッジ幅を組み合わせた確率で判定します）「ピント候補」ならマークは緑、AF 点の近くに顔があるもののその目にピントが合っている可能性が低ければオレンジ、Riffle が判断できなければ（AF 点がない、マニュアルフォーカス、AF 点の近くに顔がない、まだ計算していない）白になります。カメラの顔追尾はマークの色に影響しなくなりました。この判定はサムネイルの直後に 2 回目のパスで計算するため、進むにつれてマークが白から変わります。ストリップではピント候補のセルの左下に緑の顔アイコンが付き、メタペインにはその確率を `AF eye in focus` としてパーセントで表示し、フィルターメニューの `AF eye` セクション（`Sharp` / `Soft` / `Unknown`）でこの状態ごとに絞り込めます。マークは Riffle が AF 点の近くで検出した顔も、シアンの枠と両目の間の点で表示します。検出はコマの表示時に行うため、`f` を押してから少し遅れて現れます。
- **シャープネス表示**: 各サムネイルの横のバーで、連写のどのコマが最もシャープかがわかります。カメラが顔追尾を記録していればカメラの瞳 AF 枠で、そうでなければ AF 点の周辺で、カメラが AF 点を記録していなければ顔が見つかれば被写体の目で、それもなければ最もシャープな領域でスコアを取ります（[カメラが記録する情報](./docs/cameras.md)（英語））。
- **オフライン顔検出**: 顔と目は同梱の [YuNet](https://github.com/opencv/opencv_zoo/tree/main/models/face_detection_yunet) モデル（MIT ライセンス）で検出します。ローカルで動き、ネットワークにはアクセスしません。ストリップのピント候補アイコン（フィルターメニューの `Sharp` 項目にも表示）は [Lucide](https://lucide.dev) の `scan-face`（ISC ライセンス）です。
- **連写**: 1 秒以内に撮られたコマはストリップ上で帯と枚数バッジでまとめられます。`ArrowUp` / `ArrowDown` で連写間を移動、`Alt+ArrowLeft` / `Alt+ArrowRight` で連写内のコマを順に移動して端で止まり、`Shift+x` で連写の残りを不採用にします。1 秒未満の撮影時刻を記録しないカメラでは秒単位でまとめます（[カメラが記録する情報](./docs/cameras.md)（英語））。
- **比較**: `v` で選択した 2〜4 枚を並べて表示するか、表示中の写真とその連写内で最もシャープなコマを並べます。コマをクリックすると、そのコマだけにスターや採用 / 不採用を付けられます。
- **不採用をゴミ箱へ移動**: フォルダーツリーでフォルダーを右クリックして `Move Rejected to Trash…` を選ぶと、そのフォルダーの不採用の写真をゴミ箱に移します。開いていないフォルダーでも使えます。`Move Rejected to Trash, Including Subfolders…` を選べば、年単位のフォルダー全体をまとめて処理できます。`Cmd+クリック` / `Ctrl+クリック` / `Shift+クリック` で複数のフォルダーを選んで右クリックすれば、それらをまとめて処理できます。移動の前に、フォルダーごとの不採用の数と空く容量の合計をダイアログで確認できます。削除はしないので、`Edit > Undo` 1 回で移動をまるごと取り消せ、判定も戻ります。元の場所に同じ名前のファイルがあるとき、ゴミ箱を空にしたとき、手で戻し済みのときは、そのファイルを報告してそのままにします。続けて `Edit > Redo` を使うと、戻ってきたファイルをもう一度ゴミ箱に移します。
- **JPEG の撮影時刻の連番化**: Riffle でカリングし、残した写真を RAW 現像ソフトから JPEG で書き出したら、フォルダーツリーで書き出し先のフォルダーを右クリックして `Sequence JPEG Timestamps…` を選ぶと、連写の撮影時刻が 1 秒ずつずらされます。1 秒未満の時刻を無視する Google フォトでも、コマが撮影順に並びます。並び順は撮影時刻、1 秒未満の時刻、ファイル名の順で決まるため、2 台のカメラから書き出したフォルダーでも正しく交互に並びます。書き込む前に新しい時刻をプレビューで確認できます。結果は書き出し先フォルダーの隣の `<フォルダー>-sequenced/` に完全なコピーとして書き出され、実行が終わるとそのフォルダーがファイルマネージャーで表示されます。元のファイルには一切手を加えません。もう一度実行すると、元の時刻からコピーを作り直します。
- **JPEG だけのフォルダー**: `<フォルダー>-sequenced/` の書き出し結果のように JPEG だけが入ったフォルダーは閲覧専用で開くので、並び順と撮影時刻を Riffle の中で確かめられます。サムネイル、プレビュー、EXIF の行を撮影時刻順に表示します。スター、フラグ、カラーラベル、サイドカー、ピント候補の表示はありません。
- **MCP コンパニオン**: AI アシスタントなどの MCP クライアントが、カリングに付き添えます。Riffle が表示している内容を読み、プレビューを見て、表示を動かし、キーと同じ経路でスター、採用 / 不採用、ラベルを付けられます。デフォルトはオフで、このコンピューター内からしか接続できず、ゴミ箱への移動はできません（[MCP コンパニオン](#mcp-コンパニオン)）。

すべての機能と、キーの一覧は [docs/usage.md](./docs/usage.md)（英語）で説明しています（[キー](./docs/usage.md#keys)）。

## 他のソフトとの連携

判定は RAW の隣のサイドカーファイルに保存されます。形式は初回起動時に選び、あとから設定で変更できます。

- **Lightroom (XMP)**: `FOO.ARW` に対して `FOO.xmp` を作ります。Lightroom などが読む形式です。採用 / 不採用は `xmpDM:good` に、カラーラベルは Lightroom と同じく `photoshop:LabelColor` と `xmp:Label` の両方に書き込みます。`xmp:Label` に書く名前は色ごとに設定で変えられます。
- **PhotoLab (.dop)**: `FOO.ARW` に対して `FOO.ARW.dop` を作ります。採用 / 不採用もここに入ります。
- **Both**: 判定を `FOO.xmp` と `FOO.ARW.dop` の両方に書き込みます。一度のカリングで Lightroom と PhotoLab の両方で現像できます。フォルダーを開いたときは 2 つのうち最後に更新された方を読むので、どちらのソフトで後から編集しても反映されます。Both に切り替えても既存のファイルは書き換えないので、判定し直すまでは 1 枚の 2 つのサイドカーの内容が食い違うことがあります。

他のソフトが作ったサイドカーはその場で編集します。スター、フラグ、ラベル以外（現像設定、キーワードなど）には手を触れません。

同じフォルダーにある、名前が同じで拡張子だけが違う 2 つの RAW（`FOO.ARW` と `FOO.DNG`）は `FOO.xmp` を共有するため、サポートしていません。一方の名前を変えたりゴミ箱に移したりすると、共有の `FOO.xmp` も一緒に移り、もう一方の判定も持っていかれます。RAW ごとに `FOO.ARW.dop` を作る PhotoLab (.dop) 形式だけを最初から使い、フォルダーに `FOO.xmp` を置かないか（**Both** では共有の `FOO.xmp` も書き込みます。また、選んだ形式にかかわらず、既存の `FOO.xmp` は一緒に移ります）、そうした RAW を別々のフォルダーに分けてください。

### Lightroom

- Lightroom（Classic ではないもの）は、写真を読み込むときに Riffle が書いた XMP を読みます。
- Lightroom で変更したスター、フラグ、カラーラベルは、Lightroom 自身が RAW の隣の `.xmp` に書き戻します。Lightroom Classic と違い、`Ctrl+S` や自動書き出しの設定はいりません。Riffle はフォルダーを開いたときにその変更を読み込みます。
- Windows 版の Lightroom 9.5.1 で確認しています。

### Lightroom Classic

- Lightroom Classic は、写真を最初に読み込むときに Riffle が書いた XMP を読みます。
- 読み込み後は、Riffle が変更したサイドカーを再読み込みしません（再起動しても同じです）。選択中の写真だけなら、ライブラリのグリッドで写真を右クリックして `Metadata > Read Metadata from File` を選びます。フォルダー全体なら、フォルダーを右クリックして `Synchronize Folder...` を選び、`Scan for metadata updates` にチェックを入れて `Synchronize` をクリックしてください。
- Lightroom Classic はデフォルトでは XMP を書き出しません。`Ctrl+S`（`Metadata > Save Metadata to File`）で選択中の写真に書き出すか、`Catalog Settings > Metadata > Automatically write changes into XMP` をオンにしてください。
- カラーラベルは Lightroom Classic のカラーラベルセット（`Metadata > Color Label Set > Edit...`）と名前で照合されます。セットのラベル名を `Red`、`Yellow`、`Green`、`Blue`、`Purple` に変えるか、Riffle の設定でラベル名をセットの名前に合わせてください。Lightroom の各言語版のデフォルトセット用のプリセットを用意しています（現在は英語と日本語）。言語の追加は [JSON ファイル 1 つ](./crates/core/i18n/README.md)で済みます。

### DxO PhotoLab

- PhotoLab は `.dop` の中の識別子で画像を見分けます。PhotoLab が一度見た画像（`.dop` が書かれていなくても、フォルダーを開くだけで該当します）に、PhotoLab の知らない識別子の `.dop` を新しく作ると、Riffle の判定を持った仮想コピーとして取り込まれ、マスターは元の状態のままになります。
- そこで Windows では、Riffle は `.dop` を新しく作るときに PhotoLab のデータベース（最も新しい `%APPDATA%\DxO\DxO PhotoLab N\Database\PhotoLab.db`）から画像の識別子を読んで書き込みます。PhotoLab は判定をマスターに反映します。
- データベースが見つからないとき（macOS、または PhotoLab が入っていないとき）も、新しい `.dop` はそのまま使えます。PhotoLab はマスターとして取り込みますが、すでにその画像を見ていた場合は判定が仮想コピーに入ります。
- すでにある `.dop` は、これまでどおりその場で編集します。
- Windows 版の PhotoLab 10.0.1 で確認しています。

### MCP コンパニオン

Riffle は [MCP](https://modelcontextprotocol.io/) のエンドポイントを提供でき、MCP クライアント（AI アシスタントなど）をカリングの相棒にできます。設定の `MCP` タブで `Let MCP clients connect` をオンにすると（デフォルトはオフ）、サーバーがこのコンピューター内だけで次の URL を待ち受けます。

```text
http://127.0.0.1:41917/mcp
```

Streamable HTTP で MCP を話すクライアントなら、この URL だけで接続できます。例を 2 つ挙げます。

- **Claude Code**:
  `claude mcp add --transport http riffle http://127.0.0.1:41917/mcp`
- **Claude Desktop**: `claude_desktop_config.json` に次を追加します。`npx -y mcp-remote` を経由するので Node.js が必要です（`url` を直接書く形は検証していません）。

  ```json
  {
    "mcpServers": {
      "riffle": {
        "command": "npx",
        "args": ["-y", "mcp-remote", "http://127.0.0.1:41917/mcp"]
      }
    }
  }
  ```

`MCP` タブには URL とこれらの例がコピーボタン付きで表示されます。各ツールの動作は [docs/usage.md](./docs/usage.md#mcp-companion)（英語）で説明しています。

## 対応状況

チェックのないものはまだ検証できていません。使ってみた結果の報告は大歓迎です。

### OS

- macOS（Apple Silicon）
  - [x] 26
- Windows
  - [x] 11
- Linux
  - [x] Ubuntu

動いた場合は Discussions の[OS 動作報告スレッド](https://github.com/minodisk/riffle/discussions/286)に投稿してください。動かない場合は[OS 用の issue テンプレート](https://github.com/minodisk/riffle/issues/new?template=os.yml)から issue を立ててください。

### RAW 形式とカメラ

- ARW
  - [x] Sony α1
  - [x] Sony α9 III
  - [x] Sony α7 V
  - [x] Sony α7 IV
  - [x] Sony α7R V
  - [x] Sony α7S III
  - [x] Sony α7C II
  - [x] Sony α7CR
  - [x] Sony α6700
  - [x] Sony ZV-E1
- CR3
  - [x] Canon EOS R
  - [x] Canon EOS RP
  - [x] Canon EOS R3
  - [x] Canon EOS R5
  - [x] Canon EOS R5 Mark II
  - [x] Canon EOS R6
  - [x] Canon EOS R6 Mark II
  - [x] Canon EOS R6 Mark III
  - [x] Canon EOS R7
  - [x] Canon EOS R10
  - [x] Canon EOS R50
  - [x] Canon EOS R50 V
  - [x] Canon EOS R100
- DNG
  - [x] Leica M11-P
  - [x] SIGMA BF
  - [x] SIGMA fp L
- NEF
  - [x] Nikon Z 9
  - [x] Nikon Z 8
  - [x] Nikon Z 7II
  - [x] Nikon Z 6II
  - [x] Nikon Z 6
  - [x] Nikon Z 5
  - [x] Nikon Z f
  - [x] Nikon Z fc
  - [x] Nikon Z 50
  - [x] Nikon Z 30
  - [x] Nikon D850
  - [x] Nikon D500

HDR PQ（HEIF）で撮影した CR3 には JPEG のプレビューがなく、まだ開けません。

JPEG（`.jpg` / `.jpeg`）だけが入り、RAW ファイルを含まないフォルダーは閲覧専用で開きます。サムネイル、プレビュー、EXIF の行を撮影時刻順に表示し、スター、フラグ、カラーラベル、サイドカー、ピント候補の表示はありません。

カメラごとに記録している情報と、それによって変わる機能は [docs/cameras.md](./docs/cameras.md)（英語）にまとめています。対応状況を形式ごとではなくカメラごとに載せている理由は [docs/raw-formats.md](./docs/raw-formats.md)（英語）で説明しています。

リストにないカメラで動いた場合は Discussions の[カメラ動作報告スレッド](https://github.com/minodisk/riffle/discussions/287)に投稿してください。動かない場合は[カメラ用の issue テンプレート](https://github.com/minodisk/riffle/issues/new?template=camera.yml)から issue を立ててください。調査にはサンプルファイルが必要なので、添付（またはリンク）をお願いします。1 枚で十分ですが、できれば横位置と縦位置の 2 枚があると、回転とフォーカスマークも確認できます。

### サイドカー形式とソフト

- XMP
  - [x] Adobe Lightroom
  - [x] Adobe Lightroom Classic
- DOP
  - [x] DxO PhotoLab

Riffle で付けたスター、フラグ、カラーラベルがソフト上で正しく表示された場合は、Discussions の[ソフト動作報告スレッド](https://github.com/minodisk/riffle/discussions/288)に投稿してください。表示されない場合は[ソフト用の issue テンプレート](https://github.com/minodisk/riffle/issues/new?template=software.yml)から issue を立ててください。

## 開発者向け

- ソースからのビルド: [CONTRIBUTING.md](./CONTRIBUTING.md)（英語）
- パフォーマンス計測: [docs/performance.md](./docs/performance.md)（英語）
