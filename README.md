<div align="center">
    <h1>Nuze</h1>
    <p><strong>A Nu shell for Zenoh: debug systems, write (end-to-end) tests and build powerful CLI tools</strong></p>
    <sub>Built by the <a href="https://zenoh.io">Zenoh</a> team at <a href="https://www.zettascale.tech">ZettaScale</a></sub>
</div>

## Demo

[![asciicast](https://asciinema.org/a/Uy6yvpT86vWzYW5DmWBfLcc8V.svg)](https://asciinema.org/a/Uy6yvpT86vWzYW5DmWBfLcc8V)

## Usage

Nuze is available on crates.io:

```bash
cargo install nuze
```

A REPL instance supports multiple Zenoh sessions each identified with a name (a Nu string).
On startup, a session named `default` is created. All commands use this session unless
the argument `--session (-s)` is supplied:

```console
$ nuze
41aa8953> zenoh session list
╭───┬─────────┬──────────────────────────────────╮
│ # │  name   │               zid                │
├───┼─────────┼──────────────────────────────────┤
│ 0 │ default │ 41aa8953ad1abda60a9149e25c54067d │
╰───┴─────────┴──────────────────────────────────╯
41aa8953> zenoh zid -s default --short
41aa8953
```

If you would like to start Nuze without the `default` session, use the `--no-default-session (-0)` argument.

The Nuze CLI can be consulted with:

```console
$ nuze --help
```

To get the list of available commands:

```console
41aa8953> help zenoh
```

To get help on a specific command:

```console
41aa8953> help zenoh liveliness declare-token
```
