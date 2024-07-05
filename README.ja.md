# okane オカネ

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane?style=flat-square)](https://crates.io/crates/okane)

okane （オカネ）は [ledger-cli][ledger official]フォーマットに準拠したプレーンテキスト帳簿アプリケーションです。

現在ツールが実装しているコマンドは下記のとおりです。
* `balance`: 各アカウント(口座)の残高を表示します。
* `register`: 指定口座の変動を表示します。仕訳帳に相当します。
* `accounts`: アカウント(口座)の一覧を表示します。
* `format`: Ledgerファイルのフォーマッターです。
* `import`: 色々なデータファイル (CSV, ISO Camt053 XML)をLedgerフォーマットに取り込みます。
* `primitive`: まだまともに使えるかよくわからないサブコマンドがまとめられています。

## 使用方法

注意: まだ開発中なので突然引数などが変わることがあります。

またサポートされている文法は[syntax](doc/syntax)のとおりです。

### インストール

まだバイナリのリリースはないので自分で`cargo install`してください。

```shell
$ cargo install okane
```

なんなら`git clone`して自分でビルドしたバイナリを使うくらいがおすすめです。

### 各種クエリ

[ledger-cli][ledger document]同様のコマンドです。\
ただオプション等はほぼないので今後に期待してください。

```shell
$ okane accounts /path/to/file.ledger
$ okane balance /path/to/file.ledger
$ okane registry /path/to/file.ledger [optional account]
```

### format

```shell
$ okane format ~/ledger/account.ledger
```

現在は整形済みのテキストを標準出力に吐くだけになっています。近々inplaceの置換やdiffモード、recursiveオプションを実装したいと思っています。

### CSV / ISO Camt053 XMLファイルの取り込み

`import` コマンドでは各種取引明細ファイルからLedgerフォーマットに取り込むことができます。摘要欄などに正規表現でマッチして仕訳できるようになっています。

まず最初にYAML形式の設定ファイルを用意します。このファイルで取り込み時に関する設定を行います。この例ではファイルが `~/ledger` に保存されていると仮定します。
フォーマットはまだなんの解説もありませんが、 `tests/testdata` 以下のyamlをとりあえず参考にしてください。

その状態でまずは `okane import` コマンドを実行してください。読み込まれたLedgerフォーマットのデータは標準出力に吐かれるので、まず `/dev/null` にリダイレクトすると設定のエラーを確認できます。

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv > /dev/null
```

ログを読んである程度満足するまで設定ファイルを編集します。終えたらledgerファイルに追記で書き込みます。

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv >> ~/ledger/output_path.ledger
```

Tips: 100%の自動化を目指すと無理が出るので、80~90%程度自動化できるといいやくらいに思ってると楽です。

## ライセンス

このツールは [MIT lisence](LICENSE) でライセンスされています。
作者はこのソフトウェアの使用上生じた問題については責任を負いかねます。

[ledger official]: https://github.com/ledger/ledger/
[ledger document]: https://ledger-cli.org/doc/ledger3.html
