# okane オカネ

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane?style=flat-square)](https://crates.io/crates/okane)

okane （オカネ）は [ledger-cli][ledger official]フォーマットに準拠したプレーンテキスト帳簿アプリケーションです。

現在ツールが実装しているコマンドは下記のとおりです。
* `balance`: 各アカウント(口座)の残高を表示します。
* `register`: 指定口座の変動を表示します。仕訳帳に相当します。
* `ui`: 残高と仕訳帳をターミナルUI上で対話的に表示します。
* `accounts`: アカウント(口座)の一覧を表示します。
* `tags`: ファイル中のタグの一覧を表示します。
* `format`: Ledgerファイルのフォーマッターです。
* `import`: 色々なデータファイル (CSV, ISO Camt053 XML)をLedgerフォーマットに取り込みます。
* `primitive`: まだまともに使えるかよくわからないサブコマンドがまとめられています。

## 使用方法

注意: まだ開発中なので突然引数などが変わることがあります。

またサポートされている文法は[syntax](doc/syntax.md)のとおりです。

### インストール

バージョン0.19.0以降はバイナリのリリースがあるので`cargo binstall`が使えます。

```shell
$ cargo binstall okane
```

もしソースからコンパイルしたければ`cargo install`してください。

```shell
$ cargo install okane
```

### 各種クエリ

[ledger-cli][ledger document]同様のコマンドです。オプション等は少しあるので `--help` を見てみてください。

```shell
$ okane accounts /path/to/file.ledger
$ okane tags /path/to/file.ledger [--values]
$ okane balance /path/to/file.ledger
$ okane registry /path/to/file.ledger [optional account]
```

### 対話的なTUI

`okane ui` は残高レポートをターミナルUIで開きます。
コマンドを何度も実行することなく色んな情報を確認できます。

```shell
$ okane ui /path/to/file.ledger
```

[![okane ui のデモ](https://asciinema.org/a/tqcqRCXYuTNYGmC5.svg)](https://asciinema.org/a/tqcqRCXYuTNYGmC5)

最初の画面はフラットなアカウントごとの残高リストです。`t` でアカウントの親子関係を考慮したツリー表示に切り替わり、`space` で選択中のサブツリーを、`x` で全体を折りたたみます。`/`, `C-s`, `C-r` でアカウント名を検索でき (それぞれ Vim 風、Emacs 風になっています)、`Enter` で選択したアカウントの仕訳帳を開きます。`r` でファイルを読み込み直すので、UIを開いたままファイルを編集できます。終了は `q` です。`?` で各種操作を確認できます。

`balance` や `register` と同じ評価用オプションが使えるので、通貨換算や日付指定も可能です。

```shell
$ okane ui --price-db ~/ledger/prices.db -X CHF /path/to/file.ledger
$ okane ui --price-db ~/ledger/prices.db -X CHF --historical /path/to/file.ledger
$ okane ui --start 2024-01-01 --end 2025-01-01 /path/to/file.ledger
```

同じオプションは `.` で開くフォームから対話的に変更できます。換算先の通貨、日付範囲、価格DBを書き換えるとレポートが再計算され、選択位置などの状態はそのまま保たれます。現在有効なオプションはステータスバーに表示されます。

### format

```shell
$ okane format ~/ledger/account.ledger
```

現在は整形済みのテキストを標準出力に吐くだけになっています。近々inplaceの置換やdiffモード、recursiveオプションを実装したいと思っています。

### CSV / ISO Camt053 XMLファイルの取り込み

`import` コマンドでは各種取引明細ファイルからLedgerフォーマットに取り込むことができます。摘要欄などに正規表現でマッチして仕訳できるようになっています。

まず最初にYAML形式の設定ファイルを用意します。このファイルで取り込み時に関する設定を行います。この例ではファイルが `~/ledger` に保存されていると仮定します。
フォーマットは[import](doc/import.ja.md)に解説があります。実例は `testdata/import/` 以下のyamlを参考にしてください。

その状態でまずは `okane import` コマンドを実行してください。読み込まれたLedgerフォーマットのデータは標準出力に吐かれるので、まず `/dev/null` にリダイレクトすると設定のエラーを確認できます。

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv > /dev/null
```

ログを読んである程度満足するまで設定ファイルを編集します。終えたらledgerファイルに追記で書き込みます。

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv >> ~/ledger/output_path.ledger
```

Tips: 100%の自動化を目指すと無理が出るので、80~90%程度自動化できるといいやくらいに思ってると楽です。

#### 取り込み結果を対話的に確認する

`rewrite` ルールですべてを賄うのは難しいです。特に旅行中などわざわざルールに追加しなくても…と思うトランザクションもあるでしょう。そんなときは `--interactive` を付けると書き込む前に取り込み結果を確認・修正できます。アカウント名が補完されるので便利です。

```shell
$ okane import --config ~/ledger/import.yml --interactive \
    --ledger ~/ledger/account.ledger -o ~/ledger/account.ledger ~/ledger/input_file.csv
```

`--ledger` オプションと `--output` (略して `-o`) を指定する必要があるので気をつけてください。

[![okane import --interactive のデモ](https://asciinema.org/a/UbMyEIDPO4eMjNzY.svg)](https://asciinema.org/a/UbMyEIDPO4eMjNzY)

取り込まれた取引がルールの判定結果とともに一覧表示され、カーソル位置の取引がプレビュー欄に描画されます。`a` でそのまま確定、`e` で口座の入力ダイアログが開き、`s` でスキップします。`w` で確定した取引を `--output` に追記して終了、`q` は何も書き込まずに中断します。

## ライセンス

このツールは [MIT lisence](LICENSE) でライセンスされています。
作者はこのソフトウェアの使用上生じた問題については責任を負いかねます。

[ledger official]: https://github.com/ledger/ledger/
[ledger document]: https://ledger-cli.org/doc/ledger3.html
