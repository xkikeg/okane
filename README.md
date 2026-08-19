# okane

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane?style=flat-square)](https://crates.io/crates/okane)

Okane is a plain text accounting software developed with Rust, influenced by [ledger-cli][ledger official].

This tool supports various commands:
* `balance` to get the current balance of the accounts.
* `register` to get the history of the accounts.
* `ui` to browse the balance and the register in an interactive terminal UI.
* `accounts` to list all accounts in the file.
* `tags` to list all tags in the file.
* `format` to format given Ledger file into organized format.
* `import` to convert various source including CSV, ISO Camt053 XML into Ledger format.
* `primitive` to hold commands that are not so useful but good for debugging.

Note `balance`, `register` are still work-in-progress, and the UX would change drastically.

## How to use

Disclaimer: This software is still in early phase, subject to any kind of change.

Follows [syntax](doc/syntax.md) page for the supported syntax.

### Install

You can use `cargo binstall` to install the latest binary (later than v0.19.0).

```shell
$ cargo binstall okane
```

Of course, you can build your own binary if you want.

```shell
$ cargo install okane
```

### Query the file

Similar to [Ledger][ledger document], you can use similar commands.

```shell
$ okane accounts /path/to/file.ledger
$ okane tags /path/to/file.ledger [--values]
$ okane balance /path/to/file.ledger
$ okane registry /path/to/file.ledger [optional account]
```

### Browse the file interactively

`okane ui` opens the balance report in a terminal UI, so you can explore the
accounts instead of re-running queries.

```shell
$ okane ui /path/to/file.ledger
```

[![okane ui demo](https://asciinema.org/a/tqcqRCXYuTNYGmC5.svg)](https://asciinema.org/a/tqcqRCXYuTNYGmC5)

The balance screen starts with a flat balance view. `t` switches to a tree of
accounts, where `space` folds the selected subtree and `x` folds everything.
`/` searches the account names in Vim style (`n` / `N` walk the matches) while
`C-s`, `C-r` is Emacs style search., `Enter` opens the register of the
selected account, and `r` reloads the file from disk so you can keep the UI open
while editing. `q` quits. `?` lists every key binding of the screen you are on.

It takes the same evaluation flags as `balance` and `register`, so the amounts
can be converted into a single commodity, or filtered by the time range:

```shell
$ okane ui --price-db ~/ledger/prices.db -X CHF /path/to/file.ledger
$ okane ui --price-db ~/ledger/prices.db -X CHF --historical /path/to/file.ledger
$ okane ui --start 2024-01-01 --end 2025-01-01 /path/to/file.ledger
```

`.` opens those same options in a form, so you can change them without leaving
the UI: pick a commodity to convert into, bound the dates, or point at a
different price DB, and the report is recomputed with your place in it kept.
Whichever ones are in effect are shown in the status bar.

### Format the file

```shell
$ okane format ~/ledger/account.ledger
$ okane fmt ~/ledger/account.ledger      # same thing
```

This rewrites the given file in place, together with every file it pulls in through
`include` directives.

How an amount is printed is decided per commodity, over the whole set of files: the
`commodity ... format` directives are honoured wherever they are declared, and a
commodity without one follows the amounts written for it, so that the same commodity
looks the same everywhere.

```ledger
commodity JPY
    format 1,000 JPY
```

With `--check` it writes nothing and instead prints a unified diff of the changes it
would make, exiting with `1` if any file is not formatted. That makes it usable as a
Git hook or a CI check:

```shell
$ okane format --check ~/ledger/account.ledger || echo "run okane format"
```

To format a single file and print the result to standard output instead, without
following `include`:

```shell
$ okane primitive format ~/ledger/account.ledger
```

That one does not look for the commodity settings at all, and prints every amount the
way it is written, unless it is told how to:

```shell
$ okane primitive format --commodity-format 'CHF=1,000.00' ~/ledger/account.ledger
```

### Import CSV or ISO Camt053 XML files

First you need to write YAML file to control import behavior. We'll assume those are placed under `~/ledger/`.
See the [import](doc/import.md) page for the format,
and the `testdata/import/` directory for example configurations.

Then run the `okane import` command with logging and redirecting to `/dev/null`. This way you can dry-run and check its output.

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv > /dev/null
```

After iterating over the logs and modifying YAML file, you can redirect the standard output to the ledger file.

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv >> ~/ledger/output_path.ledger
```

Tips: You probably don't want to handle all the entries, rather should aim to cover 80-90% of entries initially.

#### Reviewing the import interactively

Since the rewrite rules never cover everything, `--interactive` lets you review
the transactions before they are written.

```shell
$ okane import --config ~/ledger/import.yml --interactive \
    --ledger ~/ledger/account.ledger -o ~/ledger/account.ledger ~/ledger/input_file.csv
```

Note that it requires both `--ledger` and `--output` options.

[![okane import --interactive demo](https://asciinema.org/a/UbMyEIDPO4eMjNzY.svg)](https://asciinema.org/a/UbMyEIDPO4eMjNzY)

Every transaction is listed with the account the rules picked for it, and the
one under the cursor is rendered in the preview pane. `a` accepts it as-is,
`e` opens an account completion dialog over the accounts of `--ledger`,
and `s` skips it. `w` appends everything you accepted to `--output`; `q` aborts
without writing anything.

## License

This tool is licensed under [MIT lisence](LICENSE).

[ledger official]: https://github.com/ledger/ledger/
[ledger document]: https://ledger-cli.org/doc/ledger3.html
