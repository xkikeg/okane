# okane import

`okane import` は CSV ファイルをはじめ、各種の銀行やカード会社からダウンロードしてきたファイルから Ledger 形式のファイルを生成するコマンドです。

## 設定

このコマンドを使用するにはYaML形式の設定ファイルが必須なので、まずはその書き方を紹介します。

ただ、実際に設定を書く際は[テスト用の設定ファイル](../cli/tests/testdata/import/test_config.yml)などを参考に書いていくのがいいかもしれません。このドキュメントは主にどんな意味の設定だったかわからなくなった時用のリファレンスです。

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
* `commodity`: 必須属性: コモディティについての設定です。長いので後回しにします。
* `format`: オプション属性: 入力ファイルの仕様・書式についての設定です。長いので後回しにします。
* `output`: オプション属性: 出力されるLedgerファイルについての設定です。長いので後回しにします。
* `rewrite`: オプション属性: 読み込み時のマッチングするルールについて記述します。一番大事な設定ですので後で解説します。

これを入力分だけ書けばいいということになります。ただ、複数のファイルで設定を使いまわしたいこともあると思います。そんなときのために設定は `path` がマッチする設定すべてをマージして適用することにしています。設定はマッチした長さが短い順からマージされます。`rewrite` の設定は追記、他の設定はいい感じにマージされると思ってください。

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
  rename:
    米ドル: USD
```

* `primary`: 口座内での主要通貨を指定します。文字列で指定するのと同じです。
* `conversion`: 外貨取引が行われた際のレート計算に関する設定のデフォルト値です。あとで`rewrite`の項目として詳しく説明します。
* `rename`: ハッシュのkeyとなる通貨をvalueに置き換えます。この場合米ドルをUSDに書き換えます。

### format(書式)の設定

書式設定では入力ファイルや出力の書式について設定します。例はこんな感じです。

```yaml
format:
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

* `date`: 必須: 日付のフォーマット文字列を指定します。`"%Y/%m/%d"` (年/月/日)が日本ではよく使われると思います。
* `delimiter`: ファイルの区切り文字を指定します。2024-07-30時点ではCSVファイルにしか効果がありません。
* `fields`: CSVファイルで列の意味を記述します。数字の場合は1-originの列番号として、文字列の場合はCSVの一行目をheaderと考えてその値で列を指定できます。以下の項目に対して文字列ないし数字で列を指定します。
    * `date`: 日付
    * `payee`: 受取/支払先
    * `category`: 取引の種類
    * `note`: 追記事項
    * `amount`: 取引の金額
    * `credit`: アカウント残高が増加する際の取引の金額
    * `debit`: アカウント残高が減少する際の取引の金額
    * `balance`: アカウントの残高
    * `commodity`: 取引のコモディティ
    * `rate`: 取引時のコモディティ(為替)レート
    * `secondary_amount`: 取引が2つのコモディティでされている際のその2つ目のコモディティでの金額。
    * `secondary_commodity`: 2つ目のコモディティ。rewriteの`conversion`も参照すること。

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
    type: primary
- account: Expenses:Grocery
  matcher:
  - payee: EURO GROCERY
  - payee: 山田商店

```

* `matcher`: 必須: このルールを適用する条件です。すぐ下で説明する属性を持つハッシュか、そのlistとして記述します。ハッシュの属性は論理積(AND)ですべてマッチしないと適用されません。listの場合要素同士は論理和(OR)になり、一つでも当てはまったら適用されます。
    * `domain_code`, `domain_family`, `domain_sub_family`: ISO Camt053フォーマットのみで有効です。取引の各コードが一致するものを選択します。
    * `creditor_name`, `creditor_account_id`, `ultimate_creditor_name`: ISO Camt053フォーマットのみで有効です。正規表現で支払側の名前やIDにマッチします。
    * `debtor_name`, `debtor_account_id`, `ultimate_debtor_name`: ISO Camt053フォーマットのみで有効です。正規表現で受け取り側の名前やIDにマッチします。
    * `remittance_unstructured_info`, `additional_entry_info`, `additional_transaction_info`: ISO Camt053フォーマットのみで有効です。正規表現で対応するフィールドにマッチします。
    * `category`: CSV/visecaのみで有効です。取引のカテゴリで、`fields`で`category`として指定された列の値です。正規表現でマッチします。
    * `payee`: この取引の相手方の名前です。正規表現が指定できます。以前のルールで上書きされた場合その値が適用されます。
* `account`: ルールが適用された場合アカウントを指定された文字列にします。
* `payee`: ルールが適用された場合`payee`(相手方)を指定された文字列にします。
* `pending`: `true`にした場合その取引が保留中のマークがつきます。
* `conversion`: コモディティ(通貨)の為替レートについて指定します。取引が2つの通貨をまたいで行われた際にのみ有効です。次の項目を設定できます。
    * `amount`: 第二通貨(外貨)側での金額の計算方法を指定します。デフォルトでは`extract`で、フィールドとして指定された`secondary_amount`に書かれた値を読み込みます。`compute`が指定されたときはレートから計算します。
    * `commodity`: 取引の第二通貨(外貨)を指定できます。指定がなければfieldsで指定された`secondary_commodity`の値を使用します。
    * `rate`: `rate`フィールドで指定された値がどちら向きの値なのかを指定します。標準では`price_of_secondary`、つまり第二通貨のレートを第一通貨で指定します。(`1 $secondary_commodity = $rate $comodity`)。`price_of_primary`が指定された場合、逆に第一通貨のレートを第二通貨で指定します。(`1 $commodity = $rate $secondary_commodity`)

このmatcherは複数マッチした場合listの順が早い方から適用されます。途中でマッチしてもそれ以降のmatcherは適用されます。また、正規表現中に名前付きグループがあった場合、その部分マッチが`payee`ならpayee(相手方の名前)が、`code`なら取引コードが上書きされます。

