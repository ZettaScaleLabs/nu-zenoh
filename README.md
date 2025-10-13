# `nu-zenoh`

## `nuze`: Zenoh Nu Shell

`nuze` is an experimental Zenoh shell built with [Nushell](https://www.nushell.sh/)
to provide an interactive environment for debugging [Zenoh](https://zenoh.io/) systems. To get started:

```bash
cargo install --git https://github.com/ZettaScaleLabs/nu-zenoh.git nuze
nuze
```

A REPL instance supports multiple Zenoh sessions each identified by a name (string).
On startup, a session named `default` is created. All commands use this session unless
the argument `-s, --session` is supplied:

```console
> zenoh session list
╭───┬─────────┬──────────────────────────────────╮
│ # │  name   │               zid                │
├───┼─────────┼──────────────────────────────────┤
│ 0 │ default │ 41aa8953ad1abda60a9149e25c54067d │
╰───┴─────────┴──────────────────────────────────╯
> zenoh zid
41aa8953ad1abda60a9149e25c54067d
```

To get the list of available commands:

```console
> zenoh --help
```
