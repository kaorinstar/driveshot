# driveshot

[English version](README.md)

画面の一部を取り込み、ご自身のクラウドドライブへ保存し、共有URLを発行します。設定した保存期間を
過ぎたファイルは自動で削除します。

**Driveshotは開発初期の段階で、まだリリースしていません。** 画面の取り込み、アップロード、共有、
削除はいずれも未実装です。現在あるのは [開発の状況](#開発の状況) に記載した骨組みだけです。
この文書は、これから作るものを説明し、未実装の部分を明示します。

## 目的

スクリーンショットの共有は、通常、他社のサービスへアップロードする形になります。そのファイルは、
サービス側の判断があるまで残り続けます。Driveshotは、利用者がすでに契約し管理しているドライブ
（Googleドライブ、OneDrive、Dropbox）へ保存し、利用者が設定した期間の経過後に削除します。
必要な間はURLで共有でき、その後はファイルごとなくなります。

## 実装する機能

1. ホットキーで画面の一部を**取り込む**。
2. 各サービスのOAuth認証を通じてクラウドドライブへ**アップロードする**。Driveshotがパスワードを
   受け取ることはありません。
3. アップロードしたファイルを「リンクを知っている人は閲覧可」に設定し、URLをクリップボードへ
   コピーして**共有する**。
4. 保存期間の経過後に**削除する**。期間は1日・1週間・1か月・無期限から利用者が選びます。

アップロードを実装するまでの間、取り込んだ画像は**ピクチャフォルダーの中の Driveshot フォルダー**に
保存され、そのまま残ります。

```
C:\Users\<ユーザー名>\Pictures\Driveshot\driveshot-20260921-211731.png
```

保存先は決め打ちにせず、WindowsやmacOSに「ピクチャはどこか」を尋ねて決めます。そのため、ピクチャ
フォルダーがOneDriveなどへ移動されている場合は、画像もそちらに保存されます。最後に保存した画像の
フルパスは設定ウィンドウに表示されます。探す場合はこれが最も確実です。

## 開発の状況

| 項目 | 状況 |
|---|---|
| リポジトリ、ビルド、テスト、CI、パッケージ化 | 動作する |
| 導入と起動（Windows） | 実機で確認済み |
| 導入と起動（macOS） | ビルドは通るが、未実行 |
| 保存期間とアップロード記録の処理（`driveshot-core`） | 実装・テスト済み |
| 設定ウィンドウ | 骨組み。保存先の一覧と削除予定日時を表示する |
| トレイ常駐とホットキー | Windowsの実機で確認済み |
| 画面の一部の取り込みとローカル保存 | Windowsの実機で確認済み（拡大率100%・125%・150%） |
| クラウドへのアップロードとOAuth認証 | 未着手 |
| 共有URLの発行 | 未着手 |
| 期限による自動削除 | 未着手 |

設計上の未決定事項が2点あります。最初に対応するクラウドドライブの数
（[#2](https://github.com/kaorinstar/driveshot/issues/2)）と、パソコンの電源が切れている間も削除を
実行する必要があるかどうか（[#3](https://github.com/kaorinstar/driveshot/issues/3)）です。
それぞれの選択肢と負担は [docs/architecture.ja.md](docs/architecture.ja.md) に記載しています。

## 動作条件

- **Windows 10以降。** ウィンドウの描画にはWebView2を使います。Windows 11には標準搭載されており、
  Windows 10でもWindows Update経由でほぼすべての環境に導入済みです。
- **macOS 10.15以降。** ウィンドウの描画にはmacOS標準のWKWebViewを使うため、追加の導入は不要です。
  画面の取り込みには、別途「画面収録」の許可が必要です。初回利用時にmacOSが確認します。

## ソースからビルドする

[Rust](https://www.rust-lang.org/tools/install)（stable）と [Node.js](https://nodejs.org/) 22以降が
必要です。

```
npm install
npm run tauri build
```

インストーラーは `target/release/bundle/` に作られます。ワークスペース全体のビルド出力先は
リポジトリのルートです。`src-tauri` はワークスペースの一部のため、その下に `target/` は
作られません。

開発中に動かす場合は次のとおりです。

```
npm run tauri dev
```

**LinuxはDriveshotの配布対象ではありません。** Linux向けのパッケージは作りません。ただし、
ソースはLinuxでもビルドできます。開発作業の多くはLinux上で行うため、この点は重要です。
ビルドには、表示エンジン・トレイ・画面取り込みが使うシステムライブラリが必要です。
Ubuntu・Debianの場合は次のとおりです。

```
sudo apt-get install libwebkit2gtk-4.1-dev libsoup-3.0-dev libgtk-3-dev librsvg2-dev \
  patchelf libayatana-appindicator3-dev libpipewire-0.3-dev libgbm-dev libdrm-dev \
  libegl1-mesa-dev libwayland-dev libclang-dev clang
```

これらを導入しない場合、実行できないのはアプリ本体のビルドだけです。重要な部分は次の範囲で
確認できます。

```
cargo fmt --all -- --check
cargo clippy -p driveshot-core --all-targets -- -D warnings
cargo test -p driveshot-core
npm run build
```

警告はすべてエラーとして扱います。警告が残っている状態で作業を完了としないでください。

## フォルダー構成

```
crates/driveshot-core/   UI・OSに依存しない処理。すべてのOSでテストできる。
src-tauri/               アプリ本体。ウィンドウ、コマンド、今後の取り込みとアップロード。
src/                     設定ウィンドウ。HTML・CSS・TypeScript。
docs/                    設計方針（英語・日本語）。
tools/make-icon.py       アプリのアイコンを描画する。詳細はファイル冒頭のコメントを参照。
.github/workflows/       検証用（build.yml）と配布用（release.yml）。
```

新しい計算処理は、まず `crates/driveshot-core` に置くことを検討してください。理由は
[docs/architecture.ja.md](docs/architecture.ja.md) に記載しています。

## 自動ビルド

ワークフローを2つに分けています。検証用の実行がリポジトリへの書き込み権限を必要としないように
するためです。

- **`.github/workflows/build.yml`** は、`main` へのpushとすべてのプルリクエストで動きます。
  書式の確認、`driveshot-core` の静的解析とテスト（Linux）、Windows・macOSのいずれか、または
  両方でのワークスペース全体のビルドとテスト、設定ウィンドウのビルド、依存関係の既知の脆弱性と
  ライセンスの確認を行います。パッケージは作らず、`contents: read` の権限で動きます。
- **`.github/workflows/release.yml`** は、ビルド・テスト・パッケージ化を行います。`v0.1.0` の
  ようなタグをpushすると、WindowsのインストーラーとmacOSのディスクイメージの2つを添付した
  リリースを公開します。手動実行した場合は、同じものをビルド成果物として作り、リリースは作成
  しません。タグを付ける前にパッケージ化の変更を試せる唯一の方法です。`contents: write` の権限を
  持つのは、このワークフローだけです。

`build.yml` がアプリケーションをどのプラットフォームでビルドするかは、同ファイル冒頭の
`APP_PLATFORMS` に書いてあります。値は `windows`・`macos`・`both` のいずれかです。現在は
`windows` ですが、そうしている理由はなくなりました。非公開リポジトリで課金される時間を減らすため
でした。macOSの実行時間はLinuxの10倍で課金され、両方をビルドした実行では課金対象80分のうち約50分
がmacOSのジョブでした。公開リポジトリでは標準のランナーが無料です
（[#30](https://github.com/kaorinstar/driveshot/issues/30)）。実行時間の短縮が目的だったことも
ありません。2つのジョブは同時に動き、長いのはWindowsのほうです（両方をビルドした直近の実行では
Windowsが12分16秒、macOSが4分20秒。Windowsはキャッシュが効くと5分5秒です）。`windows` のままに
しておく代償は、その間はmacOSでのコンパイルが一度も行われないことです。手動実行で **Platforms**
に `macos` を選べば、コミットせずに確認できます。`v*` タグはこの設定を参照しません。リリースは
常に両方のパッケージを作ります。

`.github/workflows/report-build-status.yml` は、上記2つのジョブ終了後に呼ばれます。
対象はpushのみです。失敗時は `ci-failure` ラベルの付いたIssueを作成します。すでに開いている場合は、
2つ目を作らずコメントを追加します。次に成功したときも同じIssueにコメントします。Issueを閉じる
ことはありません。ビルドが通ったことは、症状が消えたことを示すだけで、原因を把握したことを示さない
ためです。

各GitHub Actionsは、すべてコミットSHAで固定し、横のコメントにバージョンを記載しています。タグは
所有者が移動できる参照だからです。`.github/dependabot.yml` がCargo・npm・Actionsを毎週確認するため、
固定しても古いままにはなりません。

依存関係のライセンスは [cargo-deny](https://github.com/EmbarkStudios/cargo-deny) で確認します。
許可するライセンスとその理由は `deny.toml` に記載しています。手元で確認する場合は
`cargo install cargo-deny` の後に `cargo deny check` を実行してください。

[CodeQL](https://codeql.github.com/) は、このプロジェクト自身のコードを解析します。上記の依存関係
の確認では扱えない範囲です。`.github/workflows/codeql.yml` が、`main` へのpush、すべての
プルリクエスト、週1回、および手動実行で動きます。ジョブは言語ごとに分かれており、Rustの
ワークスペースと、設定ウィンドウのTypeScriptを対象にします。解析のためのビルドは行わず、どちらも
ソースから読み取ります。検出結果は **Security** → **Code scanning** に表示されます。

**CodeQLのRust対応はパブリックプレビューです。** Rust側で何も出なかった場合、それは「報告がなかった」
ことを意味します。「報告すべきものがない」ことの確認にはなりません。

## リリース手順

タグは `vMAJOR.MINOR.PATCH` 形式です（例：`v0.1.0`）。セマンティックバージョニングに従います。
`1.0.0` より前では、マイナー番号が追加と変更、パッチ番号が修正のみを表します。先頭の0は付けません。

`version.md` が変更履歴です。新しい版が上に並び、1つの版につき1つの節を設けます。`version.ja.md`
はその日本語訳で、同じコミットで更新します。`release.yml` はタグと一致する見出しの節をリリース
説明文として使うため、タグをpushする**前に**記載をコミットする必要があります。タグの形式誤り、
節の不在、節が空の場合は、ビルド開始前にワークフローが失敗します。

利用者に影響する変更は、その変更を行うプルリクエストの中で、両方のファイルの `## Unreleased` に
項目を書き足します。リリース準備は、その見出しをバージョン番号へ書き換え、`src-tauri/Cargo.toml`
の `[package]` にある `version` を同じ番号に設定するだけです。後からコミット履歴をたどって一覧を
作り直す必要はありません。

公開作業は [リリース作成ページ](https://github.com/kaorinstar/driveshot/releases/new) から行います。
`main` に対して `v0.1.0` のような新しいタグを指定し、タイトルと説明は空のまま公開してください。
タグが `release.yml` を起動し、`version.md` の内容で両方を埋め、パッケージを添付します。

## 既知の制約

- **コード署名をしていません。** Windowsのインストーラーは未署名のため、初回実行時にSmartScreenの
  警告が出ます。macOS版は署名も公証もしていないため、「システム設定」→「プライバシーとセキュリティ」
  で許可するまで起動できません。どちらも証明書の購入で解決しますが、未対応です
  （[#12](https://github.com/kaorinstar/driveshot/issues/12)）。
- **保存期間の処理は、Driveshotの動作中のみ実行されます。** 期限を過ぎてもパソコンの電源が入って
  いなければ、次回起動時まで削除されません。詳細は
  [docs/architecture.ja.md](docs/architecture.ja.md) を参照してください。
- **アイコンはコードで描画しています。** `tools/make-icon.py` で生成したもので、デザイナーによる
  ものではありません。より良いものに差し替えるまでの、正式なアイコンとして使います。

## 今後の予定

予定している順序です。前述の未決定事項2点の結論により変わる可能性があります。

1. ~~タスクトレイへの常駐と、グローバルホットキー~~（[#4](https://github.com/kaorinstar/driveshot/issues/4)）— 完了。
2. ~~画面の一部の取り込み。ローカル保存のみで、アップロードはしない~~（[#5](https://github.com/kaorinstar/driveshot/issues/5)）— 完了。
3. クラウドドライブ1つの一連の流れ。OAuth認証、アップロード、共有URLのクリップボードへのコピー（[#6](https://github.com/kaorinstar/driveshot/issues/6)）。
4. アップロード記録の保存と、保存期間経過後の削除（[#7](https://github.com/kaorinstar/driveshot/issues/7)）。
5. 残り2つのクラウドドライブへの対応（[#8](https://github.com/kaorinstar/driveshot/issues/8)）。
6. 設定の保存。保存期間、ホットキー、保存先（[#9](https://github.com/kaorinstar/driveshot/issues/9)）。

## 開発に参加する場合

大きな変更の前には、先にIssueの作成をお願いします。小規模なプロジェクトであり、未決定の設計事項は
`docs/architecture.ja.md` に記載しています。実装によって結論を出すプルリクエストは、議論が難しく
なります。

コード、コメント、識別子、コミットメッセージ、文書は英語で記述します。ファイル名が `.ja.md` で
終わるものは、隣にある同名ファイルの日本語訳で、同じコミットで更新します。

## セキュリティ

脆弱性は、Issueではなく
[GitHubのフォーム](https://github.com/kaorinstar/driveshot/security/advisories/new) から非公開で
ご報告ください。アプリが何に触れるか、今後何に触れる予定か、すでに把握している事項は
[SECURITY.ja.md](SECURITY.ja.md) に記載しています。

## ライセンス

[MIT](LICENSE)。
