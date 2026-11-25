[English](../install.md) · [Polski](../pl/install.md) · [Deutsch](../de/install.md) · [Español](../es/install.md) · [Français](../fr/install.md) · **Italiano** · [Nederlands](../nl/install.md) · [Português (Brasil)](../pt-br/install.md) · [Українська](../uk/install.md)

# Installazione

## Scarica una versione pronta

La versione più recente è [{{VERSION}}]({{RELEASE_URL}}) (versioni preliminari comprese). Tutte le versioni sono nella
[pagina delle release](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| Sistema | Archivio |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Estrai l’archivio ed esegui `oxysol` (`oxysol.exe` su Windows) in un terminale. Gli archivi con `-portable` nel nome
tengono i file del gioco in una cartella `data` accanto al programma. Le versioni per Linux non hanno l’audio. Il file
`SHA256SUMS` nella pagina della release permette di verificare un download.

## Compila dai sorgenti

Si compila il branch `release`, che contiene l’ultimo commit ad aver superato il controllo completo su Linux, Windows
e macOS.

### Con gli script di installazione (consigliato)

Gli script nella cartella `build/` controllano prima tutte le dipendenze richieste, poi compilano e installano `oxysol`
in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` su Windows):

- `--check` — si ferma alla prima dipendenza mancante, con un codice di uscita diverso da zero.
- `--list` — elenca tutte le dipendenze mancanti e il comando di installazione per il tuo sistema.
- `--install` — chiede conferma prima di installare le dipendenze mancanti con il gestore di pacchetti, ricontrolla e
  compila solo se il secondo controllo ha successo.
- `--no-audio` — salta su Linux il controllo di ALSA/`pkg-config` e compila con `--no-default-features`.

Senza un’opzione di modalità, un terminale interattivo chiede quale modalità usare; un’esecuzione non interattiva si
comporta come `--list`.

**Linux e macOS** (`--check` e `--list` verificati in CI; `--install` non ancora verificato):

```sh
curl -fsSL https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.sh | sh -s -- --list
```

**Windows (PowerShell)** (`--check` e `--list` verificati su Windows; `--install` non ancora verificato):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.ps1))) --list
```

Sostituisci `--list` con `--install` per installare le dipendenze mancanti e compilare.

### Passaggi manuali

**Serve Rust.** Installalo con [rustup](https://rustup.rs). La toolchain esatta è fissata in `rust-toolchain.toml`;
rustup la riconosce da solo e la installa alla prima compilazione, quindi non serve nessun’altra configurazione. La
compilazione include anche un po’ di codice C (SQLite), quindi serve una toolchain C:

- **Windows:** i Build Tools di Visual Studio (o Visual Studio) con il carico di lavoro «Sviluppo di applicazioni
  desktop con C++» («Desktop development with C++»). rustup si offre di installarli.
- **macOS:** gli strumenti da riga di comando di Xcode: `xcode-select --install`.
- **Linux:** un compilatore C (`gcc` o `cc`), `curl` e `tar`. Per l’audio servono anche `pkg-config` e gli header di ALSA,
  per esempio `sudo apt install build-essential pkg-config libasound2-dev` su Debian e Ubuntu. Senza di essi compila con
  `cargo build --release --no-default-features`, che esclude l’audio.

La prima compilazione richiede qualche minuto.

### Windows (PowerShell)

Verificato su Windows 10 il 19/11/2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Non ancora verificato: i comandi sono scritti, non eseguiti.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Non ancora verificato: i comandi sono scritti, non eseguiti.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Dove finisce il programma

La compilazione scrive il programma in `target/release/oxysol` (`target\release\oxysol.exe` su Windows) dentro la
cartella estratta. Avvialo da lì oppure copialo in una cartella del tuo `PATH`.

### In alternativa: cargo install

Se preferisci, cargo può scaricare, compilare e installare il programma in un solo passaggio. I requisiti sono gli
stessi indicati sopra.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Così `oxysol` finisce nella cartella `bin` di cargo (`~/.cargo/bin`, oppure `%USERPROFILE%\.cargo\bin` su Windows), che
rustup aggiunge al tuo `PATH`. Poi esegui `oxysol`. Verificato su Windows 10 il 19/11/2026; su Linux e macOS non ancora
verificato.

Per aggiornare, ripeti i passaggi: il download contiene sempre la versione più recente del branch `release`.
