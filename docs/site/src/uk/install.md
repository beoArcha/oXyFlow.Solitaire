[English](../install.md) · [Polski](../pl/install.md) · [Deutsch](../de/install.md) · [Español](../es/install.md) · [Français](../fr/install.md) · [Italiano](../it/install.md) · [Nederlands](../nl/install.md) · [Português (Brasil)](../pt-br/install.md) · **Українська**

# Встановлення

## Завантажити готову збірку

Найновіша збірка — [{{VERSION}}]({{RELEASE_URL}}) (включно з попередніми версіями). Усі збірки є на
[сторінці випусків](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| Система | Архів |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Розпакуйте архів і запустіть `oxysol` (у Windows — `oxysol.exe`) у терміналі. Архіви з `-portable` у назві зберігають
файли гри в теці `data` поруч із програмою. Збірки для Linux не мають звуку. Файл `SHA256SUMS` на сторінці випуску дає
змогу перевірити завантаження.

## Зібрати з вихідного коду

Збирають гілку `release`: у ній останній коміт, що пройшов повну перевірку в Linux, Windows і macOS.

**Потрібен Rust.** Встановіть його за допомогою [rustup](https://rustup.rs). Точну версію інструментарію закріплено в
`rust-toolchain.toml`; rustup сам її розпізнає та встановить під час першої збірки, тож інших налаштувань не потрібно.
Збірка також компілює трохи коду на C (SQLite), тому потрібен інструментарій C:

- **Windows:** Visual Studio Build Tools (або Visual Studio) з робочим навантаженням «Розробка класичних застосунків
  на C++» («Desktop development with C++»). rustup запропонує їх встановити.
- **macOS:** інструменти командного рядка Xcode: `xcode-select --install`.
- **Linux:** компілятор C (`gcc` або `cc`) і `curl`. Для звуку — ще й заголовки ALSA, наприклад
  `sudo apt install build-essential libasound2-dev` у Debian та Ubuntu. Без них збирайте командою
  `cargo build --release --no-default-features`, тоді звуку не буде.

Перша збірка триває кілька хвилин.

### Windows (PowerShell)

Перевірено у Windows 10 19.11.2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Ще не перевірено: команди записано, але не виконано.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Ще не перевірено: команди записано, але не виконано.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Де опиниться програма

Збірка записує програму в `target/release/oxysol` (у Windows — `target\release\oxysol.exe`) усередині розпакованої
теки. Запускайте її звідти або скопіюйте в теку зі свого `PATH`.

### Інший спосіб: cargo install

Якщо хочете, cargo може завантажити, зібрати й встановити програму за один крок. Вимоги ті самі, що й вище.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Так `oxysol` потрапить у теку `bin` cargo (`~/.cargo/bin`, у Windows — `%USERPROFILE%\.cargo\bin`), яку rustup додає до
вашого `PATH`. Потім запустіть `oxysol`. Перевірено у Windows 10 19.11.2026; у Linux і macOS ще не перевірено.

Щоб оновити гру, повторіть ці кроки: завантаження завжди містить найновіший стан гілки `release`.
