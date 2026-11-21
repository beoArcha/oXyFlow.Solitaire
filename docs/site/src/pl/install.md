[English](../install.md) · **Polski** · [Deutsch](../de/install.md) · [Español](../es/install.md) · [Français](../fr/install.md) · [Italiano](../it/install.md) · [Nederlands](../nl/install.md) · [Português (Brasil)](../pt-br/install.md) · [Українська](../uk/install.md)

# Instalacja

## Pobierz gotową wersję

Najnowsza wersja to [{{VERSION}}]({{RELEASE_URL}}) (łącznie z przedpremierowymi). Wszystkie wersje są na
[stronie wydań](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| System | Archiwum |
|---|---|
| Windows (x86-64) | `oxysol-<wersja>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<wersja>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<wersja>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<wersja>-aarch64-apple-darwin.tar.gz` |

Rozpakuj archiwum i uruchom `oxysol` (w systemie Windows `oxysol.exe`) w terminalu. Archiwa z dopiskiem `-portable`
trzymają pliki gry w folderze `data` obok programu. Wersje na Linuksa nie mają dźwięku. Plik `SHA256SUMS` na stronie
wydania pozwala sprawdzić pobrany plik.

## Zbuduj ze źródeł

Budujesz gałąź `release`, czyli ostatni commit, który przeszedł pełną kontrolę w systemach Linux, Windows i macOS.

**Potrzebny jest Rust.** Zainstaluj go przez [rustup](https://rustup.rs). Dokładna wersja narzędzi jest przypięta w
pliku `rust-toolchain.toml`; rustup sam ją znajdzie i zainstaluje przy pierwszej kompilacji, więc nic więcej nie
trzeba ustawiać. Kompilacja buduje też odrobinę kodu w C (SQLite), więc potrzebny jest kompilator C:

- **Windows:** Visual Studio Build Tools (lub Visual Studio) z pakietem „Programowanie aplikacji klasycznych w
  języku C++” (ang. „Desktop development with C++”). rustup proponuje ich instalację.
- **macOS:** narzędzia wiersza poleceń Xcode: `xcode-select --install`.
- **Linux:** kompilator C (`gcc` lub `cc`) i `curl`. Dla dźwięku także nagłówki ALSA, na przykład
  `sudo apt install build-essential libasound2-dev` w Debianie i Ubuntu. Bez nich zbuduj program poleceniem
  `cargo build --release --no-default-features`, które pomija dźwięk.

Pierwsza kompilacja trwa kilka minut.

### Windows (PowerShell)

Sprawdzone w systemie Windows 10 dnia 2026-11-19.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Jeszcze niesprawdzone: polecenia są napisane, ale nie uruchomione.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Jeszcze niesprawdzone: polecenia są napisane, ale nie uruchomione.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Gdzie ląduje program

Kompilacja zapisuje program w `target/release/oxysol` (w systemie Windows `target\release\oxysol.exe`) wewnątrz
rozpakowanego folderu. Uruchom go stamtąd albo skopiuj do folderu z listy `PATH`.

### Inaczej: cargo install

Jeśli wolisz, cargo może pobrać, zbudować i zainstalować program jednym poleceniem. Obowiązują te same wymagania
co wyżej.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Polecenie umieszcza `oxysol` w folderze `bin` cargo (`~/.cargo/bin`, a w systemie Windows
`%USERPROFILE%\.cargo\bin`), który rustup dodaje do `PATH`. Potem uruchom `oxysol`. Sprawdzone w systemie
Windows 10 dnia 2026-11-19; w systemach Linux i macOS jeszcze niesprawdzone.

Aby zaktualizować, powtórz kroki: pobrane archiwum zawsze zawiera najnowszą gałąź `release`.
