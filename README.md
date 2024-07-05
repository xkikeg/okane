# okane

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane?style=flat-square)](https://crates.io/crates/okane)

Okane is a plain text accounting software developed with Rust, influenced by [ledger-cli][ledger official].

This tool supports various commands:
* `balance` to get the current balance of the accounts.
* `register` to get the history of the accounts.
* `accounts` to list all accounts in the file.
* `format` to format given Ledger file into organized format.
* `import` to convert various source including CSV, ISO Camt053 XML into Ledger format.
* `primitive` to hold commands that are not so useful but good for debugging.

Note `balance`, `register` are still work-in-progress, and the UX would change drastically.

## How to use

Disclaimer: This software is still in early phase, subject to any kind of change.

Follows [syntax](doc/syntax.md) page for the supported syntax.

### Install

Up until now no binary release is provided, so you need to run `cargo install` to install the tool.

```shell
$ cargo install okane
```

### Query the file

Similar to [Ledger][ledger document], you can use similar commands.

```shell
$ okane accounts /path/to/file.ledger
$ okane balance /path/to/file.ledger
$ okane registry /path/to/file.ledger [optional account]
```

### Format the file

```shell
$ okane format ~/ledger/account.ledger
```

This command currently prints the formatted output into standard output.
In future in-place format would be provided, also to emit diffs to be used as Git hook.

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
