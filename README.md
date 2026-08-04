# okane

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane?style=flat-square)](https://crates.io/crates/okane)

Okane is a plain text accounting software developed with Rust, influenced by [ledger-cli][ledger official].

This tool supports various commands:
* `balance` to get the current balance of the accounts.
* `register` to get the history of the accounts.
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
The format of YAML is (sorry) not documented, but you can see `tests/testdata` directory as example configurations.

Then run the `okane import` command with logging and redirecting to `/dev/null`. This way you can dry-run and check its output.

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv > /dev/null
```

After iterating over the logs and modifying YAML file, you can redirect the standard output to the ledger file.

```shell
$ RUST_LOG=info okane import --config ~/ledger/import.yml ~/ledger/input_file.csv >> ~/ledger/output_path.ledger
```

Tips: You probably don't want to handle all the entries, rather should aim to cover 80-90% of entries initially.

## License

This tool is licensed under [MIT lisence](LICENSE).

[ledger official]: https://github.com/ledger/ledger/
[ledger document]: https://ledger-cli.org/doc/ledger3.html
