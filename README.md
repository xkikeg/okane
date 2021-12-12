# okane

[![CircleCI](https://circleci.com/gh/xkikeg/okane/tree/main.svg?style=svg)](https://circleci.com/gh/xkikeg/okane/tree/main)

Rust based plain text accounting software, influenced by [ledger](https://github.com/ledger/ledger/).

Currently this software is developed just to meet author personal needs, and implementing `import` functionality from CSV to Ledger.

## How to use

Disclaimer: This software is still in early phase, subject to any kind of change.

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
