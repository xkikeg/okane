# okane import

`okane import` は CSV ファイルをはじめ、各種の銀行やカード会社からダウンロードしてきたファイルから Ledger 形式のファイルを作るコマンドです。
このコマンドを使用するには import 設定を書くことが必須になるので、まずその書き方を紹介します。

## 設定

### 設定ファイルの概要

設定ファイルはYAML形式のファイルです。今のところファイル名やディレクトリは何でもいいです。

設定ファイルはインポートする対象のファイル名ごとに書きます。そのため設定ファイルを全体でみると

```yaml
path: foo/
...
---
path: bar/
...
```

のように `path` を含む設定をいくつか書いて、`---` で区切った内容になっています。`path` 以外には次の設定があります。

* `path`: 必須属性: 入力ファイルのファイルパスの一部を指定します。部分文字列比較で、正規表現とかグロブパターンは使えません。
* `encoding`: 必須属性: 対象ファイルの文字エンコーディングを指定します。UTF-8を指定することがほとんどでしょうが、日本だとShift_JISと書くこともまだまだ多いです。
* `account`: 必須属性: Ledger に読み込まれたときのアカウント名を指定します。(例: `Assets:Banks:Tanuki` とか `Liabilities:Card:Kitsune` とか)
* `account_type` 必須属性: アカウントが資産か負債かを `asset`, `liability` の二値で指定します。大体は銀行なら `asset` に、クレカなら `liability` にします。
* `operator`: オプション属性: 取引の際に手数料が発生した場合、その手数料の支払先として登録される文字列を指定します。手数料の出てこないサービスなら指定不要です。
* `commodity`: 必須属性: コモディティについての設定です。長いので後回しにします。
* `format`: オプション属性: ファイルの仕様・書式についての設定です。長いので後回しにします。
* `rewrite`: オプション属性: 読み込み時のマッチングするルールについて記述します。一番大事な設定ですので後で解説します。

これを入力分だけ書けばいいということになります。ただ、複数のファイルで設定を使いまわしたいこともあると思います。そんなときのために設定は `path` がマッチする設定すべてをマージして適用することにしています。マッチした長さが短い順からマージしていき `rewrite` だけは追加、ほかは書き換えになっています。

### コモディティ(通貨)の設定

まずコモディティというのは Ledger 用語で、通貨や株式あるいはそれこそ商品のように代替可能でその価値が一単位あたり等しいものです。普通インポートする際に出てくるのは通貨、あっても有価証券でしょう。なので、基本的には通貨の設定と思ってもらって大丈夫です。

コモディティの設定がなんで必要かというと、特にCSVファイルは大体の場合金額がただの数字で書いてあって、日本円なのかUSドルなのかわからなくなるからです。一番カンタンに指定する方法は、ただ文字列でコモディティを指定します。

```yaml
commodity: JPY
```

ただ、今後の文法拡張のために次の記法でも書くことができます。意味の差はありません。

```yaml
commodity:
  primary: JPY
```

今後コモディティの書き換えなどを実装予定ですが、その機能を使うにはこの文法で書く必要があります。

### format(書式)の設定

書式設定では入力ファイルや出力の書式について設定します。例はこんな感じです。

```yaml
format:
  commodity:
    EUR:
      precision: 2
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

* `commodity`: コモディティとその精度`precision`をmapで指定します。私はUSDとかEURには`precision: 2`を、JPYには`precision: 0`を指定します。おいおいこの設定は変える予定です。
* `date`: 必須: 日付のフォーマット文字列を指定します。`"%Y/%m/%d"` (年/月/日)が日本ではよく使われると思います。
* `delimiter`: ファイルの区切り文字を指定します。2024-07-30時点ではCSVファイルにしか効果がありません。
* `fields`: CSVファイルで列の意味を記述します。数字の場合は0-originの列番号として、文字列の場合はCSVの一行目をheaderと考えてその値で列を指定できます。以下の項目に対して文字列ないし数字で列を指定します。
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

## rewriteルール

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

* `matcher`: 必須: このルールを適用する条件です。すぐ下で説明する属性を持つmapか、そのlistとして記述します。listの場合論理和(OR)になり、一つでも当てはまったら適用されます。
    * `domain_code`, `domain_family`, `domain_sub_family`: ISO Camt053フォーマットのみで有効です。定数だけが指定可能です。詳細は省きます。
    * `creditor_name`, `creditor_account_id`, `ultimate_creditor_name`: ISO Camt053フォーマットのみで有効です。正規表現が指定できます。詳細は省きます。
    * `debtor_name`, `debtor_account_id`, `ultimate_debtor_name`: ISO Camt053フォーマットのみで有効です。正規表現が指定できます。詳細は省きます。
    * `remittance_unstructured_info`, `additional_transaction_info`: ISO Camt053フォーマットのみで有効です。正規表現が指定できます。詳細は省きます。
    * `category`: CSV/visecaのみで有効です。取引の種類、`fields`で`category`として指定された列の文字列です。正規表現が指定できます。
    * `payee`: この取引の相手方の名前です。正規表現が指定できます。以前のルールで上書きされた場合その値が適用されます。
* `account`: ルールが適用された場合アカウントを指定された文字列にします。
* `payee`: ルールが適用された場合`payee`(相手方)を指定された文字列にします。
* `peinding`: `true`にした場合その取引が保留中のマークがつきます。
* `conversion`: コモディティ(通貨)の変換レートについて指定します。いくつかの指定方法があります。
    * `conversion: { commodity: foo }` マッチした取引の第二通貨(セカンダリ・コモディティ)を直接指定された値に設定します。
      `$amount $commodity == $amount / $rate $conversion_commodity`, `1 $conversion_commodity == $rate $commodity`です。
      TODO: Do we need to specify secondary_amount still? Or rate is used?
    * `conversion: {type: primary }`: `secondary_amount`と`rate`を第一通貨として扱います。つまり`$amount $commodity == $secondary_amount $primary_commodity`, `1 $commodity == $rate $primary_commodity`ということです。
    * `conversion: {type: secondary }`: `secondary_amount`を`secondary_commodity`として扱います。つまり、`$amount $commodity == $secondary_amount $secondary_commodity`, `1 $secondary_commodity = $rate $commodity`です。

このmatcherは複数マッチした場合listの順が早い方から適用されます。また、正規表現中に名前付きグループがあった場合、その部分マッチが`payee`なら相手方の名前に、`code`なら取引コードに設定されます。
