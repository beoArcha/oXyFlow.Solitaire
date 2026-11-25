[English](../install.md) · [Polski](../pl/install.md) · [Deutsch](../de/install.md) · [Español](../es/install.md) · [Français](../fr/install.md) · [Italiano](../it/install.md) · **Nederlands** · [Português (Brasil)](../pt-br/install.md) · [Українська](../uk/install.md)

# Installeren

## Een kant-en-klare versie downloaden

De nieuwste versie is [{{VERSION}}]({{RELEASE_URL}}) (voorlopige versies inbegrepen). Alle versies staan op de
[releasepagina](https://github.com/beoArcha/oXyFlow.Solitaire/releases).

| Systeem | Archief |
|---|---|
| Windows (x86-64) | `oxysol-<version>-x86_64-pc-windows-msvc.zip` |
| Linux (x86-64) | `oxysol-<version>-x86_64-unknown-linux-musl.tar.gz` |
| Linux (ARM64) | `oxysol-<version>-aarch64-unknown-linux-musl.tar.gz` |
| macOS (Apple silicon) | `oxysol-<version>-aarch64-apple-darwin.tar.gz` |

Pak het archief uit en start `oxysol` (`oxysol.exe` op Windows) in een terminal. De archieven met `-portable` in de
naam bewaren de bestanden van het spel in een map `data` naast het programma. De Linux-versies hebben geen geluid. Met
`SHA256SUMS` op de releasepagina kun je een download controleren.

## Bouwen vanuit de broncode

Je bouwt de branch `release`, met de laatste commit die de volledige controle op Linux, Windows en macOS heeft
doorstaan.

### Met de installatiescripts (aanbevolen)

De scripts in de map `build/` controleren eerst alle vereiste afhankelijkheden en bouwen en installeren `oxysol` daarna
in `~/.cargo/bin` (`%USERPROFILE%\.cargo\bin` op Windows):

- `--check` — stopt bij de eerste ontbrekende afhankelijkheid, met een exitcode ongelijk aan nul.
- `--list` — somt alle ontbrekende afhankelijkheden op, met het installatiecommando voor jouw systeem.
- `--install` — vraagt om toestemming voordat ontbrekende afhankelijkheden via je pakketbeheerder worden geïnstalleerd,
  controleert opnieuw en bouwt alleen als de tweede controle slaagt.
- `--no-audio` — slaat op Linux de ALSA/`pkg-config`-controle over en bouwt met `--no-default-features`.

Zonder modusvlag vraagt een interactieve terminal welke modus je wilt; een niet-interactieve run gedraagt zich als
`--list`.

**Linux en macOS** (`--check` en `--list` gecontroleerd in CI; `--install` nog niet gecontroleerd):

```sh
curl -fsSL https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.sh | sh -s -- --list
```

**Windows (PowerShell)** (`--check` en `--list` gecontroleerd op Windows; `--install` nog niet gecontroleerd):

```powershell
& ([scriptblock]::Create((irm https://raw.githubusercontent.com/beoArcha/oXyFlow.Solitaire/main/build/install.ps1))) --list
```

Vervang `--list` door `--install` om ontbrekende afhankelijkheden te installeren en te bouwen.

### Handmatige stappen

**Rust is nodig.** Installeer het met [rustup](https://rustup.rs). De exacte toolchain staat vast in
`rust-toolchain.toml`; rustup herkent die vanzelf en installeert hem bij de eerste build, dus verder hoef je niets in te
stellen. De build compileert ook een beetje C-code (SQLite), dus je hebt een C-toolchain nodig:

- **Windows:** de Visual Studio Build Tools (of Visual Studio) met de workload ‘Desktopontwikkeling met C++’
  (‘Desktop development with C++’). rustup biedt aan ze te installeren.
- **macOS:** de Xcode-opdrachtregeltools: `xcode-select --install`.
- **Linux:** een C-compiler (`gcc` of `cc`), `curl` en `tar`. Voor geluid ook `pkg-config` en de ALSA-headers,
  bijvoorbeeld `sudo apt install build-essential pkg-config libasound2-dev` op Debian en Ubuntu. Zonder die headers bouw je met
  `cargo build --release --no-default-features`, dan zonder geluid.

De eerste build duurt een paar minuten.

### Windows (PowerShell)

Gecontroleerd op Windows 10 op 19-11-2026.

```powershell
curl.exe -L -o oxysol-source.zip https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.zip
Expand-Archive oxysol-source.zip -DestinationPath oxysol-source
cd oxysol-source\oXyFlow.Solitaire-release
cargo build --release
.\target\release\oxysol.exe
```

### Linux

Nog niet gecontroleerd: de opdrachten zijn opgeschreven, niet uitgevoerd.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### macOS

Nog niet gecontroleerd: de opdrachten zijn opgeschreven, niet uitgevoerd.

```sh
curl -L -o oxysol-source.tar.gz https://github.com/beoArcha/oXyFlow.Solitaire/archive/refs/heads/release.tar.gz
tar -xzf oxysol-source.tar.gz
cd oXyFlow.Solitaire-release
cargo build --release
./target/release/oxysol
```

### Waar het programma terechtkomt

De build schrijft het programma naar `target/release/oxysol` (`target\release\oxysol.exe` op Windows) in de uitgepakte
map. Start het vanaf daar of kopieer het naar een map in je `PATH`.

### Alternatief: cargo install

Als je dat liever hebt, kan cargo het programma in één stap ophalen, bouwen en installeren. De vereisten zijn dezelfde
als hierboven.

```sh
cargo install --git https://github.com/beoArcha/oXyFlow.Solitaire --branch release --locked oxysol
```

Daarmee komt `oxysol` in de `bin`-map van cargo (`~/.cargo/bin`, of `%USERPROFILE%\.cargo\bin` op Windows), die rustup
aan je `PATH` toevoegt. Start daarna `oxysol`. Gecontroleerd op Windows 10 op 19-11-2026; op Linux en macOS nog niet.

Om bij te werken herhaal je de stappen: de download bevat altijd de nieuwste stand van de branch `release`.
