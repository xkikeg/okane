# okane import

[日本語版はこちら](import.ja.md)

`okane import` generates a Ledger-format file out of files downloaded from banks and
credit card issuers, such as CSV statements.

```console
$ okane import --config path/to/import.yml path/to/statement.csv
```

The result is written to stdout, so redirect it to `/dev/null` first to check for
configuration errors, then append it to your ledger once it looks right.
`RUST_LOG=info` prints which rows were skipped and which rules matched.

There is also an interactive review mode, which shows every generated transaction in a
TUI and lets you fix the account before anything is written:

```console
$ okane import --config path/to/import.yml --interactive \
    --ledger path/to/main.ledger --output path/to/output.ledger \
    path/to/statement.csv
```

`--interactive` requires both `--ledger` (the ledger whose account names feed the
autocomplete) and `--output` (the file the accepted transactions are appended to).

## Configuration

This command requires a YAML config file, so this document describes how to write one.

In practice you may find it easier to start from the
[test config file](../testdata/import/test_config.yml) and adapt it. This document is
mainly a reference for when you cannot remember what a given setting meant.

### Overview of the config file

The config file is a YAML file. There is currently no requirement on its file name.

You write one config entry per group of input files to import. So a config file as a
whole looks like:

```yaml
path: foo/
...
---
path: bar/
...
```

that is, several settings each containing a `path`, separated by `---`. Besides `path`,
the following settings exist.

* `path`: **required**. Specifies a part of the input file path. Currently it is a
  plain substring comparison; regular expressions and glob patterns are not supported.
