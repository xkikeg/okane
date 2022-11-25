# okane オカネ

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane)](https://crates.io/crates/okane)

okane （オカネ）は [ledger-cli](https://github.com/ledger/ledger/)フォーマットに準拠したプレーンテキスト帳簿アプリケーションです。

現在のところこのソフトは純粋に作者の必要とする下の2機能だけ実装されていて、他は元実装の ledger-cli を利用する形を取っています。
*  `format` Ledgerファイルのフォーマッターです。
*  `import` 色々なデータファイル (CSV, ISO Camt053 XML)をLedgerフォーマットに取り込みます。

## 使用方法

注意: まだ開発中なので突然引数などが変わることがあります。

### format

```shell
cargo build
RUST_LOG=info ./target/debug/okane format ~/ledger/account.ledger
```

現在は整形済みのテキストを標準出力に吐くだけになっています。未対応の構文がまだ多いです。近々inplaceの置換やdiffモード、recursiveオプションを実装したいと思っています。

### CSV / ISO Camt053 XMLファイルの取り込み

`import` コマンドでは各種取引明細ファイルからLedgerフォーマットに取り込むことができます。摘要欄などに正規表現でマッチして仕訳できるようになっています。

まず最初にYAML形式の設定ファイルを用意します。このファイルで取り込み時に関する設定を行います。この例ではファイルが `~/ledger` に保存されていると仮定します。
フォーマットはまだドキュメントされていませんが、 `tests/testdata` 以下のyamlをとりあえず参考にしてください。

その状態でまずは `okane import` コマンドを実行してください。読み込まれたLedgerフォーマットのデータは標準出力に吐かれるので、まず `/dev/null` にリダイレクトすると設定のエラーを確認できます。

```shell
$ cargo build
$ RUST_LOG=info ./target/debug/okane import --config ~/ledger/import.yml ~/ledger/input_file.csv > /dev/null
```

ログを読んである程度満足するまで設定ファイルを編集します。終えたらledgerファイルに追記で書き込みます。

```shell
$ cargo build
$ RUST_LOG=info ./target/debug/okane import --config ~/ledger/import.yml ~/ledger/input_file.csv >> ~/ledger/output_path.ledger
```

Tips: 100%の自動化を目指すと無理が出るので、80~90%程度自動化できるといいやくらいに思ってると楽です。
