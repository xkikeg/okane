# okane import

[English version here](import.md)

`okane import` は CSV ファイルをはじめ、各種の銀行やカード会社からダウンロードしてきたファイルから Ledger 形式のファイルを生成するコマンドです。

```console
$ RUST_LOG=info okane import --config path/to/import.yml path/to/statement.csv
```

結果は標準出力に書き出されるので、まず `/dev/null` にリダイレクトしつつログで設定の漏れを確認し、問題なさそうならLedgerファイルに追記するのがよいでしょう。`RUST_LOG=info` でマッチするルールがなかったデータが分かります。

生成された取引をTUIで一件ずつ確認し、書き出す前にアカウントを修正できる対話モードもあります。

```console
$ okane import --config path/to/import.yml --interactive \
    --ledger path/to/main.ledger --output path/to/output.ledger \
    path/to/statement.csv
```

`--interactive` を使う場合、`--ledger` (補完に使うアカウント名を読み込むLedgerファイル) と `--output` (確定した取引が追記されるファイル) の両方が必須です。

## 設定

このコマンドを使用するにはYaML形式の設定ファイルが必須なので、まずはその書き方を紹介します。

ただ、実際に設定を書く際は[テスト用の設定ファイル](../testdata/import/test_config.yml)などを参考に書いていくのがいいかもしれません。このドキュメントは主にどんな意味の設定だったかわからなくなった時用のリファレンスです。

### 設定ファイルの概要

設定ファイルはYAML形式のファイルです。今のところファイル名に特段の指定はありません。

設定ファイルはインポートする対象のファイル名ごとに書きます。そのため設定ファイルを全体でみると

```yaml
path: foo/
...
---
path: bar/
...
```

のように `path` を含む設定をいくつか書いて、`---` で区切った内容になっています。`path` 以外には次の設定があります。