* `encoding`: **required**. The character encoding of the target file, such as `UTF-8`
  or `Shift_JIS`. The accepted strings are the ones
  [supported](https://encoding.spec.whatwg.org/) by encoding_rs.
* `account`: **required**. The account name used once loaded into Ledger
  (e.g. `Assets:Banks:Tanuki` or `Liabilities:Card:Kitsune`).
* `account_type`: **required**. Whether the account is an asset or a liability, given as
  `asset` or `liability`. A bank account is `asset`, a credit card is `liability`.
* `operator`: optional. The string recorded as the counterparty of any fee incurred by
  the transaction. Not needed for services that never charge fees.
* `charge_account`: optional. The account fees and commissions are posted to. Defaults
  to `Expenses:Commissions` when unset.
* `template`: optional, defaults to `false`. Set `true` to make this entry a template
  fragment, meaning it can only contribute values through merging and can never be the
  sole match for a file. See [merging](#merging-multiple-entries) below.
* `commodity`: **required**. Settings about the commodity. Covered later, it is long.
* `format`: optional. Settings about the specification and format of the input file.
  Covered later, it is long.
* `output`: optional. Settings about the emitted Ledger file. Covered later, it is long.
* `rewrite`: optional. Describes the rules matched while loading. This is the most
  important setting, so it is explained later.

Writing that once per input is all that is needed. Note the "required" attributes only
have to be present in the *merged* result, not in every fragment.

### Merging multiple entries

You often want to reuse settings across several files. For that, all entries whose
`path` matches are merged and then applied. Entries are merged in order of increasing
match length, so more specific paths win. `rewrite` rules are appended, and the other
settings are merged sensibly (a later, more specific value overrides an earlier one).

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

An entry with `template: true` participates in this merge but cannot stand on its own:
if every fragment matching a file is a template, the file is treated as unmatched. This
is handy for putting shared defaults under `path: ""`, which matches everything:

```yaml
path: ""
template: true
output:
  commodity:
    default:
      scale: 2
```

### Commodity (currency) settings

In economics a commodity means a good traded on the futures market, but in Ledger
terminology it refers to anything whose value is determined solely by its quantity,
including currencies and stocks. In practice it is the currency.

Commodity settings are needed because CSV files in particular express amounts as bare
numbers, which often makes it impossible to tell Japanese yen from US dollars. The
simplest way to configure it is to name the currency with a string.

```yaml
commodity: JPY
```

Alternatively, more complex items can be given as a hash.

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

* `primary`: the main currency used inside the account. Equivalent to giving the string
  directly.
* `conversion`: the default values for the rate computation applied when a foreign
  currency transaction happens. Explained in detail later as a `rewrite` item.
* `hidden_fee`: the default hidden fee settings. Explained in detail later as a
  `rewrite` item.
* `rename`: replaces the commodity named by the hash key with the value. In this case
  it rewrites 米ドル into USD. Note that renaming happens on output, so `rewrite`
  matchers on `commodity` / `secondary_commodity` still see the original name
  ([#304](https://github.com/xkikeg/okane/issues/304)).

### format settings

The format settings describe the format of the input file.
An example looks like this:

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

The following attributes can be specified.

* `file_type`: the type of the input file, one of `CSV`, `TSV`, `ISO_CAMT053` or
  `VISECA`. If unset, it is guessed from the suffix: `.csv` means CSV and `.tsv` means
  TSV. For any other file it is mandatory to set this field.
* `date`: the format string for the date, in
  [`chrono::format::strftime`](https://docs.rs/chrono/latest/chrono/format/strftime/index.html)
  syntax. Required for CSV/TSV. `"%Y/%m/%d"` (year/month/day) is commonly used in Japan.
* `delimiter`: the delimiter of the file. As of 2024-07-30 it only takes effect for CSV
  files. It must be a single ASCII character.
* `fields`: describes the meaning of the columns of a CSV file. See below.
* `skip.head`: the number of lines skipped at the head of the file, before the header
  row is read. CSV only.
* `row_order`: `old_to_new` (the default) or `new_to_old`. With `new_to_old` the emitted
  transactions are reversed so that the output is in chronological order.

#### format.fields

Each key below is given either a number, a string, or a template. A number is treated as
a 1-origin column index; a string selects the column by matching the value in the first
row of the CSV, which is treated as the header. A hash of the form
`{template: "..."}` builds the value out of other fields (see the next section).

Either `amount`, or the `credit`/`debit` pair, must be set; `date` and `payee` are always
required.

* `date`: the date.
* `payee`: the payee, i.e. the other side of the transaction.
* `code`: a code that uniquely identifies the transaction. Emitted as the Ledger
  transaction code. A `(?P<code>...)` capture in a `rewrite` matcher takes precedence
  over this field.
* `category`: the type of the transaction. Not emitted directly, but usable in
  `rewrite` matchers and in templates.
* `note`: additional information. Emitted as a comment on the transaction.
* `amount`: the amount of the transaction. Its sign is interpreted according to
  `account_type`: on a `liability` account the sign is flipped.
* `credit`: the amount of the transaction when the account balance increases.
* `debit`: the amount of the transaction when the account balance decreases.
* `balance`: the balance of the account, emitted as a Ledger balance assertion.
* `commodity`: the commodity of the transaction. Falls back to `commodity.primary` when
  the column is absent.
* `rate`: the commodity (exchange) rate at the time of the transaction.
* `secondary_amount`: when the transaction is done in two commodities, the amount in the
  second one.
* `secondary_commodity`: the second commodity. See also `conversion` under `rewrite`.
* `charge`: the charge, commission or fee related to the transaction. Posted to
  `charge_account` (or `Expenses:Commissions`) with `operator` recorded as its payee, so
  `operator` must be set whenever this field can be non-zero.

#### Templates in format.fields

Instead of a column, a field can be built from a template:

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

Inside the template, `{name}` refers to another field key and `{1}` refers to the
1-origin column index. A template may only reference fields that are mapped to a plain
column: referring to another template, or to the field being rendered itself, is an
error.

### output settings

These settings specify the format of the Ledger output.

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

* `commodity`: settings for the commodity (currency).
    * `default`: the standard commodity setting. Used whenever `overrides` does not
      specify one.
        * `style`: the style of the number. `plain` for the usual form, `comma3_dot` for
          comma separation every three digits.
        * `scale`: the minimum number of digits below the decimal point that an amount
          should be shown with. For instance with `2`, an exact amount is still shown
          with two decimal places, as `1.00`.
    * `overrides`: per-commodity overrides, keyed by commodity name. Fields left unset
      fall back to `default`.

### rewrite rules

The rewrite rules are the most important part of this config file. They make the actual
transactions automatically get assigned the account they belong to and the party they
were made with.

Example:

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

* `matcher`: **required**. The condition under which this rule applies. Written as a
  hash with the attributes described just below, or as a list of such hashes. The
  attributes of a hash are a logical AND: the rule does not apply unless all of them
  match. In the list form, the elements are a logical OR: the rule applies as soon as
  one of them matches. All values are regular expressions, matched **case
  insensitively** and not anchored, so `payee: Migros` also matches `MIGROS SUPERMARKT`.
    * `domain_code`, `domain_family`, `domain_sub_family`: valid only for the ISO
      Camt053 format. Selects the entries whose respective transaction codes match.
      These are exact code values, not regular expressions.
    * `creditor_name`, `creditor_account_id`, `ultimate_creditor_name`: valid only for
      the ISO Camt053 format. Matches the name or ID of the crediting side by regexp.
    * `debtor_name`, `debtor_account_id`, `ultimate_debtor_name`: valid only for the ISO
      Camt053 format. Matches the name or ID of the debiting side by regexp.
    * `remittance_unstructured_info`, `additional_entry_info`,
      `additional_transaction_info`: valid only for the ISO Camt053 format. Matches the
      corresponding field by regexp.
    * `category`: valid only for CSV/viseca. The category of the transaction, i.e. the
      value of the column given as `category` in `fields`. Matched by regexp.
    * `commodity`, `secondary_commodity`: the commodity and the secondary commodity of
      the transaction, matched by regexp. Useful for applying a `hidden_fee` only to a
      specific currency pair. These see the name *before* `commodity.rename` is applied.
    * `payee`: the name of the counterparty of this transaction. A regular expression
      can be given. If an earlier rule overwrote it, that value applies.
* `account`: sets the account to the given string when the rule applies.
* `payee`: sets the `payee` (the counterparty) to the given string when the rule
  applies.
* `pending`: when set to `true`, the posting to `account` gets marked as pending (`!`).
  Only effective together with `account`; see the note at the end of this section.
* `conversion`: specifies the exchange rate of the commodity (currency). Only effective
  when a transaction spans two currencies. The following items can be set.
    * `amount`: how the amount on the second (foreign) currency side is computed. The
      default is `extract`, which reads the value written in the field given as
      `secondary_amount`. When `compute` is given, it is computed from the rate.
    * `commodity`: specifies the second (foreign) currency of the transaction. If
      unspecified, the value of the `secondary_commodity` field given in `fields` is
      used.
    * `rate`: specifies in which direction the value given by the `rate` field points.
      By default it is `price_of_secondary`, i.e. the rate of the second currency
      expressed in the first one (`1 $secondary_commodity = $rate $commodity`). When
      `price_of_primary` is given, it is the reverse: the rate of the first currency
      expressed in the second one (`1 $commodity = $rate $secondary_commodity`).
    * `disabled`: set `true` to disable all conversions for the matched transactions,
      even when the input has a rate and a secondary amount.
* `hidden_fee`: specifies a fee that is not charged explicitly, but is instead baked
  into the exchange rate advertised by the operator as a spread. When set, okane
  recovers the real rate and books the difference as a commission to `charge_account`,
  so `operator` must be set as well. The following items can be set.
    * `spread`: the spread, given either as a percentage (`1.8%`) or as a fixed amount
      in the same commodity as the rate (`0.15 JPY`). Leaving it unset disables the
      hidden fee.
    * `condition`: when the hidden fee is incurred. `ALWAYS_INCURRED` (the default)
      assumes the operator takes the spread in both directions, so the sign of the
      transaction decides whether the real rate is above or below the advertised one.
      `DEBIT_ONLY` always assumes the rated posting is an expense, which is what you
      want for a credit card, where almost every "income" is a reimbursement priced at
      the original rate.

When several matchers match, they are applied in list order, earliest first. Matching
partway through does not stop the later matchers from being applied. Also, if the
regular expression has a named group, that submatch overwrites the payee (the name of
the counterparty) when it is named `payee`, and the transaction code when it is named
`code`.

Every field is only overwritten when a later matching rule actually sets it, so a rule
that matches but leaves `account` unset keeps the account chosen by an earlier rule.

`pending` marks the destination posting with `!`, and it only takes effect on a rule
that also sets `account`; putting `pending: true` on a rule without an `account` does
nothing. A transaction that no rule assigned an account to falls back to
`Expenses:Unknown` (or `Income:Unknown`), is marked pending, and okane logs a warning.
