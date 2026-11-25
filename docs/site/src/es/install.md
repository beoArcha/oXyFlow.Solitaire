[English](../install.md) · [Polski](../pl/install.md) · [Deutsch](../de/install.md) · **Español** · [Français](../fr/install.md) · [Italiano](../it/install.md) · [Nederlands](../nl/install.md) · [Português (Brasil)](../pt-br/install.md) · [Українська](../uk/install.md)

# Instalación

## Descarga una versión lista para usar

La versión más reciente es [{{VERSION}}]({{RELEASE_URL}}) (incluidas las preliminares). Todas las versiones están en la
[página de versiones](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| Sistema | Archivo |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Descomprime el archivo y ejecuta `oxysol` (`oxysol.exe` en Windows) en una terminal. Los archivos con `-portable` en el
nombre guardan los datos del juego en una carpeta `data` junto al programa. Las versiones para Linux no tienen sonido.
El archivo `SHA256SUMS` de la página de la versión permite comprobar una descarga.

## Compila desde el código fuente

Se compila la rama `release`, que contiene el último commit que superó la comprobación completa en Linux, Windows y
macOS.

### Con los scripts de instalación (recomendado)

Los scripts de la carpeta `build/` comprueban primero todas las dependencias necesarias y solo después compilan e
instalan `oxysol` en `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` en Windows):

- `--check` — se detiene en la primera dependencia que falte, con un código de salida distinto de cero.
- `--list` — muestra todas las dependencias que faltan y el comando de instalación para tu sistema.
- `--install` — pide confirmación antes de instalar las dependencias que falten con tu gestor de paquetes, vuelve a
  comprobar y compila solo si la segunda comprobación pasa.
- `--no-audio` — omite en Linux la comprobación de ALSA/`pkg-config` y compila con `--no-default-features`.

Sin opción de modo, una terminal interactiva pregunta qué modo usar; una ejecución no interactiva se comporta como
`--list`.

**Linux y macOS** (`--check` y `--list` comprobados en CI; `--install` aún no comprobado):

```sh
curl -fsSL https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.sh | sh -s -- --list
```

**Windows (PowerShell)** (`--check` y `--list` comprobados en Windows; `--install` aún no comprobado):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.ps1))) --list
```

Sustituye `--list` por `--install` para instalar las dependencias que falten y compilar.

### Pasos manuales

**Necesitas Rust.** Instálalo con [rustup](https://rustup.rs). La toolchain exacta está fijada en `rust-toolchain.toml`;
rustup la detecta por sí solo y la instala en la primera compilación, así que no hace falta ninguna otra configuración.
La compilación también incluye un poco de código C (SQLite), por lo que se necesita una toolchain de C:

- **Windows:** las Build Tools de Visual Studio (o Visual Studio) con la carga de trabajo «Desarrollo para el escritorio
  con C++» («Desktop development with C++»). rustup ofrece instalarlas.
- **macOS:** las herramientas de línea de comandos de Xcode: `xcode-select --install`.
- **Linux:** un compilador de C (`gcc` o `cc`), `curl` y `tar`. Para el sonido, también `pkg-config` y las cabeceras de
  ALSA, por ejemplo `sudo apt install build-essential pkg-config libasound2-dev` en Debian y Ubuntu. Sin ellas, compila con
  `cargo build --release --no-default-features`, que deja fuera el sonido.

La primera compilación tarda unos minutos.

### Windows (PowerShell)

Comprobado en Windows 10 el 19/11/2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Aún no comprobado: los comandos están escritos, pero no se han ejecutado.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Aún no comprobado: los comandos están escritos, pero no se han ejecutado.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Dónde queda el programa

La compilación escribe el programa en `target/release/oxysol` (`target\release\oxysol.exe` en Windows) dentro de la
carpeta descomprimida. Ejecútalo desde ahí o cópialo a una carpeta de tu `PATH`.

### Alternativa: cargo install

Si lo prefieres, cargo puede descargar, compilar e instalar el programa en un solo paso. Los requisitos son los mismos
que arriba.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Así `oxysol` queda en la carpeta `bin` de cargo (`~/.cargo/bin`, o `%USERPROFILE%\.cargo\bin` en Windows), que rustup
añade a tu `PATH`. Después ejecuta `oxysol`. Comprobado en Windows 10 el 19/11/2026; en Linux y macOS aún no se ha
comprobado.

Para actualizar, repite los pasos: la descarga siempre contiene la versión más reciente de la rama `release`.
