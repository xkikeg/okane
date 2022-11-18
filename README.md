# okane

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)
[![crates.io](https://img.shields.io/crates/v/okane)](https://crates.io/crates/okane)

Rust based plain text accounting software, influenced by [ledger](https://github.com/ledger/ledger/).

Currently this software is developed just to meet author personal needs, and implementing two features.
*  `format` is to format given Ledger file into organized format.
*  `import` is to convert various source including CSV, ISO Camt053 XML into Ledger format.

## How to use

Disclaimer: This software is still in early phase, subject to any kind of change.

### Format the file

```shell
cargo build
RUST_LOG=info ./target/debug/okane format ~/ledger/account.ledger
```

This command currently prints the formatted output into standard output.
I'll work to replace the files in-place, also to emit diffs to be used as Git hook.

### Import CSV or ISO Camt053 XML files

First you need to write YAML file to control import behavior. We'll assume those are placed under `~/ledger/`.
The format of YAML is (sorry) not documented, but you can see `tests/testdata` directory for example configuration.

Then run the `okane import` command with logging and redirecting to `/dev/null`. This way you can check unhandled entries.

```shell
$ cargo build
$ RUST_LOG=info ./target/debug/okane import --config ~/ledger/import.yml ~/ledger/input_file.csv > /dev/null
```

After iterating over the logs and modifying YAML file, you can redirect the standard output to the ledger file.

```shell
$ cargo build
$ RUST_LOG=info ./target/debug/okane import --config ~/ledger/import.yml ~/ledger/input_file.csv >> ~/ledger/output_path.ledger
```

Tips: You probably don't want to handle all the entries, rather should aim to cover 80-90% of entries initially.
