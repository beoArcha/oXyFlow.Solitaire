**English** · [Polski](pl/install.md) · [Deutsch](de/install.md) · [Español](es/install.md) · [Français](fr/install.md) · [Italiano](it/install.md) · [Nederlands](nl/install.md) · [Português (Brasil)](pt-br/install.md) · [Українська](uk/install.md)

# Install

## Download a build

The newest build is [{{VERSION}}]({{RELEASE_URL}}) (pre-releases included). Every build is on the
[releases page](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| System | Archive |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Unpack the archive and run `oxysol` (`oxysol.exe` on Windows) in a terminal. The archives named `-portable` keep
the game's files in a `data` folder next to the program. The Linux builds have no sound. `SHA256SUMS` on the release
page lets you check a download.

## Build from source

You build the `release` branch, which holds the last commit that passed the full check on Linux, Windows and macOS.

**Rust is required.** Install it with [rustup](https://rustup.rs). The exact toolchain is pinned in
`rust-toolchain.toml`; rustup picks it up by itself and installs it on the first build, so you need no other setup.
The build also compiles a little C code (SQLite), so a C toolchain is needed:

- **Windows:** the Visual Studio Build Tools (or Visual Studio) with the "Desktop development with C++" workload.
  rustup offers to install them.
- **macOS:** the Xcode command line tools: `xcode-select --install`.
- **Linux:** a C compiler (`gcc` or `cc`) and `curl`. For sound, the ALSA headers too, for example
  `sudo apt install build-essential libasound2-dev` on Debian and Ubuntu. Without them, build with
  `cargo build --release --no-default-features`, which leaves the sound out.

The first build takes a few minutes.

### Windows (PowerShell)

Verified on Windows 10 on 2026-11-19.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Not verified yet: the commands are written, not run.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Not verified yet: the commands are written, not run.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Where the program ends up

The build writes the program to `target/release/oxysol` (`target\release\oxysol.exe` on Windows) inside the unpacked
folder. Run it from there, or copy it to a folder on your `PATH`.

### Alternative: cargo install

If you prefer, cargo can fetch, build and install the program in one step. It needs the same requirements as above.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

This puts `oxysol` in cargo's `bin` folder (`~/.cargo/bin`, or `%USERPROFILE%\.cargo\bin` on Windows), which rustup
adds to your `PATH`. Then run `oxysol`. Verified on Windows 10 on 2026-11-19; on Linux and macOS it is not verified yet.

To update, repeat the steps: the download always holds the newest `release` branch.
