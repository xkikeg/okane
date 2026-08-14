# Demo recordings

The terminal recordings embedded in [README.md](../../README.md) and
[README.ja.md](../../README.ja.md), plus the script that produces them.

| cast | scene |
| ---- | ----- |
| `okane-ui.cast` | `okane ui` against `testdata/report/multi_commodity.ledger`: the `?` key help, scrolling the flat balance, tree mode, folding, search, drilling into the register. |
| `okane-import.cast` | `okane import --interactive` against `testdata/import/index_amount.csv`: accepting a row, fixing an `Expenses:Unknown` through the account prompt, skipping a row, writing the result. |

## Re-recording

```shell
./doc/demo/record.sh          # both scenes
./doc/demo/record.sh ui       # just one
./doc/demo/record.sh import
```

The script builds `target/release/okane` if it is missing, then overwrites the
`.cast` files in place. Play one back locally with:

```shell
asciinema play doc/demo/okane-ui.cast
```

The scenes only ever read from `testdata/`; every file the demo writes to
(`out.ledger` in the import scene) lives in a temporary scratch directory that
is removed afterwards.

## Why it goes through tmux

`asciinema rec` records whatever terminal it is started in, and the scenes need
keystrokes fed to a TUI. So each scene starts a **detached tmux session with an
explicit geometry** and runs `asciinema rec` inside it:

- A detached tmux pane is a real pty of exactly the requested size, so the cast
  is always 100x30 regardless of the terminal (or CI job) that ran the script.
  80 columns is too narrow — the balance screen's footer hint truncates
  mid-word.
- `tmux send-keys` can then drive the TUI, and `tmux capture-pane` lets the
  script *wait for* a screen instead of guessing at sleeps.
- A private socket (`-L okane-demo`) plus a generated config (`-f`) keeps the
  user's `~/.tmux.conf` and any running tmux server out of the picture, and the
  inner shell is `bash --noprofile --rcfile` with a fixed prompt for the same
  reason. tmux's own status bar is turned off so it never lands in the cast.

## Publishing

The casts are hosted on [asciinema.org](https://asciinema.org) and embedded in
the READMEs as `https://asciinema.org/a/<id>.svg` thumbnails.

```shell
asciinema auth                              # once, to claim uploads
asciinema upload doc/demo/okane-ui.cast     # prints the recording URL
```

Authenticate first: asciinema.org deletes *unclaimed* anonymous recordings after
7 days, which would leave the README pointing at a dead link. Re-uploading a
re-recorded cast produces a **new** id, so the README links need updating too.
