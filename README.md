# oXySol

Ultimate Office Solitaire for the terminal. Klondike, Spider, FreeCell and Pyramid in one TUI
application, binary `oxysol`.

## Run the check

The toolchain is pinned in `rust-toolchain.toml`. From the repository root:

```sh
cargo run                    # prints the version
cargo xtask check --quick    # fmt, clippy, lint, unit and integration tests
cargo xtask check            # the above plus architecture and e2e tests and sync-metadata --check
```

## Portable copy

The `-portable` archive runs from any folder. Unzip it anywhere: the game keeps its files
(`oxysol.db`, `oxysol.log`) in a `data` folder next to the program and writes nothing to system
folders. To switch back to the system data folder, delete the `oxysol.portable` file next to the
program.

## License

MIT. See [LICENSE](LICENSE).