* `path`: 必須属性: 入力ファイルのファイルパスの一部を指定します。現時点では部分文字列比較で、正規表現とかグロブパターンは使えません。
* `encoding`: 必須属性: 対象ファイルの文字エンコーディングを指定します。`UTF-8`や`Shift_JIS`などが指定できます。指定できる文字列はencoding_rsの[サポートするもの](https://encoding.spec.whatwg.org/)です。
* `account`: 必須属性: Ledger に読み込まれたときのアカウント名を指定します。(例: `Assets:Banks:Tanuki` とか `Liabilities:Card:Kitsune` とか)
* `account_type` 必須属性: アカウントが資産か負債かを `asset`, `liability` の二値で指定します。例えば銀行なら `asset` に、クレカなら `liability` にします。
* `operator`: オプション属性: 取引の際に手数料が発生した場合、その手数料の支払先として登録される文字列を指定します。手数料の出てこないサービスなら指定不要です。
* `charge_account`: オプション属性: 手数料が計上されるアカウントを指定します。指定がない場合 `Expenses:Commissions` が使われます。
* `template`: オプション属性 (デフォルトは `false`): `true` にするとそのエントリはテンプレートとなり、マージで値を提供することしかできず、単独でファイルにマッチすることはできなくなります。後述の[複数エントリのマージ](#複数エントリのマージ)を参照してください。
* `commodity`: 必須属性: コモディティについての設定です。長いので後回しにします。
* `format`: オプション属性: 入力ファイルの仕様・書式についての設定です。長いので後回しにします。
* `output`: オプション属性: 出力されるLedgerファイルについての設定です。長いので後回しにします。
* `rewrite`: オプション属性: 読み込み時のマッチングするルールについて記述します。一番大事な設定ですので後で解説します。

これを入力分だけ書けばいいということになります。なお「必須属性」はマージ後の結果に存在していればよく、個々のエントリすべてに書く必要はありません。

### 複数エントリのマージ

複数のファイルで設定を使いまわしたいこともあると思います。そんなときのために設定は `path` がマッチする設定すべてをマージして適用することにしています。設定はマッチした長さが短い順からマージされます。`rewrite` の設定は追記、他の設定は last win でマージされると思ってください。

```yaml
path: foo/
encoding: UTF-8
account_type: asset
commodity: JPY
format:
  ...
rewrite:
- ...
---
path: foo/bar/
commodity: USD
rewrite:
- ...
```

`template: true` が指定されたエントリはこのマージには参加しますが、単独では成立しません。あるファイルにマッチしたエントリがすべてテンプレートだった場合、そのファイルはマッチする設定が見つからなかったとしてエラーになります。すべてにマッチする `path: ""` に共通のデフォルト値を置く際に便利です。

```yaml
path: ""
template: true
output:
  commodity:
    default:
      scale: 2
```

### コモディティ(通貨)の設定

コモディティというのは経済学では先物取引の商品を指すようですが、 Ledger 用語では、通貨や株式も含めて個数だけで値段が決まるものを指します。とどのつまりは通貨です。
コモディティの設定がなぜ必要かというと、特にCSVファイルは金額が単位のない数で表されていて、日本円なのかUSドルなのか判別つきかねる事が多いからです。
設定する際、最も簡単な方法は文字列で通貨を指定することです。

```yaml
commodity: JPY
```

一方でより複雑な項目をハッシュで指定することも可能です。

```yaml
commodity:
  primary: JPY
  conversion:
    amount: *extract|compute
    commodity: string
    rate: *price_of_secondary|price_of_primary
    disabled: *false|true
  hidden_fee:
    spread: 1.8%
    condition: *ALWAYS_INCURRED|DEBIT_ONLY
  rename:
    米ドル: USD
```

* `primary`: 口座内での主要通貨を指定します。文字列で指定するのと同じです。
* `conversion`: 外貨取引が行われた際のレート計算に関する設定のデフォルト値です。あとで`rewrite`の項目として詳しく説明します。
* `hidden_fee`: 隠れ手数料の設定のデフォルト値です。あとで`rewrite`の項目として詳しく説明します。
* `rename`: ハッシュのkeyとなる通貨をvalueに置き換えます。この場合米ドルをUSDに書き換えます。なお置き換えは出力時に行われるので、`rewrite`の`commodity`/`secondary_commodity` matcherが見るのは置き換え前の名前です ([#304](https://github.com/xkikeg/okane/issues/304))。

### format(書式)の設定

書式設定では入力ファイルの書式について設定します。例はこんな感じです。

```yaml
format:
  file_type: CSV
  date: "%Y/%m/%d"
  delimiter: ";"
  fields:
    date: お取り引き日
    payee: 摘要
    debit: 出金額
    credit: 入金額
    balance: 残高
    commodity: 通貨
    rate: 適用レート
    secondary_amount: 取引円換算額
  skip:
    head: 10
  row_order: new_to_old
```

下記の属性が指定できます。

* `file_type`: 入力ファイルの種別で、`CSV`, `TSV`, `ISO_CAMT053`, `VISECA` のいずれかです。指定がない場合は拡張子から推測され、`.csv` ならCSV、`.tsv` ならTSVとなります。それ以外のファイルではこの属性の指定が必須です。
* `date`: 日付のフォーマット文字列を [`chrono::format::strftime`](https://docs.rs/chrono/latest/chrono/format/strftime/index.html) の書式で指定します。CSV/TSVでは必須です。`"%Y/%m/%d"` (年/月/日)が日本ではよく使われると思います。
* `delimiter`: ファイルの区切り文字を指定します。2024-07-30時点ではCSVファイルにしか効果がありません。ASCII1文字である必要があります。
* `fields`: CSVファイルで列の意味を記述します。詳細は後述します。
* `skip.head`: ヘッダ行を読み込む前に、ファイル先頭で読み飛ばす行数を指定します。CSVのみで有効です。
* `row_order`: `old_to_new` (デフォルト) または `new_to_old` を指定します。`new_to_old` の場合、出力が日付順になるように取引の順序を反転します。

#### format.fields の設定

以下の各項目には数字、文字列、あるいはテンプレートを指定します。数字の場合は1-originの列番号として、文字列の場合はCSVの一行目をheaderと考えてその値で列を指定できます。`{template: "..."}` の形のハッシュを指定すると他のフィールドから値を組み立てられます(次節参照)。

`amount` または `credit`/`debit` のペアのどちらかが必須です。また `date` と `payee` は常に必須です。

* `date`: 日付
* `payee`: 受取/支払先
* `code`: 取引を一意に識別するコード。Ledgerの取引コードとして出力されます。`rewrite`のmatcherで `(?P<code>...)` がキャプチャされた場合はそちらが優先されます。
* `category`: 取引の種類。直接は出力されませんが、`rewrite`のmatcherやテンプレートから参照できます。
* `note`: 追記事項。取引のコメントとして出力されます。
* `amount`: 取引の金額。符号は `account_type` に従って解釈され、`liability` の口座では符号が反転します。
* `credit`: アカウント残高が増加する際の取引の金額
* `debit`: アカウント残高が減少する際の取引の金額
* `balance`: アカウントの残高。Ledgerの残高アサーションとして出力されます。
* `commodity`: 取引のコモディティ。列がない場合は `commodity.primary` にフォールバックします。
* `rate`: 取引時のコモディティ(為替)レート
* `secondary_amount`: 取引が2つのコモディティでされている際のその2つ目のコモディティでの金額。
* `secondary_commodity`: 2つ目のコモディティ。rewriteの`conversion`も参照すること。
* `charge`: 取引に関連する手数料。`charge_account` (指定がなければ `Expenses:Commissions`) に、`operator` を支払先として計上されます。そのためこの列が非ゼロになりうる場合は `operator` の指定が必須です。

#### format.fields のテンプレート

列の代わりに、テンプレートからフィールドを組み立てることもできます。

```yaml
format:
  fields:
    date: Date
    category: Action
    note: Description
    payee:
      template: "{category} - {note}"
    amount: Amount
```

テンプレート内では `{name}` で他のフィールドキーを、`{1}` で1-originの列番号を参照できます。参照できるのは単純な列に対応付けられたフィールドだけで、他のテンプレートや、レンダリング中のフィールド自身を参照するとエラーになります。

### outputの設定

この設定ではLedgerの出力に関する書式を指定します。

```yaml
output:
  commodity:
    default:
      style: comma3_dot
      scale: null
    overrides:
      EUR:
        style: plain
        scale: 2
```

* `commodity`: コモディティ(通貨)の設定です。
    * `default`: 標準のコモディティ設定です。`overrides`で指定されない場合この設定が使われます。
        * `style`: 数値のスタイルを指定します。`plain`で通常、`comma3_dot`で3桁コンマ区切りです。
        * `scale`: 金額が最低でも小数点以下何桁まで表示されてほしいかを指定します。
          例えば`2`の場合`1.00`のようにピッタリでも小数点2桁で表示されます。
    * `overrides`: コモディティ名をkeyとした個別の上書き設定です。指定されなかった項目は `default` にフォールバックします。

### rewriteルール

rewriteルールはこの設定ファイルで一番大事な部分で、実際の取引がどのアカウントに属するのか、誰との取引なのかが自動で指定されるようにします。

例

```yaml
rewrite:
- matcher:
    payee: ^Visaデビット　(?P<code>\d+)　(?P<payee>.*)$
- account: Assets:Wire
  matcher:
    payee: 円普通預金(へ|より)振替
  conversion:
    commodity: JPY
    rate: price_of_primary
- account: Expenses:Grocery
  matcher:
  - payee: EURO GROCERY
  - payee: 山田商店
```

* `matcher`: 必須: このルールを適用する条件です。すぐ下で説明する属性を持つハッシュか、そのlistとして記述します。ハッシュの属性は論理積(AND)ですべてマッチしないと適用されません。listの場合要素同士は論理和(OR)になり、一つでも当てはまったら適用されます。値はすべて正規表現で、**大文字小文字を区別せず**、また部分一致でマッチします。つまり `payee: Migros` は `MIGROS SUPERMARKT` にもマッチします。
    * `domain_code`, `domain_family`, `domain_sub_family`: ISO Camt053フォーマットのみで有効です。取引の各コードが一致するものを選択します。これらは正規表現ではなくコードそのものの一致です。
    * `creditor_name`, `creditor_account_id`, `ultimate_creditor_name`: ISO Camt053フォーマットのみで有効です。正規表現で支払側の名前やIDにマッチします。
    * `debtor_name`, `debtor_account_id`, `ultimate_debtor_name`: ISO Camt053フォーマットのみで有効です。正規表現で受け取り側の名前やIDにマッチします。
    * `remittance_unstructured_info`, `additional_entry_info`, `additional_transaction_info`: ISO Camt053フォーマットのみで有効です。正規表現で対応するフィールドにマッチします。
    * `category`: CSV/visecaのみで有効です。取引のカテゴリで、`fields`で`category`として指定された列の値です。正規表現でマッチします。
    * `commodity`, `secondary_commodity`: 取引のコモディティと第二コモディティに正規表現でマッチします。特定の通貨ペアにだけ `hidden_fee` を適用したい場合に便利です。`commodity.rename` の適用**前**の名前を見ることに注意してください。
    * `payee`: この取引の相手方の名前です。正規表現が指定できます。以前のルールで上書きされた場合その値が適用されます。
* `account`: ルールが適用された場合アカウントを指定された文字列にします。
* `payee`: ルールが適用された場合`payee`(相手方)を指定された文字列にします。
* `pending`: `true`にした場合 `account` へのpostingに保留中のマーク (`!`) がつきます。`account`と併せて指定したときのみ有効です。この節の最後の注記も参照してください。
* `conversion`: コモディティ(通貨)の為替レートについて指定します。取引が2つの通貨をまたいで行われた際にのみ有効です。次の項目を設定できます。
    * `amount`: 第二通貨(外貨)側での金額の計算方法を指定します。デフォルトでは`extract`で、フィールドとして指定された`secondary_amount`に書かれた値を読み込みます。`compute`が指定されたときはレートから計算します。
    * `commodity`: 取引の第二通貨(外貨)を指定できます。指定がなければfieldsで指定された`secondary_commodity`の値を使用します。
    * `rate`: `rate`フィールドで指定された値がどちら向きの値なのかを指定します。標準では`price_of_secondary`、つまり第二通貨のレートを第一通貨で指定します。(`1 $secondary_commodity = $rate $comodity`)。`price_of_primary`が指定された場合、逆に第一通貨のレートを第二通貨で指定します。(`1 $commodity = $rate $secondary_commodity`)
    * `disabled`: `true`にすると、入力にレートと第二通貨の金額があってもマッチした取引の変換をすべて無効にします。
* `hidden_fee`: 明示的な手数料としてではなく、事業者が提示する為替レートにスプレッドとして埋め込まれた「隠れ手数料」を指定します。設定すると okane は本来のレートを逆算し、その差額を手数料として `charge_account` に計上するため、`operator` の指定も必要になります。次の項目を設定できます。
    * `spread`: スプレッドを、パーセント (`1.8%`) またはレートと同じコモディティの固定額 (`0.15 JPY`) で指定します。指定しない場合、隠れ手数料は無効になります。
    * `condition`: 隠れ手数料が発生する条件です。`ALWAYS_INCURRED` (デフォルト) は事業者が両方向でスプレッドを取ると仮定し、取引の符号によって本来のレートが提示レートより上か下かを判断します。`DEBIT_ONLY` はレートのついた記帳を常に支出とみなします。クレジットカードでは「収入」のほぼすべてが元のレートでの払い戻しなので、こちらが適切です。

このmatcherは複数マッチした場合listの順が早い方から適用されます。途中でマッチしてもそれ以降のmatcherは適用されます。また、正規表現中に名前付きグループがあった場合、その部分マッチが`payee`ならpayee(相手方の名前)が、`code`なら取引コードが上書きされます。

どの項目も、後続のマッチしたルールが実際に値を設定したときにだけ上書きされます。そのため、マッチしたものの `account` を設定していないルールがあっても、以前のルールが決めたアカウントはそのまま残ります。

`pending` は相手方のpostingに `!` を付けるもので、`account` も設定しているルールでのみ有効です。`account` のないルールに `pending: true` を書いても何も起きません。どのルールでもアカウントが決まらなかった取引は `Expenses:Unknown` (または `Income:Unknown`) にフォールバックし、保留中となり、okane は警告をログに出します。
